use crate::agent::model_catalog::ModelCatalog;
use crate::agent::suggest;
use crate::agent::{clarify_interaction, loop_runner};
use crate::agent::types::MessageAttachment;
use crate::agent::provider::openai_compat::{
    encode_attachment_data_url, is_allowed_image_mime, is_upload_attachment_path,
    model_from_str, validate_attachments, MAX_ATTACHMENT_BYTES,
};
use base64::Engine;
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

pub mod updater;

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
pub struct MessageAttachmentInput {
    pub path: String,
    pub mime: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: String,
    pub content: String,
    #[serde(default)]
    pub attachments: Vec<MessageAttachmentInput>,
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
    app: AppHandle,
    state: State<AppState>,
    req: CreateProjectRequest,
) -> Result<Project, String> {
    let project = state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .create_project(&req.name, &req.root_path)
        .map_err(|e| e.to_string())?;
    crate::core::asset_scope::allow_project_root(&app, &project.root_path);
    Ok(project)
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

#[derive(Debug, Deserialize)]
pub struct SaveUploadRequest {
    pub project_id: String,
    pub filename: String,
    pub mime: String,
    pub data_base64: String,
}

#[derive(Debug, Serialize)]
pub struct SaveUploadResponse {
    pub path: String,
    pub mime: String,
}

#[tauri::command]
pub fn list_models() -> Vec<crate::agent::model_catalog::ModelInfo> {
    ModelCatalog::list_public().cloned().collect()
}

#[derive(Debug, Deserialize)]
pub struct ReadAttachmentPreviewRequest {
    pub project_id: String,
    pub path: String,
    pub mime: String,
}

#[tauri::command]
pub fn read_attachment_preview(
    state: State<AppState>,
    req: ReadAttachmentPreviewRequest,
) -> Result<String, String> {
    if !is_upload_attachment_path(&req.path) {
        return Err("attachment path must be under .cache/attachments/".into());
    }
    let attachment = MessageAttachment {
        path: req.path,
        mime: req.mime,
    };
    validate_attachments(std::slice::from_ref(&attachment)).map_err(|e| e.to_string())?;
    let project = project_by_id(&state, &req.project_id)?;
    let sandbox = Sandbox::new(&project.root_path).map_err(|e| e.to_string())?;
    encode_attachment_data_url(&sandbox, &attachment)
}

#[derive(Debug, Serialize)]
pub struct SessionContextUsage {
    pub used_tokens: u32,
    pub max_tokens: u32,
    pub ratio: f64,
}

#[tauri::command]
pub fn get_session_context_usage(
    state: State<AppState>,
    session_id: String,
) -> Result<SessionContextUsage, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let session = store
        .get_session(&session_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "session not found".to_string())?;
    let used = store
        .get_session_token_count(&session_id)
        .map_err(|e| e.to_string())?
        .unwrap_or(0);
    let max = model_from_str(&session.model).max_context_size();
    let ratio = if max == 0 {
        0.0
    } else {
        used as f64 / max as f64
    };
    Ok(SessionContextUsage {
        used_tokens: used,
        max_tokens: max,
        ratio,
    })
}

#[tauri::command]
pub fn save_upload(state: State<AppState>, req: SaveUploadRequest) -> Result<SaveUploadResponse, String> {
    if !is_allowed_image_mime(&req.mime) {
        return Err(format!("unsupported image mime: {}", req.mime));
    }
    let project = project_by_id(&state, &req.project_id)?;
    let sandbox = Sandbox::new(&project.root_path).map_err(|e| e.to_string())?;
    let attachments_dir = sandbox
        .resolve_for_write(crate::core::cache_paths::ATTACHMENTS_DIR)
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&attachments_dir).map_err(|e| e.to_string())?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(req.data_base64.as_bytes())
        .map_err(|e| format!("invalid base64: {e}"))?;
    if bytes.len() as u64 > MAX_ATTACHMENT_BYTES {
        return Err(format!(
            "attachment exceeds {}MB limit",
            MAX_ATTACHMENT_BYTES / 1024 / 1024
        ));
    }

    let ext = std::path::Path::new(&req.filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png");
    let stored_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    let relative_path = crate::core::cache_paths::attachment_rel_path(&stored_name);
    let target = sandbox
        .resolve_for_write(&relative_path)
        .map_err(|e| e.to_string())?;
    std::fs::write(&target, bytes).map_err(|e| e.to_string())?;

    Ok(SaveUploadResponse {
        path: relative_path,
        mime: req.mime,
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
    let attachments: Vec<MessageAttachment> = req
        .attachments
        .into_iter()
        .map(|item| MessageAttachment {
            path: item.path,
            mime: item.mime,
        })
        .collect();
    loop_runner::run_turn(app, shared, req.session_id, req.content, attachments).await
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
