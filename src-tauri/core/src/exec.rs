use std::process::Command;

use crate::models::{EntryView, RunScheme, ScriptEntry};

/// 取文件所在目录，兼容 `/` 与 `\`；路径无法解析时返回 None
pub fn dir_of(path: &str) -> Option<String> {
    let i = path.rfind(|c| c == '/' || c == '\\')?;
    if i > 0 {
        Some(path[..i].to_string())
    } else {
        None
    }
}

/// 按平台为命令行参数加引号：POSIX 用单引号，Windows cmd 用双引号
pub fn quote_arg(p: &str) -> String {
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
                format!("cd /d {} && ", quote_arg(d))
            } else {
                format!("cd {} && ", quote_arg(d))
            }
        }
        _ => String::new(),
    }
}

/// `{file}` 占位符替换为引号包裹的路径；无占位符时在末尾追加。
/// `extra_args` 为运行时的临时参数（已按需引用），非空时追加到命令末尾。
pub fn build_command(
    template: &str,
    file_path: &str,
    cd_to_script_dir: bool,
    exec_dir: Option<&str>,
    extra_args: Option<&str>,
) -> String {
    let quoted = quote_arg(file_path);
    let trimmed = template.trim();
    let mut cmd = if trimmed.contains("{file}") {
        trimmed.replace("{file}", &quoted)
    } else {
        format!("{trimmed} {quoted}")
    };
    if let Some(args) = extra_args.map(str::trim).filter(|a| !a.is_empty()) {
        cmd.push(' ');
        cmd.push_str(args);
    }
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
            None,
        )
    });
    EntryView {
        entry: entry.clone(),
        exec_dir,
        effective_command,
    }
}

/// 按指定方案拼接实际运行的完整命令；`extra_args` 为运行时临时参数
pub fn build_run_command(
    entry: &ScriptEntry,
    scheme: &RunScheme,
    extra_args: Option<&str>,
) -> String {
    let exec_dir = entry_exec_dir(entry);
    build_command(
        &scheme.command,
        &entry.path,
        scheme.cd_to_script_dir,
        exec_dir.as_deref(),
        extra_args,
    )
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

/// 用平台 shell 执行命令行：Unix `sh -c`，Windows `cmd /S /C`
pub fn shell_command(cmdline: &str) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut c = new_command("cmd");
        // cmd.exe 不按 CRT 规则解析引号，必须原样传入命令行；
        // 用 /S 配合外层引号，保证内部路径引号不被破坏
        c.arg("/S").arg("/C").raw_arg(format!("\"{cmdline}\""));
        c
    }
    #[cfg(not(windows))]
    {
        let mut c = new_command("sh");
        c.arg("-c").arg(cmdline);
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ScriptEntry;

    fn entry(path: &str) -> ScriptEntry {
        ScriptEntry {
            path: path.to_string(),
            name: "demo".to_string(),
            lang: "plain".to_string(),
            last_used_at: 0,
            use_count: 0,
            schemes: Vec::new(),
            active_scheme_id: None,
            managed: None,
            source_path: None,
            work_dir: None,
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn build_command_substitutes_placeholder_and_appends_path() {
        assert_eq!(
            build_command("python3 {file}", "/tmp/a b.py", false, None, None),
            "python3 '/tmp/a b.py'"
        );
        assert_eq!(
            build_command("python3", "/tmp/a.py", false, None, None),
            "python3 '/tmp/a.py'"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn build_command_appends_extra_args_at_the_end() {
        assert_eq!(
            build_command("python3 {file} --verbose", "/tmp/a.py", false, None, Some("--x 1")),
            "python3 '/tmp/a.py' --verbose --x 1"
        );
        assert_eq!(
            build_command("python3", "/tmp/a.py", false, None, Some("  ")),
            "python3 '/tmp/a.py'"
        );
        assert_eq!(
            build_command("python3", "/tmp/a.py", false, None, Some("'a b' c")),
            "python3 '/tmp/a.py' 'a b' c"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn build_command_prepends_cd_when_enabled() {
        assert_eq!(
            build_command("{file}", "/tmp/dir/a.py", true, None, None),
            "cd '/tmp/dir' && '/tmp/dir/a.py'"
        );
        assert_eq!(
            build_command("{file}", "/tmp/a.py", true, Some("/custom"), None),
            "cd '/custom' && '/tmp/a.py'"
        );
        assert_eq!(
            build_command("{file}", "/tmp/a.py", true, Some("/custom"), Some("--x")),
            "cd '/custom' && '/tmp/a.py' --x"
        );
    }

    #[test]
    fn entry_exec_dir_uses_work_dir_for_managed_entries() {
        let mut e = entry("/data/scripts/123.py");
        e.managed = Some(true);
        e.source_path = Some("/proj/a.py".to_string());
        e.work_dir = Some("/proj".to_string());
        assert_eq!(entry_exec_dir(&e).as_deref(), Some("/proj"));

        e.work_dir = None;
        assert_eq!(entry_exec_dir(&e).as_deref(), Some("/proj"));

        let linked = entry("/proj/b.py");
        assert_eq!(entry_exec_dir(&linked).as_deref(), Some("/proj"));
    }
}
