use std::collections::HashSet;
use std::sync::OnceLock;

use crate::exec::{entry_exec_dir, new_command};
use crate::models::{ScriptEntry, SearchGroups};

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextTool {
    Rg,
    Grep,
}

static TOOL: OnceLock<Option<TextTool>> = OnceLock::new();

fn probe(tool: &str) -> bool {
    new_command(tool)
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// 探测系统可用的文本搜索工具（只探测一次，优先 ripgrep）
fn detect_tool() -> Option<TextTool> {
    *TOOL.get_or_init(|| {
        if probe("rg") {
            Some(TextTool::Rg)
        } else if probe("grep") {
            Some(TextTool::Grep)
        } else {
            None
        }
    })
}

/// 名称或执行目录是否包含关键词（空关键词视为全部匹配）
pub fn matches_meta(entry: &ScriptEntry, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    entry.name.to_lowercase().contains(&q)
        || entry_exec_dir(entry)
            .unwrap_or_default()
            .to_lowercase()
            .contains(&q)
}

/// 按「文件名 → 目录 → 内容待查」分组，各组内保持传入的 MRU 顺序
pub fn group(entries: &[&ScriptEntry], query: &str) -> SearchGroups {
    let mut groups = SearchGroups::default();
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return groups;
    }
    for entry in entries {
        if entry.name.to_lowercase().contains(&q) {
            groups.by_name.push(entry.path.clone());
            continue;
        }
        let dir = entry_exec_dir(entry).unwrap_or_default().to_lowercase();
        if dir.contains(&q) {
            groups.by_dir.push(entry.path.clone());
        } else {
            groups.pending.push(entry.path.clone());
        }
    }
    groups
}

/// 在给定文件中搜索内容，返回命中文件路径。
/// 优先使用系统自带的 ripgrep / grep；都不可用时回退为逐个读取文件。
pub fn search_contents(paths: &[String], keyword: &str) -> Vec<String> {
    let mut matched = Vec::new();
    let kw = keyword.trim().to_lowercase();
    if paths.is_empty() || kw.is_empty() {
        return matched;
    }

    if let Some(tool) = detect_tool() {
        if let Ok(out) = run_tool(tool, paths, &kw) {
            let known: HashSet<&str> = paths.iter().map(|p| p.as_str()).collect();
            let output = String::from_utf8_lossy(&out.stdout);
            let mut hits = HashSet::new();
            for line in output.lines() {
                let path = line.trim();
                if !path.is_empty() && known.contains(path) {
                    hits.insert(path.to_string());
                }
            }
            // rg/grep 的多文件输出顺序不稳定，统一按传入顺序（即 MRU）返回
            return paths
                .iter()
                .filter(|path| hits.contains(path.as_str()))
                .cloned()
                .collect();
        }
    }

    for path in paths {
        if let Ok(text) = std::fs::read_to_string(path) {
            if text.to_lowercase().contains(&kw) {
                matched.push(path.clone());
            }
        }
    }
    matched
}

fn run_tool(
    tool: TextTool,
    paths: &[String],
    keyword: &str,
) -> std::io::Result<std::process::Output> {
    let mut cmd = new_command(match tool {
        TextTool::Rg => "rg",
        TextTool::Grep => "grep",
    });
    // -l 只输出命中文件名；-i 忽略大小写；-F 按字面量匹配；-- 隔离用户输入
    match tool {
        TextTool::Rg => {
            cmd.args(["-uu", "-l", "-i", "-F", "--", keyword]);
        }
        TextTool::Grep => {
            cmd.args(["-l", "-i", "-F", "-s", "--", keyword]);
        }
    }
    cmd.args(paths).output()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RunScheme;

    fn entry(path: &str) -> ScriptEntry {
        ScriptEntry {
            path: path.to_string(),
            name: crate::ops::file_name(path),
            lang: "plain".to_string(),
            last_used_at: 0,
            use_count: 0,
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
    fn group_prioritizes_name_then_dir_then_content() {
        let named = entry("/proj/alpha/run.py");
        let dired = entry("/proj/beta/report.md");
        let pending = entry("/proj/gamma/other.txt");
        let all = [&named, &dired, &pending];

        let by_name = group(&all, "run");
        assert_eq!(by_name.by_name, vec!["/proj/alpha/run.py"]);
        assert!(by_name.by_dir.is_empty());
        assert_eq!(
            by_name.pending,
            vec!["/proj/beta/report.md", "/proj/gamma/other.txt"]
        );

        let by_dir = group(&all, "beta");
        assert!(by_dir.by_name.is_empty());
        assert_eq!(by_dir.by_dir, vec!["/proj/beta/report.md"]);
        assert_eq!(
            by_dir.pending,
            vec!["/proj/alpha/run.py", "/proj/gamma/other.txt"]
        );

        let empty = group(&all, "  ");
        assert!(empty.by_name.is_empty() && empty.by_dir.is_empty() && empty.pending.is_empty());
    }

    #[test]
    fn matches_meta_checks_name_and_exec_dir() {
        let e = entry("/proj/alpha/run.py");
        assert!(matches_meta(&e, "run"));
        assert!(matches_meta(&e, "alpha"));
        assert!(matches_meta(&e, "  "));
        assert!(!matches_meta(&e, "beta"));
    }

    #[test]
    fn search_contents_preserves_input_order() {
        let dir = std::env::temp_dir().join(format!(
            "scriptbutler-search-{}",
            crate::ops::new_id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let paths: Vec<String> = ["a.txt", "b.txt", "c.txt"]
            .iter()
            .map(|name| {
                let path = dir.join(name);
                std::fs::write(&path, format!("needle {name}\n")).unwrap();
                path.to_string_lossy().to_string()
            })
            .collect();

        let input = vec![paths[2].clone(), paths[0].clone(), paths[1].clone()];
        assert_eq!(search_contents(&input, "needle"), input);
        assert!(search_contents(&input, "missing").is_empty());
        assert!(search_contents(&[], "needle").is_empty());

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
