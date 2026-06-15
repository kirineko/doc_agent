pub mod agent;
pub mod core;
pub mod ipc;
pub mod state;
pub mod tools;

use ipc::updater::fetch_latest_release_version;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Ok(resource_dir) = app.path().resource_dir() {
                crate::tools::pdf::configure_resource_dir(resource_dir.join("pdfium"));
            }
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let state = AppState::new(data_dir).expect("failed to initialize app state");
            if let Ok(store) = state.store.lock() {
                if let Ok(projects) = store.list_projects() {
                    crate::core::asset_scope::allow_project_roots(
                        app.handle(),
                        projects.iter().map(|p| p.root_path.as_str()),
                    );
                }
            }
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::list_projects,
            ipc::create_project,
            ipc::hide_project,
            ipc::list_project_files_cmd,
            ipc::list_project_dir_cmd,
            ipc::open_project_file,
            ipc::generate_suggestions,
            ipc::list_sessions,
            ipc::create_session,
            ipc::update_session,
            ipc::delete_session,
            ipc::list_models,
            ipc::read_attachment_preview,
            ipc::get_session_context_usage,
            ipc::save_upload,
            ipc::list_messages,
            ipc::send_message,
            ipc::submit_clarify_answer,
            ipc::cancel_clarify,
            ipc::set_api_key,
            ipc::has_api_key,
            ipc::clear_api_key,
            ipc::pick_project_directory,
            fetch_latest_release_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
