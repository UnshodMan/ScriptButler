mod commands;
mod exec;
mod language;
mod models;
mod runner;
mod search;
mod state;

use tauri::Manager;

/// 当前时间（Unix 毫秒）
pub fn unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// Rust 业务层：持久化、排序、语言识别、命令拼接、脚本托管、检索与运行。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            app.manage(state::AppState::new(dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_scripts,
            commands::add_script,
            commands::manage_script,
            commands::remove_script,
            commands::set_work_dir,
            commands::mark_used,
            commands::ensure_default_scheme,
            commands::create_scheme,
            commands::update_scheme,
            commands::delete_scheme,
            commands::set_active_scheme,
            commands::read_script,
            commands::write_script,
            commands::start_run,
            commands::stop_run,
            commands::search_scripts,
            commands::search_contents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
