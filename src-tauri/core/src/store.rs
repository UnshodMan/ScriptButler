use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::models::ScriptEntry;

const APP_ID: &str = "com.scriptbutler.app";

/// scripts.json 的磁盘结构，保持与 tauri-plugin-store 时期一致
#[derive(Serialize, Deserialize, Default)]
pub struct StoreFile {
    #[serde(default)]
    pub entries: Vec<ScriptEntry>,
}

#[derive(Debug)]
pub enum StoreError {
    Io(String),
    InvalidData(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(detail) => write!(f, "io error: {detail}"),
            StoreError::InvalidData(detail) => write!(f, "invalid data: {detail}"),
        }
    }
}

impl std::error::Error for StoreError {}

/// 读取条目；文件不存在视为空列表，读取或解析失败返回错误（调用方可决定是否覆盖）
pub fn try_load_entries(path: &Path) -> Result<Vec<ScriptEntry>, StoreError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(StoreError::Io(e.to_string())),
    };
    match serde_json::from_str::<StoreFile>(&text) {
        Ok(file) => Ok(file.entries),
        Err(e) => Err(StoreError::InvalidData(e.to_string())),
    }
}

/// 写回 scripts.json（人类可读），并确保父目录存在
pub fn save_entries(path: &Path, entries: &[ScriptEntry]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let file = StoreFile {
        entries: entries.to_vec(),
    };
    let text = serde_json::to_string_pretty(&file)
        .map_err(|e| format!("failed to serialize scripts: {e}"))?;
    std::fs::write(path, text).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

/// 应用数据目录；CLI 侧解析，GUI 侧仍以 Tauri appDataDir 为准
pub fn data_dir() -> Result<PathBuf, String> {
    if let Some(dir) = std::env::var_os("SCRIPTBUTLER_DATA_DIR") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("APPDATA")
            .ok_or_else(|| "APPDATA is not set; set SCRIPTBUTLER_DATA_DIR".to_string())?;
        return Ok(PathBuf::from(base).join(APP_ID));
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| "HOME is not set; set SCRIPTBUTLER_DATA_DIR".to_string())?;
        return Ok(PathBuf::from(home)
            .join("Library/Application Support")
            .join(APP_ID));
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(base) = std::env::var_os("XDG_DATA_HOME") {
            if !base.is_empty() {
                return Ok(PathBuf::from(base).join(APP_ID));
            }
        }
        let home = std::env::var_os("HOME")
            .ok_or_else(|| "HOME is not set; set SCRIPTBUTLER_DATA_DIR".to_string())?;
        return Ok(PathBuf::from(home).join(".local/share").join(APP_ID));
    }

    #[allow(unreachable_code)]
    Err("unsupported platform; set SCRIPTBUTLER_DATA_DIR".to_string())
}

/// 托管脚本目录
pub fn managed_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("scripts")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ScriptEntry;

    fn entry(path: &str) -> ScriptEntry {
        ScriptEntry {
            path: path.to_string(),
            name: path.to_string(),
            lang: "plain".to_string(),
            last_used_at: 1,
            use_count: 1,
            schemes: Vec::new(),
            active_scheme_id: None,
            managed: None,
            source_path: None,
            work_dir: None,
        }
    }

    fn temp_file() -> PathBuf {
        std::env::temp_dir()
            .join(format!("scriptbutler-store-{}", crate::ops::new_id()))
            .join("scripts.json")
    }

    #[test]
    fn save_and_load_roundtrip() {
        let path = temp_file();
        save_entries(&path, &[entry("/tmp/a.sh")]).unwrap();
        let loaded = try_load_entries(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].path, "/tmp/a.sh");

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn missing_file_is_an_empty_list() {
        let path = temp_file();
        assert!(try_load_entries(&path).unwrap().is_empty());
    }

    #[test]
    fn invalid_json_is_reported() {
        let path = temp_file();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "{oops").unwrap();
        assert!(matches!(
            try_load_entries(&path),
            Err(StoreError::InvalidData(_))
        ));
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
