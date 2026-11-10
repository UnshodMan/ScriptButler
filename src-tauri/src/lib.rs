mod commands;
mod models;
mod runner;
mod state;

use tauri::Manager;

// Tauri 适配层：IPC 命令、子进程事件与状态同步；业务逻辑位于 scriptbutler-core。
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
