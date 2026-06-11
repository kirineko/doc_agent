use crate::agent::suggest;
use crate::agent::{clarify_interaction, loop_runner};
use crate::core::project_files::{
    list_project_dir, list_project_files, ProjectDirListing, ProjectFileList,
};
use crate::core::sandbox::Sandbox;
use crate::core::store::{ClarifyPending, Message, Project, Session, ToolCallRecord};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
    pub thinking_enabled: Option<bool>,
    pub thinking_effort: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct SubmitClarifyAnswerRequest {
    pub session_id: String,
    pub question_id: String,
    #[serde(default)]
    pub selected: Vec<String>,
    pub custom: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CancelClarifyRequest {
    pub session_id: String,
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .list_projects()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_project(state: State<AppState>, project_id: String) -> Result<(), String> {
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .hide_project(&project_id)
        .map_err(|e| e.to_string())
}

/// 取项目并尽快释放 store 锁，后续文件 I/O 不持锁。
fn project_by_id(state: &State<AppState>, project_id: &str) -> Result<Project, String> {
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get_project(project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "project not found".to_string())
}

#[tauri::command]
pub fn list_project_files_cmd(
    state: State<AppState>,
    project_id: String,
) -> Result<ProjectFileList, String> {
    let project = project_by_id(&state, &project_id)?;
    Ok(list_project_files(
        PathBuf::from(&project.root_path).as_path(),
    ))
}

#[tauri::command]
pub fn list_project_dir_cmd(
    state: State<AppState>,
    project_id: String,
    relative_path: String,
) -> Result<ProjectDirListing, String> {
    let project = project_by_id(&state, &project_id)?;
    list_project_dir(PathBuf::from(&project.root_path).as_path(), &relative_path)
}

#[tauri::command]
pub fn open_project_file(
    state: State<AppState>,
    project_id: String,
    relative_path: String,
) -> Result<(), String> {
    let project = project_by_id(&state, &project_id)?;
    let sandbox = Sandbox::new(&project.root_path).map_err(|e| e.to_string())?;
    let resolved = sandbox.resolve(&relative_path).map_err(|e| e.to_string())?;
    if resolved.is_dir() {
        return Err("cannot open a directory".into());
    }
    tauri_plugin_opener::open_path(&resolved, None::<&str>).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct GenerateSuggestionsRequest {
    pub session_id: String,
    pub kind: String,
}

#[tauri::command]
pub async fn generate_suggestions(
    state: State<'_, AppState>,
    req: GenerateSuggestionsRequest,
) -> Result<Vec<String>, String> {
    let shared = AppState {
        store: state.store.clone(),
        secrets: state.secrets.clone(),
        tools: state.tools.clone(),
    };
    suggest::generate_suggestions(shared, req.session_id, &req.kind).await
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
            req.thinking_enabled.unwrap_or(true),
            req.thinking_effort.as_deref().unwrap_or("high"),
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
    pub clarify_pending: Option<ClarifyPending>,
}

#[tauri::command]
pub fn list_messages(state: State<AppState>, session_id: String) -> Result<MessageBundle, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    Ok(MessageBundle {
        messages: store
            .list_messages(&session_id)
            .map_err(|e| e.to_string())?,
        tool_calls: store
            .list_tool_calls_for_session(&session_id)
            .map_err(|e| e.to_string())?,
        clarify_pending: store
            .get_clarify_pending(&session_id)
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
pub async fn submit_clarify_answer(
    app: AppHandle,
    state: State<'_, AppState>,
    req: SubmitClarifyAnswerRequest,
) -> Result<(), String> {
    let shared = AppState {
        store: state.store.clone(),
        secrets: state.secrets.clone(),
        tools: state.tools.clone(),
    };
    clarify_interaction::submit_clarify_answer(
        app,
        shared,
        clarify_interaction::SubmitClarifyAnswer {
            session_id: req.session_id,
            question_id: req.question_id,
            selected: req.selected,
            custom: req.custom,
        },
    )
    .await
}

#[tauri::command]
pub async fn cancel_clarify(
    app: AppHandle,
    state: State<'_, AppState>,
    req: CancelClarifyRequest,
) -> Result<(), String> {
    let shared = AppState {
        store: state.store.clone(),
        secrets: state.secrets.clone(),
        tools: state.tools.clone(),
    };
    clarify_interaction::cancel_clarify(app, shared, req.session_id).await
}

#[tauri::command]
pub fn set_api_key(
    state: State<AppState>,
    provider: String,
    api_key: String,
) -> Result<(), String> {
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
    let path = app.dialog().file().blocking_pick_folder();
    Ok(path.map(|p| p.to_string()))
}
