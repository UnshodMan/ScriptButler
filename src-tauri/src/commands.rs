use tauri::{AppHandle, State};

use scriptbutler_core::exec::to_view;
use scriptbutler_core::ops::{self, ManageOptions};
use scriptbutler_core::search;

use crate::models::{ApiError, ListResponse, SearchGroups};
use crate::runner;
use crate::state::AppState;

#[tauri::command]
pub fn list_scripts(state: State<AppState>) -> ListResponse {
    let entries = state.lock_entries();
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed: None,
    }
}

#[tauri::command]
pub fn add_script(
    state: State<AppState>,
    path: String,
    default_scheme_name: String,
) -> Result<ListResponse, ApiError> {
    let mut entries = state.lock_entries();
    let outcome = ops::add_entry(&mut entries, &path, &default_scheme_name)?;
    state.save_entries(&entries);
    Ok(ListResponse {
        entries: ops::sorted_views(&entries),
        changed: Some(outcome.view),
    })
}

/// 托管：把脚本复制到应用数据目录下的 scripts/，以时间戳命名。
/// 新条目保留原脚本名、原始路径与全部运行方案，默认在原目录执行。
#[tauri::command]
pub fn manage_script(state: State<AppState>, path: String) -> Result<ListResponse, ApiError> {
    let mut entries = state.lock_entries();
    let outcome = ops::manage_script(
        &mut entries,
        &path,
        &state.managed_dir,
        &ManageOptions {
            scheme_name: "Default",
            name: None,
            command: None,
            work_dir: None,
            dedupe: false,
        },
    )?;
    state.save_entries(&entries);
    Ok(ListResponse {
        entries: ops::sorted_views(&entries),
        changed: Some(outcome.view),
    })
}

#[tauri::command]
pub fn remove_script(state: State<AppState>, path: String) -> ListResponse {
    let mut entries = state.lock_entries();
    if ops::remove_entry(&mut entries, &path) {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed: None,
    }
}

/// 托管脚本：修改执行目录（随时可改，立即持久化）
#[tauri::command]
pub fn set_work_dir(state: State<AppState>, path: String, work_dir: String) -> ListResponse {
    let mut entries = state.lock_entries();
    let changed = if ops::set_work_dir(&mut entries, &path, &work_dir) {
        entries.iter().find(|e| e.path == path).map(to_view)
    } else {
        None
    };
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn mark_used(state: State<AppState>, path: String) -> ListResponse {
    let mut entries = state.lock_entries();
    let changed = if ops::mark_used(&mut entries, &path) {
        entries.iter().find(|e| e.path == path).map(to_view)
    } else {
        None
    };
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed,
    }
}

/// 首次打开且无方案时，按语言生成默认方案
#[tauri::command]
pub fn ensure_default_scheme(state: State<AppState>, path: String, name: String) -> ListResponse {
    let mut entries = state.lock_entries();
    let changed = ops::ensure_default_scheme(&mut entries, &path, &name);
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn create_scheme(
    state: State<AppState>,
    path: String,
    name: String,
    command: String,
    cd_to_script_dir: bool,
) -> ListResponse {
    let mut entries = state.lock_entries();
    let changed = ops::create_scheme(&mut entries, &path, &name, &command, cd_to_script_dir);
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn update_scheme(
    state: State<AppState>,
    path: String,
    id: String,
    name: String,
    command: String,
    cd_to_script_dir: bool,
) -> ListResponse {
    let mut entries = state.lock_entries();
    let changed = ops::update_scheme(
        &mut entries,
        &path,
        &id,
        &name,
        &command,
        cd_to_script_dir,
    );
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn delete_scheme(state: State<AppState>, path: String, id: String) -> ListResponse {
    let mut entries = state.lock_entries();
    let changed = ops::delete_scheme(&mut entries, &path, &id);
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn set_active_scheme(state: State<AppState>, path: String, id: String) -> ListResponse {
    let mut entries = state.lock_entries();
    let changed = ops::set_active_scheme(&mut entries, &path, &id);
    if changed.is_some() {
        state.save_entries(&entries);
    }
    ListResponse {
        entries: ops::sorted_views(&entries),
        changed,
    }
}

#[tauri::command]
pub fn read_script(path: String) -> Result<String, ApiError> {
    std::fs::read_to_string(&path).map_err(|e| ApiError::ReadFailed {
        detail: e.to_string(),
    })
}

#[tauri::command]
pub fn write_script(path: String, content: String) -> Result<(), ApiError> {
    std::fs::write(&path, content).map_err(|e| ApiError::WriteFailed {
        detail: e.to_string(),
    })
}

#[tauri::command]
pub fn start_run(
    app: AppHandle,
    state: State<AppState>,
    path: String,
    scheme_id: String,
    args: Option<String>,
) -> Result<String, ApiError> {
    runner::start_run(app, &state, &path, &scheme_id, args.as_deref())
}

#[tauri::command]
pub fn stop_run(state: State<AppState>, run_id: String) {
    runner::stop_run(&state, &run_id);
}

#[tauri::command]
pub fn search_scripts(state: State<AppState>, query: String) -> SearchGroups {
    let entries = state.lock_entries();
    let sorted = ops::sorted_refs(&entries);
    search::group(&sorted, &query)
}

#[tauri::command]
pub async fn search_contents(paths: Vec<String>, keyword: String) -> Vec<String> {
    tauri::async_runtime::spawn_blocking(move || search::search_contents(&paths, &keyword))
        .await
        .unwrap_or_default()
}
