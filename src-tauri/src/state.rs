use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;

use scriptbutler_core::models::ScriptEntry;
use scriptbutler_core::store;

/// 文件签名：修改时间 + 长度，用于发现 CLI 等外部进程的写盘
type FileSig = Option<(SystemTime, u64)>;

pub struct AppState {
    pub entries: Mutex<Vec<ScriptEntry>>,
    pub store_path: PathBuf,
    pub managed_dir: PathBuf,
    pub children: Mutex<HashMap<String, std::process::Child>>,
    last_sig: Mutex<FileSig>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let store_path = app_data_dir.join("scripts.json");
        let managed_dir = scriptbutler_core::store::managed_dir(&app_data_dir);
        let entries = match store::try_load_entries(&store_path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("failed to load {}: {e}", store_path.display());
                Vec::new()
            }
        };
        let _ = std::fs::create_dir_all(&managed_dir);
        let last_sig = file_sig(&store_path);
        Self {
            entries: Mutex::new(entries),
            store_path,
            managed_dir,
            children: Mutex::new(HashMap::new()),
            last_sig: Mutex::new(last_sig),
        }
    }

    /// 写回 scripts.json；失败仅记录日志，不阻断界面操作
    pub fn save_entries(&self, entries: &[ScriptEntry]) {
        match store::save_entries(&self.store_path, entries) {
            Ok(()) => {
                *self.last_sig.lock().unwrap() = file_sig(&self.store_path);
            }
            Err(e) => eprintln!("{e}"),
        }
    }

    /// 获取条目锁；若文件被外部修改且能正常解析，先整体重载，避免覆盖 CLI 的写入
    pub fn lock_entries(&self) -> MutexGuard<'_, Vec<ScriptEntry>> {
        let mut guard = self.entries.lock().unwrap();
        self.reload_if_changed(&mut guard);
        guard
    }

    fn reload_if_changed(&self, entries: &mut Vec<ScriptEntry>) {
        let sig = file_sig(&self.store_path);
        let mut last_sig = self.last_sig.lock().unwrap();
        if *last_sig == sig {
            return;
        }
        match store::try_load_entries(&self.store_path) {
            Ok(next) => {
                *entries = next;
                *last_sig = sig;
            }
            Err(e) => {
                // 保留内存数据，不覆盖可能损坏的文件
                eprintln!("failed to reload {}: {e}", self.store_path.display());
            }
        }
    }
}

fn file_sig(path: &Path) -> FileSig {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptbutler_core::models::RunScheme;
    use std::time::Duration;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "scriptbutler-state-{}",
            scriptbutler_core::ops::new_id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn entry(path: &str, name: &str) -> ScriptEntry {
        ScriptEntry {
            path: path.to_string(),
            name: name.to_string(),
            lang: "plain".to_string(),
            last_used_at: 1,
            use_count: 1,
            schemes: vec![RunScheme {
                id: "id".to_string(),
                name: "Default".to_string(),
                command: "{file}".to_string(),
                cd_to_script_dir: true,
            }],
            active_scheme_id: Some("id".to_string()),
            managed: None,
            source_path: None,
            work_dir: None,
        }
    }

    #[test]
    fn reloads_after_external_write() {
        let dir = temp_dir();
        let path = dir.join("scripts.json");
        store::save_entries(&path, &[entry("/tmp/a.sh", "a.sh")]).unwrap();
        let state = AppState::new(dir.clone());
        assert_eq!(state.lock_entries().len(), 1);

        // 模拟 CLI 进程写盘
        std::thread::sleep(Duration::from_millis(10));
        store::save_entries(&path, &[entry("/tmp/b.sh", "b.sh"), entry("/tmp/c.sh", "c.sh")])
            .unwrap();

        let entries = state.lock_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "b.sh");

        drop(entries);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn keeps_memory_when_external_file_is_corrupt() {
        let dir = temp_dir();
        let path = dir.join("scripts.json");
        store::save_entries(&path, &[entry("/tmp/a.sh", "a.sh")]).unwrap();
        let state = AppState::new(dir.clone());

        std::thread::sleep(Duration::from_millis(10));
        std::fs::write(&path, "{oops").unwrap();

        let entries = state.lock_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "a.sh");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{oops");

        drop(entries);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
