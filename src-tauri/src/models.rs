use serde::Serialize;

use scriptbutler_core::ops::CoreError;
pub use scriptbutler_core::models::{EntryView, SearchGroups};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub entries: Vec<EntryView>,
    pub changed: Option<EntryView>,
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

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::NotFound => ApiError::NotFound,
            CoreError::NotFile => ApiError::NotFile,
            CoreError::ManageFailed(detail) => ApiError::ManageFailed { detail },
            CoreError::InvalidData(detail) | CoreError::Io(detail) => {
                ApiError::ReadFailed { detail }
            }
        }
    }
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
