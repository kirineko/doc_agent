use crate::agent::loop_runner;
use crate::core::store::{Message, Project, Session, ToolCallRecord};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub root_path: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub project_id: String,
    pub title: String,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub session_id: String,
    pub title: Option<String>,
    pub model: Option<String>,
    pub thinking_enabled: Option<bool>,
    pub thinking_effort: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: String,
    pub content: String,
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    state.store.lock().map_err(|e| e.to_string())?.list_projects().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    req: CreateProjectRequest,
) -> Result<Project, String> {
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .create_project(&req.name, &req.root_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_sessions(state: State<AppState>, project_id: String) -> Result<Vec<Session>, String> {
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .list_sessions(&project_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_session(
    state: State<AppState>,
    req: CreateSessionRequest,
) -> Result<Session, String> {
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .create_session(
            &req.project_id,
            &req.title,
            req.model.as_deref().unwrap_or("deepseek-v4-flash"),
            true,
            "high",
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_session(
    state: State<AppState>,
    req: UpdateSessionRequest,
) -> Result<Session, String> {
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .update_session(
            &req.session_id,
            req.title.as_deref(),
            req.model.as_deref(),
            req.thinking_enabled,
            req.thinking_effort.as_deref(),
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_session(state: State<AppState>, session_id: String) -> Result<(), String> {
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .delete_session(&session_id)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct MessageBundle {
    pub messages: Vec<Message>,
    pub tool_calls: Vec<ToolCallRecord>,
}

#[tauri::command]
pub fn list_messages(state: State<AppState>, session_id: String) -> Result<MessageBundle, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    Ok(MessageBundle {
        messages: store.list_messages(&session_id).map_err(|e| e.to_string())?,
        tool_calls: store
            .list_tool_calls_for_session(&session_id)
            .map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, AppState>,
    req: SendMessageRequest,
) -> Result<(), String> {
    let shared = AppState {
        store: state.store.clone(),
        secrets: state.secrets.clone(),
        tools: state.tools.clone(),
    };
    loop_runner::run_turn(app, shared, req.session_id, req.content).await
}

#[tauri::command]
pub fn set_api_key(state: State<AppState>, provider: String, api_key: String) -> Result<(), String> {
    state
        .secrets
        .set_api_key(&provider, &api_key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn has_api_key(state: State<AppState>, provider: String) -> Result<bool, String> {
    state
        .secrets
        .has_api_key(&provider)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_api_key(state: State<AppState>, provider: String) -> Result<(), String> {
    state
        .secrets
        .clear_api_key(&provider)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pick_project_directory(app: AppHandle) -> Result<Option<String>, String> {
    let path = app
        .dialog()
        .file()
        .blocking_pick_folder();
    Ok(path.map(|p| p.to_string()))
}
