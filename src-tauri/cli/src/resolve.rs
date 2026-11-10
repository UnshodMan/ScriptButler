use std::path::PathBuf;

use scriptbutler_core::models::{RunScheme, ScriptEntry};

use crate::output::CliError;

pub fn absolute(path: &str) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| PathBuf::from(path))
}

/// 按「绝对路径 → 托管源路径 → 名称（忽略大小写）」解析条目
pub fn resolve_index(entries: &[ScriptEntry], reference: &str) -> Result<usize, CliError> {
    let abs = absolute(reference).to_string_lossy().to_string();
    if let Some(i) = entries
        .iter()
        .position(|e| e.path == reference || e.path == abs)
    {
        return Ok(i);
    }
    if let Some(i) = entries
        .iter()
        .position(|e| e.is_managed() && e.source_path.as_deref() == Some(reference))
    {
        return Ok(i);
    }
    if let Some(i) = entries
        .iter()
        .position(|e| e.is_managed() && e.source_path.as_deref() == Some(abs.as_str()))
    {
        return Ok(i);
    }

    let lower = reference.to_lowercase();
    let matches: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.name.to_lowercase() == lower)
        .map(|(i, _)| i)
        .collect();
    match matches.len() {
        0 => Err(CliError::NotFound(format!(
            "script '{reference}' was not found"
        ))),
        1 => Ok(matches[0]),
        _ => {
            let paths = matches
                .iter()
                .map(|i| entries[*i].path.clone())
                .collect::<Vec<_>>();
            Err(CliError::Ambiguous(paths))
        }
    }
}

/// 方案解析：先按 id，再按名称（忽略大小写）；缺省取激活方案
pub fn resolve_scheme<'a>(
    entry: &'a ScriptEntry,
    reference: Option<&str>,
) -> Result<&'a RunScheme, CliError> {
    let Some(reference) = reference else {
        return entry.active_scheme().ok_or_else(|| {
            CliError::NotFound(format!("script '{}' has no run scheme", entry.name))
        });
    };

    if let Some(scheme) = entry.schemes.iter().find(|s| s.id == reference) {
        return Ok(scheme);
    }

    let lower = reference.to_lowercase();
    let matches: Vec<&RunScheme> = entry
        .schemes
        .iter()
        .filter(|s| s.name.to_lowercase() == lower)
        .collect();
    match matches.len() {
        0 => Err(CliError::NotFound(format!(
            "scheme '{reference}' was not found for '{}'",
            entry.name
        ))),
        1 => Ok(matches[0]),
        _ => {
            let names = matches
                .iter()
                .map(|s| format!("{} ({})", s.name, s.id))
                .collect::<Vec<_>>();
            Err(CliError::Ambiguous(names))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptbutler_core::models::{RunScheme, ScriptEntry};

    fn entry(
        path: &str,
        name: &str,
        managed: bool,
        source: Option<&str>,
        schemes: Vec<RunScheme>,
    ) -> ScriptEntry {
        ScriptEntry {
            path: path.to_string(),
            name: name.to_string(),
            lang: "plain".to_string(),
            last_used_at: 0,
            use_count: 0,
            schemes,
            active_scheme_id: None,
            managed: managed.then_some(true),
            source_path: source.map(str::to_string),
            work_dir: None,
        }
    }

    fn scheme(id: &str, name: &str) -> RunScheme {
        RunScheme {
            id: id.to_string(),
            name: name.to_string(),
            command: "{file}".to_string(),
            cd_to_script_dir: true,
        }
    }

    #[test]
    fn resolves_by_path_then_managed_source_then_name() {
        let entries = vec![
            entry("/data/1.py", "a.py", true, Some("/proj/a.py"), Vec::new()),
            entry("/data/2.py", "b.py", false, None, Vec::new()),
        ];

        assert_eq!(resolve_index(&entries, "/data/2.py").unwrap(), 1);
        assert_eq!(resolve_index(&entries, "/proj/a.py").unwrap(), 0);
        assert_eq!(resolve_index(&entries, "B.PY").unwrap(), 1);
        assert!(matches!(
            resolve_index(&entries, "missing"),
            Err(CliError::NotFound(_))
        ));
    }

    #[test]
    fn duplicate_names_are_ambiguous() {
        let entries = vec![
            entry("/a/x.sh", "x.sh", false, None, Vec::new()),
            entry("/b/x.sh", "x.sh", false, None, Vec::new()),
        ];
        assert!(matches!(
            resolve_index(&entries, "x.sh"),
            Err(CliError::Ambiguous(paths)) if paths.len() == 2
        ));
    }

    #[test]
    fn resolves_scheme_by_id_then_name() {
        let mut e = entry(
            "/data/a.py",
            "a.py",
            false,
            None,
            vec![scheme("s1", "Default"), scheme("s2", "Fast")],
        );
        e.active_scheme_id = Some("s1".to_string());

        assert_eq!(resolve_scheme(&e, None).unwrap().id, "s1");
        assert_eq!(resolve_scheme(&e, Some("s1")).unwrap().id, "s1");
        assert_eq!(resolve_scheme(&e, Some("fast")).unwrap().id, "s2");
        assert!(matches!(
            resolve_scheme(&e, Some("missing")),
            Err(CliError::NotFound(_))
        ));
    }
}
