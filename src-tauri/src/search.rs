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
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let path = line.trim();
                if !path.is_empty() && known.contains(path) {
                    matched.push(path.to_string());
                }
            }
            return matched;
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
