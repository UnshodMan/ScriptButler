use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::json;
use scriptbutler_core::exec::{build_run_command, shell_command, to_view};
use scriptbutler_core::models::{EntryView, RunScheme, ScriptEntry};
use scriptbutler_core::ops::{self, CoreError, ManageOptions};
use scriptbutler_core::search;
use scriptbutler_core::store::{self, StoreError};
use scriptbutler_core::unix_ms;

use crate::env::{self, EnvAction};
use crate::output::{self, CliError};
use crate::resolve;
use crate::skill;
use crate::{AddArgs, Commands, EnvCommands, ManageArgs, RefArgs, RunArgs};

pub fn run(command: Commands, json: bool) -> Result<i32, CliError> {
    match command {
        Commands::Add(args) => cmd_add(args, json),
        Commands::Remove(args) => cmd_remove(args, json),
        Commands::Manage(args) => cmd_manage(args, json),
        Commands::List { query } => cmd_list(query, json),
        Commands::Show(args) => cmd_show(args, json),
        Commands::Search { keyword, content } => cmd_search(&keyword, content, json),
        Commands::Run(args) => cmd_run(args, json),
        Commands::Path => cmd_path(json),
        Commands::Env { command } => cmd_env(command, json),
        Commands::Skill { install } => skill::run(install, json).map(|_| 0),
    }
}

struct Store {
    data_dir: PathBuf,
    path: PathBuf,
    entries: Vec<ScriptEntry>,
}

impl Store {
    fn load() -> Result<Self, CliError> {
        let data_dir = store::data_dir().map_err(CliError::Io)?;
        let path = data_dir.join("scripts.json");
        let entries = store::try_load_entries(&path).map_err(map_store)?;
        Ok(Self {
            data_dir,
            path,
            entries,
        })
    }

    fn save(&self) -> Result<(), CliError> {
        store::save_entries(&self.path, &self.entries).map_err(CliError::Io)
    }

    fn managed_dir(&self) -> PathBuf {
        store::managed_dir(&self.data_dir)
    }
}

fn map_store(err: StoreError) -> CliError {
    match err {
        StoreError::Io(detail) => CliError::Io(detail),
        StoreError::InvalidData(detail) => CliError::InvalidData(detail),
    }
}

fn map_core(err: CoreError) -> CliError {
    match err {
        CoreError::NotFound => CliError::NotFound("file not found".to_string()),
        CoreError::NotFile => CliError::NotFile("path is not a file".to_string()),
        CoreError::InvalidData(detail) => CliError::InvalidData(detail),
        CoreError::Io(detail) => CliError::Io(detail),
        CoreError::ManageFailed(detail) => CliError::ManageFailed(detail),
    }
}

fn absolute_path(path: &str) -> String {
    resolve::absolute(path).to_string_lossy().to_string()
}

fn cmd_add(args: AddArgs, json: bool) -> Result<i32, CliError> {
    let mut store = Store::load()?;
    let path = absolute_path(&args.path);
    // 相对路径已被登记时，沿用条目里保存的绝对路径
    let path = store
        .entries
        .iter()
        .find(|e| e.path == path || e.path == args.path)
        .map(|e| e.path.clone())
        .unwrap_or(path);

    let created = ops::add_entry(&mut store.entries, &path, &args.scheme_name)
        .map_err(map_core)?
        .created;

    if let Some(entry) = store.entries.iter_mut().find(|e| e.path == path) {
        if let Some(name) = &args.name {
            entry.name = name.clone();
        }
        if let Some(command) = &args.command {
            let active_idx = entry
                .active_scheme_id
                .as_ref()
                .and_then(|id| entry.schemes.iter().position(|s| &s.id == id))
                .or_else(|| if entry.schemes.is_empty() { None } else { Some(0) });
            if let Some(i) = active_idx {
                entry.schemes[i].command = command.clone();
            } else {
                let scheme = RunScheme {
                    id: ops::new_id(),
                    name: args.scheme_name.clone(),
                    command: command.clone(),
                    cd_to_script_dir: true,
                };
                entry.active_scheme_id = Some(scheme.id.clone());
                entry.schemes.push(scheme);
            }
        }
    }

    let view = store
        .entries
        .iter()
        .find(|e| e.path == path)
        .map(to_view)
        .ok_or_else(|| CliError::NotFound("script was not registered".to_string()))?;
    store.save()?;

    if json {
        output::print_json(json!({
            "ok": true,
            "created": created,
            "copiedTo": serde_json::Value::Null,
            "data": view
        }));
    } else if created {
        println!("Added {} -> {}", view.entry.name, view.entry.path);
    } else {
        println!("Updated {} -> {}", view.entry.name, view.entry.path);
    }
    Ok(0)
}

fn cmd_manage(args: ManageArgs, json: bool) -> Result<i32, CliError> {
    let mut store = Store::load()?;
    let source = absolute_path(&args.path);
    let managed_dir = store.managed_dir();
    let outcome = ops::manage_script(
        &mut store.entries,
        &source,
        &managed_dir,
        &ManageOptions {
            scheme_name: &args.scheme_name,
            name: args.name.as_deref(),
            command: args.command.as_deref(),
            work_dir: args.workdir.as_deref(),
            dedupe: true,
        },
    )
    .map_err(map_core)?;
    store.save()?;

    if json {
        output::print_json(json!({
            "ok": true,
            "created": outcome.created,
            "copiedTo": outcome.copied_to,
            "data": outcome.view
        }));
    } else if outcome.created {
        println!("Managed {} -> {}", outcome.view.entry.name, outcome.copied_to);
    } else {
        println!(
            "Already managed: {} -> {}",
            outcome.view.entry.name, outcome.copied_to
        );
    }
    Ok(0)
}

fn cmd_remove(args: RefArgs, json: bool) -> Result<i32, CliError> {
    let mut store = Store::load()?;
    let idx = resolve::resolve_index(&store.entries, &args.reference)?;
    let path = store.entries[idx].path.clone();
    ops::remove_entry(&mut store.entries, &path);
    store.save()?;

    if json {
        output::print_json(json!({
            "ok": true,
            "data": { "path": path, "removed": true }
        }));
    } else {
        println!("Removed {path}");
    }
    Ok(0)
}

fn cmd_list(query: Option<String>, json: bool) -> Result<i32, CliError> {
    let store = Store::load()?;
    let query = query
        .map(|q| q.trim().to_lowercase())
        .filter(|q| !q.is_empty());
    let views: Vec<EntryView> = ops::sorted_views(&store.entries)
        .into_iter()
        .filter(|view| {
            query
                .as_ref()
                .map(|q| {
                    view.entry.name.to_lowercase().contains(q)
                        || view
                            .exec_dir
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(q)
                })
                .unwrap_or(true)
        })
        .collect();

    if json {
        output::print_json(json!({ "ok": true, "data": views }));
    } else if views.is_empty() {
        println!("No scripts.");
    } else {
        for view in &views {
            output::print_entry(view);
        }
    }
    Ok(0)
}

fn cmd_show(args: RefArgs, json: bool) -> Result<i32, CliError> {
    let store = Store::load()?;
    let idx = resolve::resolve_index(&store.entries, &args.reference)?;
    let view = to_view(&store.entries[idx]);

    if json {
        output::print_json(json!({ "ok": true, "data": view }));
    } else {
        output::print_entry(&view);
        for scheme in &view.entry.schemes {
            let mark = if Some(&scheme.id) == view.entry.active_scheme_id.as_ref() {
                "*"
            } else {
                " "
            };
            println!("  [{mark}] {}: {}", scheme.name, scheme.command);
        }
    }
    Ok(0)
}

fn cmd_search(keyword: &str, content: bool, json: bool) -> Result<i32, CliError> {
    let store = Store::load()?;
    let sorted = ops::sorted_refs(&store.entries);
    let groups = search::group(&sorted, keyword);
    let by_path: HashMap<&str, &ScriptEntry> =
        sorted.iter().map(|entry| (entry.path.as_str(), *entry)).collect();
    let to_views = |paths: &[String]| -> Vec<EntryView> {
        paths
            .iter()
            .filter_map(|path| by_path.get(path.as_str()).map(|entry| to_view(*entry)))
            .collect()
    };

    let by_name = to_views(&groups.by_name);
    let by_dir = to_views(&groups.by_dir);
    let by_content = if content {
        let hits = search::search_contents(&groups.pending, keyword);
        to_views(&hits)
    } else {
        Vec::new()
    };

    if json {
        output::print_json(json!({
            "ok": true,
            "data": {
                "byName": by_name,
                "byDir": by_dir,
                "byContent": by_content
            }
        }));
    } else {
        let mut any = false;
        for (title, list) in [
            ("Name", &by_name),
            ("Directory", &by_dir),
            ("Content", &by_content),
        ] {
            if list.is_empty() {
                continue;
            }
            any = true;
            println!("{title}:");
            for view in list {
                output::print_entry(view);
            }
        }
        if !any {
            println!("No matches.");
        }
    }
    Ok(0)
}

fn cmd_run(args: RunArgs, json: bool) -> Result<i32, CliError> {
    let mut store = Store::load()?;
    let idx = resolve::resolve_index(&store.entries, &args.reference)?;
    let cmdline = {
        let entry = &store.entries[idx];
        let scheme = resolve::resolve_scheme(entry, args.scheme.as_deref())?;
        build_run_command(entry, scheme)
    };
    let entry_path = store.entries[idx].path.clone();

    let mut child = shell_command(&cmdline)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CliError::SpawnFailed(e.to_string()))?;

    if let Some(entry) = store.entries.iter_mut().find(|e| e.path == entry_path) {
        entry.last_used_at = unix_ms();
        entry.use_count += 1;
    }
    store.save()?;

    let out_buf = Arc::new(Mutex::new(Vec::new()));
    let err_buf = Arc::new(Mutex::new(Vec::new()));
    let out_thread = child
        .stdout
        .take()
        .map(|stream| spawn_reader(stream, json, true, out_buf.clone()));
    let err_thread = child
        .stderr
        .take()
        .map(|stream| spawn_reader(stream, json, false, err_buf.clone()));

    let status = child.wait().map_err(|e| CliError::Io(e.to_string()))?;
    if let Some(thread) = out_thread {
        let _ = thread.join();
    }
    if let Some(thread) = err_thread {
        let _ = thread.join();
    }

    let code = exit_code(&status);
    if json {
        let stdout = out_buf.lock().unwrap().clone();
        let stderr = err_buf.lock().unwrap().clone();
        output::print_json(json!({
            "ok": true,
            "data": { "exitCode": code, "stdout": stdout, "stderr": stderr }
        }));
    }
    Ok(code)
}

fn cmd_path(json: bool) -> Result<i32, CliError> {
    let data_dir = store::data_dir().map_err(CliError::Io)?;
    let data = json!({
        "dataDir": data_dir.to_string_lossy(),
        "storePath": data_dir.join("scripts.json").to_string_lossy(),
        "managedDir": store::managed_dir(&data_dir).to_string_lossy(),
        "executable": std::env::current_exe().ok().map(|p| p.to_string_lossy().to_string())
    });

    if json {
        output::print_json(json!({ "ok": true, "data": data }));
    } else {
        println!("dataDir:    {}", data_dir.display());
        println!("storePath:  {}", data_dir.join("scripts.json").display());
        println!("managedDir: {}", store::managed_dir(&data_dir).display());
        if let Some(exe) = std::env::current_exe().ok() {
            println!("executable: {}", exe.display());
        }
    }
    Ok(0)
}

fn cmd_env(command: EnvCommands, json: bool) -> Result<i32, CliError> {
    let (action, system) = match command {
        EnvCommands::Add { system } => (EnvAction::Add, system),
        EnvCommands::Remove { system } => (EnvAction::Remove, system),
        EnvCommands::Status { system } => (EnvAction::Status, system),
    };
    env::run(action, system, json)?;
    Ok(0)
}

fn spawn_reader<R: Read + Send + 'static>(
    stream: R,
    json: bool,
    is_stdout: bool,
    buffer: Arc<Mutex<Vec<String>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        let mut line = Vec::new();
        loop {
            line.clear();
            match reader.read_until(b'\n', &mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line.last() == Some(&b'\n') {
                        line.pop();
                    }
                    if line.last() == Some(&b'\r') {
                        line.pop();
                    }
                    let text = String::from_utf8_lossy(&line).to_string();
                    if json {
                        buffer.lock().unwrap().push(text);
                    } else if is_stdout {
                        println!("{text}");
                    } else {
                        eprintln!("{text}");
                    }
                }
                Err(_) => break,
            }
        }
    })
}

fn exit_code(status: &std::process::ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }
    1
}
