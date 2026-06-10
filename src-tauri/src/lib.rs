pub mod agent;
pub mod core;
pub mod ipc;
pub mod state;
pub mod tools;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            crate::core::secrets::Secrets::cleanup_legacy_keychain();
            if let Ok(resource_dir) = app.path().resource_dir() {
                crate::tools::pdf::configure_resource_dir(resource_dir.join("pdfium"));
            }
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("doc_agent.db");
            let state = AppState::new(db_path).expect("failed to initialize app state");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::list_projects,
            ipc::create_project,
            ipc::hide_project,
            ipc::list_project_files_cmd,
            ipc::generate_suggestions,
            ipc::list_sessions,
            ipc::create_session,
            ipc::update_session,
            ipc::delete_session,
            ipc::list_messages,
            ipc::send_message,
            ipc::set_api_key,
            ipc::has_api_key,
            ipc::clear_api_key,
            ipc::pick_project_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
