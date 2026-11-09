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
