use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::exec::{dir_of, to_view};
use crate::language;
use crate::models::{
    ApiError, EntryView, ListResponse, RunScheme, ScriptEntry, SearchGroups,
};
use crate::runner;
use crate::search;
use crate::state::AppState;
use crate::unix_ms;

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn file_name(path: &str) -> String {
    path.rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(path)
        .to_string()
}

/// 托管文件名：时间戳 + 原扩展名（保证语言识别仍然有效）
fn stamp_file_name(path: &str) -> String {
    let base = file_name(path);
    let ext = match base.rfind('.') {
        Some(dot) if dot > 0 => &base[dot..],
        _ => "",
    };
    format!("{}{ext}", unix_ms())
}

/// MRU 排序：最近使用 → 使用次数 → 名称
fn sorted_refs(entries: &[ScriptEntry]) -> Vec<&ScriptEntry> {
    let mut list: Vec<&ScriptEntry> = entries.iter().collect();
    list.sort_by(|a, b| {
        b.last_used_at
            .cmp(&a.last_used_at)
            .then_with(|| b.use_count.cmp(&a.use_count))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    list
}

fn sorted_views(entries: &[ScriptEntry]) -> Vec<EntryView> {
    sorted_refs(entries).into_iter().map(to_view).collect()
}

#[tauri::command]
pub fn list_scripts(state: State<AppState>) -> ListResponse {
    let entries = state.entries.lock().unwrap();
    ListResponse {
        entries: sorted_views(&entries),
        changed: None,
    }
}

#[tauri::command]
pub fn add_script(
    state: State<AppState>,
    path: String,
    default_scheme_name: String,
) -> Result<ListResponse, ApiError> {
    let meta = std::fs::metadata(&path).map_err(|_| ApiError::NotFound)?;
    if !meta.is_file() {
        return Err(ApiError::NotFile);
    }

    let mut entries = state.entries.lock().unwrap();
    let changed = if let Some(existing) = entries.iter_mut().find(|e| e.path == path) {
        existing.last_used_at = unix_ms();
        existing.use_count += 1;
        Some(to_view(existing))
    } else {
        let lang = language::detect(&path);
        let scheme = RunScheme {
            id: new_id(),
            name: default_scheme_name,
            command: lang.default_command.to_string(),
            cd_to_script_dir: true,
        };
        let entry = ScriptEntry {
            path: path.clone(),
            name: file_name(&path),
            lang: lang.id.to_string(),
            last_used_at: unix_ms(),
            use_count: 1,
            schemes: vec![scheme.clone()],
            active_scheme_id: Some(scheme.id),
            managed: None,
            source_path: None,
            work_dir: None,
        };
        entries.push(entry);
        entries.last().map(to_view)
    };

    state.save_entries(&entries);
    Ok(ListResponse {
        entries: sorted_views(&entries),
        changed,
    })
}

/// 托管：把脚本复制到应用数据目录下的 scripts/，以时间戳命名。
/// 新条目保留原脚本名、原始路径与全部运行方案，默认在原目录执行。
#[tauri::command]
pub fn manage_script(state: State<AppState>, path: String) -> Result<ListResponse, ApiError> {
    let src = {
        let entries = state.entries.lock().unwrap();
        entries
            .iter()
            .find(|e| e.path == path)
            .cloned()
            .ok_or(ApiError::NotFound)?
    };

    std::fs::create_dir_all(&state.managed_dir).map_err(|e| ApiError::ManageFailed {
        detail: e.to_string(),
    })?;
    let target = state.managed_dir.join(stamp_file_name(&path));
    std::fs::copy(&path, &target).map_err(|e| ApiError::ManageFailed {
        detail: e.to_string(),
    })?;

    let entry = ScriptEntry {
        path: target.to_string_lossy().to_string(),
        name: src.name,
        lang: src.lang,
        last_used_at: unix_ms(),
        use_count: 0,
        schemes: src.schemes,
        active_scheme_id: src.active_scheme_id,
        managed: Some(true),
        source_path: Some(path.clone()),
        work_dir: dir_of(&path),
    };

    let mut entries = state.entries.lock().unwrap();
    entries.push(entry);
    let changed = entries.last().map(to_view);
    state.save_entries(&entries);
    Ok(ListResponse {
        entries: sorted_views(&entries),
        changed,
    })
}

#[tauri::command]
pub fn remove_script(state: State<AppState>, path: String) -> ListResponse {
    let mut entries = state.entries.lock().unwrap();
    let before = entries.len();
    entries.retain(|e| e.path != path);
    if entries.len() != before {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: sorted_views(&entries),
        changed: None,
    }
}

/// 托管脚本：修改执行目录（随时可改，立即持久化）
#[tauri::command]
pub fn set_work_dir(state: State<AppState>, path: String, work_dir: String) -> ListResponse {
    let mut entries = state.entries.lock().unwrap();
    let changed = if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        entry.work_dir = Some(work_dir);
        Some(to_view(entry))
    } else {
        None
    };
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn mark_used(state: State<AppState>, path: String) -> ListResponse {
    let mut entries = state.entries.lock().unwrap();
    let changed = if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        entry.last_used_at = unix_ms();
        entry.use_count += 1;
        Some(to_view(entry))
    } else {
        None
    };
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: sorted_views(&entries),
        changed,
    }
}

/// 首次打开且无方案时，按语言生成默认方案
#[tauri::command]
pub fn ensure_default_scheme(state: State<AppState>, path: String, name: String) -> ListResponse {
    let mut entries = state.entries.lock().unwrap();
    let mut dirty = false;
    let changed = if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        if entry.schemes.is_empty() {
            let lang = language::detect(&entry.path);
            let scheme = RunScheme {
                id: new_id(),
                name,
                command: lang.default_command.to_string(),
                cd_to_script_dir: true,
            };
            entry.active_scheme_id = Some(scheme.id.clone());
            entry.schemes.push(scheme);
            dirty = true;
        }
        Some(to_view(entry))
    } else {
        None
    };
    if dirty {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn create_scheme(
    state: State<AppState>,
    path: String,
    name: String,
    command: String,
    cd_to_script_dir: bool,
) -> ListResponse {
    let mut entries = state.entries.lock().unwrap();
    let changed = if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        let scheme = RunScheme {
            id: new_id(),
            name,
            command,
            cd_to_script_dir,
        };
        entry.active_scheme_id = Some(scheme.id.clone());
        entry.schemes.push(scheme);
        Some(to_view(entry))
    } else {
        None
    };
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn update_scheme(
    state: State<AppState>,
    path: String,
    id: String,
    name: String,
    command: String,
    cd_to_script_dir: bool,
) -> ListResponse {
    let mut entries = state.entries.lock().unwrap();
    let changed = if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        if let Some(scheme) = entry.schemes.iter_mut().find(|s| s.id == id) {
            scheme.name = name;
            scheme.command = command;
            scheme.cd_to_script_dir = cd_to_script_dir;
            entry.active_scheme_id = Some(id);
        }
        Some(to_view(entry))
    } else {
        None
    };
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn delete_scheme(state: State<AppState>, path: String, id: String) -> ListResponse {
    let mut entries = state.entries.lock().unwrap();
    let mut dirty = false;
    let changed = if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        let before = entry.schemes.len();
        entry.schemes.retain(|s| s.id != id);
        if entry.schemes.len() != before {
            let active_valid = entry
                .active_scheme_id
                .as_ref()
                .map(|active| entry.schemes.iter().any(|s| &s.id == active))
                .unwrap_or(false);
            if !active_valid {
                entry.active_scheme_id = entry.schemes.first().map(|s| s.id.clone());
            }
            dirty = true;
        }
        Some(to_view(entry))
    } else {
        None
    };
    if dirty {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn set_active_scheme(state: State<AppState>, path: String, id: String) -> ListResponse {
    let mut entries = state.entries.lock().unwrap();
    let mut dirty = false;
    let changed = if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        if entry.schemes.iter().any(|s| s.id == id) {
            entry.active_scheme_id = Some(id);
            dirty = true;
        }
        Some(to_view(entry))
    } else {
        None
    };
    if dirty {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn read_script(path: String) -> Result<String, ApiError> {
    std::fs::read_to_string(&path).map_err(|e| ApiError::ReadFailed {
        detail: e.to_string(),
    })
}

#[tauri::command]
pub fn write_script(path: String, content: String) -> Result<(), ApiError> {
    std::fs::write(&path, content).map_err(|e| ApiError::WriteFailed {
        detail: e.to_string(),
    })
}

#[tauri::command]
pub fn start_run(
    app: AppHandle,
    state: State<AppState>,
    path: String,
    scheme_id: String,
) -> Result<String, ApiError> {
    runner::start_run(app, &state, &path, &scheme_id)
}

#[tauri::command]
pub fn stop_run(state: State<AppState>, run_id: String) {
    runner::stop_run(&state, &run_id);
}

#[tauri::command]
pub fn search_scripts(state: State<AppState>, query: String) -> SearchGroups {
    let entries = state.entries.lock().unwrap();
    let sorted = sorted_refs(&entries);
    search::group(&sorted, &query)
}

#[tauri::command]
pub async fn search_contents(paths: Vec<String>, keyword: String) -> Vec<String> {
    tauri::async_runtime::spawn_blocking(move || search::search_contents(&paths, &keyword))
        .await
        .unwrap_or_default()
}
