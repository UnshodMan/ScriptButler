#[derive(Clone, Copy)]
pub struct LangInfo {
    pub id: &'static str,
    pub default_command: &'static str,
}

const PLAIN: LangInfo = LangInfo {
    id: "plain",
    default_command: "{file}",
};

/// 按扩展名识别语言，并给出首次添加时使用的默认运行命令
pub fn detect(path: &str) -> LangInfo {
    let dot = match path.rfind('.') {
        Some(d) => d,
        None => return PLAIN,
    };
    let slash = path
        .rfind(|c| c == '/' || c == '\\')
        .map(|s| s as isize)
        .unwrap_or(-1);
    if dot as isize <= slash || dot == path.len() - 1 {
        return PLAIN;
    }
    let ext = path[dot + 1..].to_ascii_lowercase();
    match ext.as_str() {
        "py" => LangInfo {
            id: "python",
            default_command: "python3 {file}",
        },
        "js" | "mjs" | "cjs" => LangInfo {
            id: "javascript",
            default_command: "node {file}",
        },
        "ts" | "mts" | "tsx" => LangInfo {
            id: "typescript",
            default_command: "npx tsx {file}",
        },
        "sh" | "bash" | "zsh" => LangInfo {
            id: "shell",
            default_command: "bash {file}",
        },
        "json" => LangInfo {
            id: "json",
            default_command: "cat {file}",
        },
        "yaml" | "yml" => LangInfo {
            id: "yaml",
            default_command: "cat {file}",
        },
        "md" => LangInfo {
            id: "markdown",
            default_command: "cat {file}",
        },
        "html" | "htm" => LangInfo {
            id: "html",
            default_command: "open {file}",
        },
        "css" => LangInfo {
            id: "css",
            default_command: "open {file}",
        },
        _ => PLAIN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_and_default_command_by_extension() {
        assert_eq!(detect("a.py").id, "python");
        assert_eq!(detect("a.PY").default_command, "python3 {file}");
        assert_eq!(detect("a.json").default_command, "cat {file}");
        assert_eq!(detect("a.md").id, "markdown");
        assert_eq!(detect("a.sh").id, "shell");
    }

    #[test]
    fn falls_back_to_plain_without_a_valid_extension() {
        assert_eq!(detect("Makefile").id, "plain");
        assert_eq!(detect("dir.with.dot/file").id, "plain");
        assert_eq!(detect("trailing.").id, "plain");
    }
}
