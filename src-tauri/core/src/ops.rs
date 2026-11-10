use std::fmt;
use std::path::{Path, PathBuf};

use crate::exec::{dir_of, to_view};
use crate::language;
use crate::models::{EntryView, RunScheme, ScriptEntry};
use crate::unix_ms;

#[derive(Debug)]
pub enum CoreError {
    NotFound,
    NotFile,
    InvalidData(String),
    Io(String),
    ManageFailed(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::NotFound => write!(f, "not found"),
            CoreError::NotFile => write!(f, "not a file"),
            CoreError::InvalidData(detail) => write!(f, "invalid data: {detail}"),
            CoreError::Io(detail) => write!(f, "io error: {detail}"),
            CoreError::ManageFailed(detail) => write!(f, "manage failed: {detail}"),
        }
    }
}

impl std::error::Error for CoreError {}

pub struct AddOutcome {
    pub view: EntryView,
    pub created: bool,
}

pub struct ManageOptions<'a> {
    pub scheme_name: &'a str,
    pub name: Option<&'a str>,
    pub command: Option<&'a str>,
    pub work_dir: Option<&'a str>,
    pub dedupe: bool,
}

pub struct ManageOutcome {
    pub view: EntryView,
    pub created: bool,
    pub copied_to: String,
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn file_name(path: &str) -> String {
    path.rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(path)
        .to_string()
}

/// 读取文件元数据；只有真正的「不存在」才映射为 NotFound，其余 I/O 错误保留细节
fn file_metadata(path: &str) -> Result<std::fs::Metadata, CoreError> {
    std::fs::metadata(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => CoreError::NotFound,
        _ => CoreError::Io(e.to_string()),
    })
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

/// 目标文件已存在时追加 `-1`、`-2`，避免同一毫秒内重复托管互相覆盖
fn unique_target(managed_dir: &Path, source_path: &str) -> PathBuf {
    let base = stamp_file_name(source_path);
    let mut target = managed_dir.join(&base);
    if !target.exists() {
        return target;
    }
    let dot = base.rfind('.').filter(|i| *i > 0).unwrap_or(base.len());
    let (stem, ext) = base.split_at(dot);
    let mut i = 1;
    loop {
        target = managed_dir.join(format!("{stem}-{i}{ext}"));
        if !target.exists() {
            return target;
        }
        i += 1;
    }
}

fn make_scheme(name: &str, command: &str) -> RunScheme {
    RunScheme {
        id: new_id(),
        name: name.to_string(),
        command: command.to_string(),
        cd_to_script_dir: true,
    }
}

/// MRU 排序：最近使用 → 使用次数 → 名称
pub fn sorted_refs(entries: &[ScriptEntry]) -> Vec<&ScriptEntry> {
    let mut list: Vec<&ScriptEntry> = entries.iter().collect();
    list.sort_by(|a, b| {
        b.last_used_at
            .cmp(&a.last_used_at)
            .then_with(|| b.use_count.cmp(&a.use_count))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    list
}

pub fn sorted_views(entries: &[ScriptEntry]) -> Vec<EntryView> {
    sorted_refs(entries).into_iter().map(to_view).collect()
}

/// 原地登记：同路径已存在则只更新 MRU，否则按扩展名生成默认方案
pub fn add_entry(
    entries: &mut Vec<ScriptEntry>,
    path: &str,
    scheme_name: &str,
) -> Result<AddOutcome, CoreError> {
    let meta = file_metadata(path)?;
    if !meta.is_file() {
        return Err(CoreError::NotFile);
    }

    if let Some(existing) = entries.iter_mut().find(|e| e.path == path) {
        existing.last_used_at = unix_ms();
        existing.use_count += 1;
        return Ok(AddOutcome {
            view: to_view(existing),
            created: false,
        });
    }

    let lang = language::detect(path);
    let scheme = make_scheme(scheme_name, lang.default_command);
    let entry = ScriptEntry {
        path: path.to_string(),
        name: file_name(path),
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
    let view = to_view(entries.last().unwrap());
    Ok(AddOutcome {
        view,
        created: true,
    })
}

/// 托管：把脚本复制到应用数据目录下的 scripts/，以时间戳命名。
/// 已登记条目继承名称与全部运行方案；未登记条目按语言生成默认方案。
pub fn manage_script(
    entries: &mut Vec<ScriptEntry>,
    source_path: &str,
    managed_dir: &Path,
    opts: &ManageOptions,
) -> Result<ManageOutcome, CoreError> {
    let meta = file_metadata(source_path)?;
    if !meta.is_file() {
        return Err(CoreError::NotFile);
    }

    if opts.dedupe {
        if let Some(existing) = entries.iter().find(|e| {
            e.is_managed()
                && (e.source_path.as_deref() == Some(source_path) || e.path == source_path)
        }) {
            return Ok(ManageOutcome {
                view: to_view(existing),
                created: false,
                copied_to: existing.path.clone(),
            });
        }
    }

    std::fs::create_dir_all(managed_dir).map_err(|e| CoreError::ManageFailed(e.to_string()))?;
    let target = unique_target(managed_dir, source_path);
    std::fs::copy(source_path, &target).map_err(|e| CoreError::ManageFailed(e.to_string()))?;

    let existing = entries.iter().find(|e| e.path == source_path).cloned();
    let name = opts
        .name
        .map(str::to_string)
        .or_else(|| existing.as_ref().map(|e| e.name.clone()))
        .unwrap_or_else(|| file_name(source_path));
    let lang = existing
        .as_ref()
        .map(|e| e.lang.clone())
        .unwrap_or_else(|| language::detect(source_path).id.to_string());

    let (schemes, active_scheme_id) = match &existing {
        Some(e) if !e.schemes.is_empty() => (e.schemes.clone(), e.active_scheme_id.clone()),
        _ => {
            let command = opts
                .command
                .map(str::to_string)
                .unwrap_or_else(|| language::detect(source_path).default_command.to_string());
            let scheme = make_scheme(opts.scheme_name, &command);
            let id = Some(scheme.id.clone());
            (vec![scheme], id)
        }
    };

    let entry = ScriptEntry {
        path: target.to_string_lossy().to_string(),
        name,
        lang,
        last_used_at: unix_ms(),
        use_count: 0,
        schemes,
        active_scheme_id,
        managed: Some(true),
        source_path: Some(source_path.to_string()),
        work_dir: opts
            .work_dir
            .map(str::to_string)
            .or_else(|| dir_of(source_path)),
    };

    entries.push(entry);
    let view = to_view(entries.last().unwrap());
    Ok(ManageOutcome {
        view,
        created: true,
        copied_to: target.to_string_lossy().to_string(),
    })
}

pub fn remove_entry(entries: &mut Vec<ScriptEntry>, path: &str) -> bool {
    let before = entries.len();
    entries.retain(|e| e.path != path);
    entries.len() != before
}

pub fn mark_used(entries: &mut [ScriptEntry], path: &str) -> bool {
    if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        entry.last_used_at = unix_ms();
        entry.use_count += 1;
        true
    } else {
        false
    }
}

pub fn set_work_dir(entries: &mut [ScriptEntry], path: &str, work_dir: &str) -> bool {
    if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        entry.work_dir = Some(work_dir.to_string());
        true
    } else {
        false
    }
}

/// 首次打开且无方案时，按语言生成默认方案；已有方案时不做修改
pub fn ensure_default_scheme(
    entries: &mut [ScriptEntry],
    path: &str,
    name: &str,
) -> Option<EntryView> {
    let entry = entries.iter_mut().find(|e| e.path == path)?;
    if !entry.schemes.is_empty() {
        return None;
    }
    let scheme = make_scheme(name, language::detect(&entry.path).default_command);
    entry.active_scheme_id = Some(scheme.id.clone());
    entry.schemes.push(scheme);
    Some(to_view(entry))
}

/// 新建方案并设为激活方案
pub fn create_scheme(
    entries: &mut [ScriptEntry],
    path: &str,
    name: &str,
    command: &str,
    cd_to_script_dir: bool,
) -> Option<EntryView> {
    let entry = entries.iter_mut().find(|e| e.path == path)?;
    let scheme = RunScheme {
        id: new_id(),
        name: name.to_string(),
        command: command.to_string(),
        cd_to_script_dir,
    };
    entry.active_scheme_id = Some(scheme.id.clone());
    entry.schemes.push(scheme);
    Some(to_view(entry))
}

/// 修改指定方案并设为激活方案；方案不存在时不做修改
pub fn update_scheme(
    entries: &mut [ScriptEntry],
    path: &str,
    id: &str,
    name: &str,
    command: &str,
    cd_to_script_dir: bool,
) -> Option<EntryView> {
    let entry = entries.iter_mut().find(|e| e.path == path)?;
    {
        let scheme = entry.schemes.iter_mut().find(|s| s.id == id)?;
        scheme.name = name.to_string();
        scheme.command = command.to_string();
        scheme.cd_to_script_dir = cd_to_script_dir;
    }
    entry.active_scheme_id = Some(id.to_string());
    Some(to_view(entry))
}

/// 删除指定方案；若删除的是激活方案，则回退到剩余首个方案
pub fn delete_scheme(entries: &mut [ScriptEntry], path: &str, id: &str) -> Option<EntryView> {
    let entry = entries.iter_mut().find(|e| e.path == path)?;
    let before = entry.schemes.len();
    entry.schemes.retain(|s| s.id != id);
    if entry.schemes.len() == before {
        return None;
    }
    let active_valid = entry
        .active_scheme_id
        .as_ref()
        .map(|active| entry.schemes.iter().any(|s| &s.id == active))
        .unwrap_or(false);
    if !active_valid {
        entry.active_scheme_id = entry.schemes.first().map(|s| s.id.clone());
    }
    Some(to_view(entry))
}

/// 切换激活方案；方案不存在时不做修改
pub fn set_active_scheme(entries: &mut [ScriptEntry], path: &str, id: &str) -> Option<EntryView> {
    let entry = entries.iter_mut().find(|e| e.path == path)?;
    if !entry.schemes.iter().any(|s| s.id == id) {
        return None;
    }
    entry.active_scheme_id = Some(id.to_string());
    Some(to_view(entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("scriptbutler-test-{}", new_id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn add_is_idempotent_and_updates_mru() {
        let dir = temp_dir();
        let file = dir.join("demo.py");
        fs::write(&file, "print('hi')\n").unwrap();
        let path = file.to_string_lossy().to_string();

        let mut entries = Vec::new();
        let first = add_entry(&mut entries, &path, "Default").unwrap();
        assert!(first.created);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].schemes.len(), 1);
        assert_eq!(entries[0].lang, "python");

        let second = add_entry(&mut entries, &path, "Default").unwrap();
        assert!(!second.created);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].use_count, 2);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn manage_copies_inherits_and_dedupes() {
        let dir = temp_dir();
        let file = dir.join("demo.py");
        fs::write(&file, "print('hi')\n").unwrap();
        let path = file.to_string_lossy().to_string();
        let managed = dir.join("managed");

        let mut entries = Vec::new();
        add_entry(&mut entries, &path, "Default").unwrap();
        let opts = ManageOptions {
            scheme_name: "Default",
            name: None,
            command: None,
            work_dir: None,
            dedupe: true,
        };
        let outcome = manage_script(&mut entries, &path, &managed, &opts).unwrap();
        assert!(outcome.created);
        assert_eq!(entries.len(), 2);
        assert!(PathBuf::from(&outcome.copied_to).exists());
        assert!(entries[1].is_managed());
        assert_eq!(entries[1].source_path.as_deref(), Some(path.as_str()));
        assert_eq!(entries[1].schemes.len(), 1);

        let again = manage_script(&mut entries, &path, &managed, &opts).unwrap();
        assert!(!again.created);
        assert_eq!(entries.len(), 2);
        assert_eq!(again.copied_to, outcome.copied_to);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn manage_unregistered_file_creates_default_scheme() {
        let dir = temp_dir();
        let file = dir.join("demo.js");
        fs::write(&file, "console.log('hi')\n").unwrap();
        let path = file.to_string_lossy().to_string();
        let managed = dir.join("managed");

        let mut entries = Vec::new();
        let opts = ManageOptions {
            scheme_name: "Default",
            name: None,
            command: Some("node {file}"),
            work_dir: None,
            dedupe: true,
        };
        let outcome = manage_script(&mut entries, &path, &managed, &opts).unwrap();
        assert!(outcome.created);
        assert_eq!(entries[0].name, "demo.js");
        assert_eq!(entries[0].lang, "javascript");
        assert_eq!(entries[0].schemes[0].command, "node {file}");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn remove_only_drops_entry() {
        let dir = temp_dir();
        let file = dir.join("demo.sh");
        fs::write(&file, "echo hi\n").unwrap();
        let path = file.to_string_lossy().to_string();

        let mut entries = Vec::new();
        add_entry(&mut entries, &path, "Default").unwrap();
        assert!(remove_entry(&mut entries, &path));
        assert!(entries.is_empty());
        assert!(file.exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_file_and_directory_are_distinguished() {
        let dir = temp_dir();
        let missing = dir.join("missing.py");
        let mut entries = Vec::new();
        assert!(matches!(
            add_entry(&mut entries, &missing.to_string_lossy(), "Default"),
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            add_entry(&mut entries, &dir.to_string_lossy(), "Default"),
            Err(CoreError::NotFile)
        ));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scheme_lifecycle_and_default_fallback() {
        let dir = temp_dir();
        let file = dir.join("demo.py");
        fs::write(&file, "print('hi')\n").unwrap();
        let path = file.to_string_lossy().to_string();

        let mut entries = Vec::new();
        add_entry(&mut entries, &path, "Default").unwrap();
        entries[0].schemes.clear();
        entries[0].active_scheme_id = None;

        // 首次打开生成默认方案，再次调用不产生修改
        let view = ensure_default_scheme(&mut entries, &path, "默认方案").unwrap();
        assert_eq!(view.entry.schemes[0].command, "python3 {file}");
        assert!(ensure_default_scheme(&mut entries, &path, "默认方案").is_none());

        // 新建后自动激活
        let created = create_scheme(&mut entries, &path, "Node", "node {file}", false).unwrap();
        let created_id = created.entry.active_scheme_id.unwrap();
        assert_eq!(entries[0].schemes.len(), 2);
        assert_eq!(entries[0].active_scheme_id.as_deref(), Some(created_id.as_str()));

        // 修改并重新激活
        let updated = update_scheme(
            &mut entries,
            &path,
            &created_id,
            "Node2",
            "node --version {file}",
            true,
        )
        .unwrap();
        let scheme = updated
            .entry
            .schemes
            .iter()
            .find(|s| s.id == created_id)
            .unwrap();
        assert_eq!(scheme.name, "Node2");
        assert_eq!(scheme.command, "node --version {file}");
        assert!(scheme.cd_to_script_dir);
        assert!(update_scheme(&mut entries, &path, "missing", "X", "x", true).is_none());

        // 删除激活方案后回退到首个剩余方案
        assert!(delete_scheme(&mut entries, &path, &created_id).is_some());
        assert_eq!(entries[0].schemes.len(), 1);
        assert_eq!(
            entries[0].active_scheme_id.as_deref(),
            Some(entries[0].schemes[0].id.as_str())
        );
        assert!(delete_scheme(&mut entries, &path, &created_id).is_none());

        // 只接受已存在的方案
        assert!(set_active_scheme(&mut entries, &path, "missing").is_none());
        let remaining_id = entries[0].schemes[0].id.clone();
        assert!(set_active_scheme(&mut entries, &path, &remaining_id).is_some());
        assert_eq!(entries[0].active_scheme_id.as_deref(), Some(remaining_id.as_str()));

        fs::remove_dir_all(&dir).unwrap();
    }
}
