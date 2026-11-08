use std::process::Command;

use crate::models::{EntryView, ScriptEntry};

/// 取文件所在目录，兼容 `/` 与 `\`；路径无法解析时返回 None
pub fn dir_of(path: &str) -> Option<String> {
    let i = path.rfind(|c| c == '/' || c == '\\')?;
    if i > 0 {
        Some(path[..i].to_string())
    } else {
        None
    }
}

/// 按平台为路径加引号：POSIX 用单引号，Windows cmd 用双引号
fn quote_path(p: &str) -> String {
    if cfg!(windows) {
        format!("\"{}\"", p.replace('"', "\"\""))
    } else {
        format!("'{}'", p.replace('\'', "'\\''"))
    }
}

/// 切换到执行目录的前缀，例如 `cd 'dir' && `；Windows 用 `cd /d` 以支持跨盘符
fn cd_prefix(dir: Option<&str>) -> String {
    match dir {
        Some(d) if !d.is_empty() => {
            if cfg!(windows) {
                format!("cd /d {} && ", quote_path(d))
            } else {
                format!("cd {} && ", quote_path(d))
            }
        }
        _ => String::new(),
    }
}

/// `{file}` 占位符替换为引号包裹的路径；无占位符时在末尾追加
pub fn build_command(
    template: &str,
    file_path: &str,
    cd_to_script_dir: bool,
    exec_dir: Option<&str>,
) -> String {
    let quoted = quote_path(file_path);
    let trimmed = template.trim();
    let cmd = if trimmed.contains("{file}") {
        trimmed.replace("{file}", &quoted)
    } else {
        format!("{trimmed} {quoted}")
    };
    let fallback = dir_of(file_path);
    let prefix = if cd_to_script_dir {
        cd_prefix(exec_dir.or(fallback.as_deref()))
    } else {
        String::new()
    };
    format!("{prefix}{cmd}")
}

/// 解析条目的执行目录：托管脚本取可配置执行目录，普通脚本取文件所在目录
pub fn entry_exec_dir(entry: &ScriptEntry) -> Option<String> {
    if !entry.is_managed() {
        return dir_of(&entry.path);
    }
    entry
        .work_dir
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| dir_of(entry.source_path.as_deref().unwrap_or(&entry.path)))
}

/// 构造列表视图：附带执行目录与当前激活方案的实际命令
pub fn to_view(entry: &ScriptEntry) -> EntryView {
    let exec_dir = entry_exec_dir(entry);
    let effective_command = entry.active_scheme().map(|scheme| {
        build_command(
            &scheme.command,
            &entry.path,
            scheme.cd_to_script_dir,
            exec_dir.as_deref(),
        )
    });
    EntryView {
        entry: entry.clone(),
        exec_dir,
        effective_command,
    }
}

/// 创建子进程命令；Windows 下隐藏控制台窗口
pub fn new_command(program: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd
}
