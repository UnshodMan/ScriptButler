use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::models::ScriptEntry;

/// scripts.json 的磁盘结构，保持与 tauri-plugin-store 时期一致
#[derive(Serialize, Deserialize, Default)]
struct StoreFile {
    #[serde(default)]
    entries: Vec<ScriptEntry>,
}

pub struct AppState {
    pub entries: Mutex<Vec<ScriptEntry>>,
    pub store_path: PathBuf,
    pub managed_dir: PathBuf,
    pub children: Mutex<HashMap<String, std::process::Child>>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let store_path = app_data_dir.join("scripts.json");
        let managed_dir = app_data_dir.join("scripts");
        let entries = load_entries(&store_path);
        let _ = std::fs::create_dir_all(&managed_dir);
        Self {
            entries: Mutex::new(entries),
            store_path,
            managed_dir,
            children: Mutex::new(HashMap::new()),
        }
    }

    /// 写回 scripts.json；失败仅记录日志，不阻断界面操作
    pub fn save_entries(&self, entries: &[ScriptEntry]) {
        let file = StoreFile {
            entries: entries.to_vec(),
        };
        match serde_json::to_string_pretty(&file) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&self.store_path, text) {
                    eprintln!("failed to write {}: {e}", self.store_path.display());
                }
            }
            Err(e) => eprintln!("failed to serialize scripts: {e}"),
        }
    }
}

/// 读取条目；文件不存在或解析失败时视为空列表，并保留原文件不做覆盖
fn load_entries(path: &Path) -> Vec<ScriptEntry> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    match serde_json::from_str::<StoreFile>(&text) {
        Ok(file) => file.entries,
        Err(e) => {
            eprintln!("failed to parse {}: {e}", path.display());
            Vec::new()
        }
    }
}
