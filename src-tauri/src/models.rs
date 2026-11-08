use serde::{Deserialize, Serialize};

pub fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunScheme {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default = "default_true")]
    pub cd_to_script_dir: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScriptEntry {
    pub path: String,
    pub name: String,
    pub lang: String,
    pub last_used_at: i64,
    pub use_count: u64,
    #[serde(default)]
    pub schemes: Vec<RunScheme>,
    #[serde(default)]
    pub active_scheme_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_dir: Option<String>,
}

impl ScriptEntry {
    pub fn is_managed(&self) -> bool {
        self.managed.unwrap_or(false)
    }

    /// 激活方案：按 id 命中，否则回退首个方案
    pub fn active_scheme(&self) -> Option<&RunScheme> {
        self.schemes
            .iter()
            .find(|s| Some(&s.id) == self.active_scheme_id.as_ref())
            .or_else(|| self.schemes.first())
    }
}

/// 返回给前端的视图模型：在条目基础上附带 Rust 计算好的派生字段
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryView {
    #[serde(flatten)]
    pub entry: ScriptEntry,
    pub exec_dir: Option<String>,
    pub effective_command: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub entries: Vec<EntryView>,
    pub changed: Option<EntryView>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchGroups {
    pub by_name: Vec<String>,
    pub by_dir: Vec<String>,
    pub pending: Vec<String>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ApiError {
    NotFound,
    NotFile,
    ManageFailed { detail: String },
    ReadFailed { detail: String },
    WriteFailed { detail: String },
    SpawnFailed { detail: String },
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RunEvent {
    Stdout { line: String },
    Stderr { line: String },
    Exit { code: Option<i32> },
    Error { message: String },
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunPayload {
    pub run_id: String,
    #[serde(flatten)]
    pub event: RunEvent,
}
