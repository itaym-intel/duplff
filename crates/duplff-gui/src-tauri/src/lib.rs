mod commands;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::start_scan,
            commands::get_results,
            commands::trash_files,
            commands::undo_last,
            commands::export_json,
            commands::export_csv,
            commands::open_in_file_manager,
            commands::dry_run,
            commands::list_action_logs,
            commands::undo_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
