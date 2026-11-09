use std::fmt;

use serde_json::json;
use scriptbutler_core::models::EntryView;

#[derive(Debug)]
pub enum CliError {
    NotFound(String),
    Ambiguous(Vec<String>),
    NotFile(String),
    InvalidData(String),
    Io(String),
    SpawnFailed(String),
    ManageFailed(String),
}

impl CliError {
    pub fn code(&self) -> &'static str {
        match self {
            CliError::NotFound(_) => "not_found",
            CliError::Ambiguous(_) => "ambiguous",
            CliError::NotFile(_) => "not_file",
            CliError::InvalidData(_) => "invalid_data",
            CliError::Io(_) => "io_error",
            CliError::SpawnFailed(_) => "spawn_failed",
            CliError::ManageFailed(_) => "manage_failed",
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::NotFound(_) | CliError::Ambiguous(_) | CliError::NotFile(_) => 2,
            CliError::InvalidData(_) => 3,
            _ => 1,
        }
    }

    pub fn message(&self) -> String {
        match self {
            CliError::NotFound(detail) => detail.clone(),
            CliError::Ambiguous(paths) => {
                format!("ambiguous reference; candidates:\n  {}", paths.join("\n  "))
            }
            CliError::NotFile(detail) => detail.clone(),
            CliError::InvalidData(detail) => format!("invalid scripts.json: {detail}"),
            CliError::Io(detail) => detail.clone(),
            CliError::SpawnFailed(detail) => format!("failed to start script: {detail}"),
            CliError::ManageFailed(detail) => format!("failed to manage script: {detail}"),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for CliError {}

pub fn print_error(err: &CliError, json: bool) {
    if json {
        print_json(json!({
            "ok": false,
            "error": { "code": err.code(), "message": err.message() }
        }));
    } else {
        eprintln!("error: {}", err.message());
    }
}

pub fn print_json(value: serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
}

pub fn print_entry(view: &EntryView) {
    let entry = &view.entry;
    let kind = if entry.is_managed() { "managed" } else { "linked" };
    println!("{}\t{}\t{}\t{}", entry.name, entry.lang, kind, entry.path);
    if let Some(dir) = &view.exec_dir {
        println!("  dir: {dir}");
    }
    if let Some(command) = &view.effective_command {
        println!("  command: {command}");
    }
    let scheme = entry.active_scheme().map(|s| s.name.as_str()).unwrap_or("-");
    println!(
        "  scheme: {scheme}\tuses: {}\tlast_used_at: {}",
        entry.use_count, entry.last_used_at
    );
}
