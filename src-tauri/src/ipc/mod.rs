use crate::agent::compaction::{force_compact_session, CompactSessionResponse};
use crate::agent::model_catalog::ModelCatalog;
use crate::agent::provider::openai_compat::{
    encode_attachment_data_url, is_allowed_image_mime, is_upload_attachment_path, model_from_str,
    validate_attachments, MAX_ATTACHMENT_BYTES,
};
use crate::agent::suggest;
use crate::agent::types::MessageAttachment;
use crate::agent::{clarify_interaction, loop_runner};
use crate::core::project_files::{
    list_project_dir, list_project_files, ProjectDirListing, ProjectFileList,
};
use crate::core::project_import::{import_project_file, ImportConflictStrategy, ImportResult};
use crate::core::sandbox::Sandbox;
use crate::core::store::{ClarifyPending, Message, Project, Session, ToolCallRecord};
use crate::state::AppState;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

pub mod provider_balance;
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

#[derive(Debug, Deserialize)]
pub struct CancelTurnRequest {
    pub session_id: String,
}

#[tauri::command]
pub fn cancel_turn(state: State<AppState>, req: CancelTurnRequest) -> Result<(), String> {
    state.turns.cancel(&req.session_id)
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
    let shared = state.inner().clone();
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
pub fn save_upload(
    state: State<AppState>,
    req: SaveUploadRequest,
) -> Result<SaveUploadResponse, String> {
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
pub async fn compact_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<CompactSessionResponse, String> {
    force_compact_session(&app, state.inner(), &session_id).await
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, AppState>,
    req: SendMessageRequest,
) -> Result<(), String> {
    let shared = state.inner().clone();
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
    let shared = state.inner().clone();
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
    let shared = state.inner().clone();
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
        .map_err(|e| e.to_string())?;
    if provider == "tavily" {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        crate::core::web_search::set_web_search_enabled(&store, true)?;
    }
    Ok(())
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
pub fn get_web_search_enabled(state: State<AppState>) -> Result<bool, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    crate::core::web_search::is_web_search_active(&state.secrets, &store)
}

#[tauri::command]
pub fn set_web_search_enabled(state: State<AppState>, enabled: bool) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    crate::core::web_search::set_web_search_enabled(&store, enabled)
}

#[tauri::command]
pub fn open_project_root(state: State<AppState>, project_id: String) -> Result<(), String> {
    let project = project_by_id(&state, &project_id)?;
    let sandbox = Sandbox::new(&project.root_path).map_err(|e| e.to_string())?;
    tauri_plugin_opener::open_path(sandbox.root(), None::<&str>).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct ImportProjectFileRequest {
    pub project_id: String,
    pub filename: String,
    pub data_base64: String,
    pub on_conflict: String,
}

fn parse_import_conflict(value: &str) -> Result<ImportConflictStrategy, String> {
    match value {
        "fail_if_exists" => Ok(ImportConflictStrategy::FailIfExists),
        "overwrite" => Ok(ImportConflictStrategy::Overwrite),
        "rename" => Ok(ImportConflictStrategy::Rename),
        other => Err(format!("unsupported on_conflict: {other}")),
    }
}

#[tauri::command]
pub fn import_project_file_cmd(
    state: State<AppState>,
    req: ImportProjectFileRequest,
) -> Result<ImportResult, String> {
    let project = project_by_id(&state, &req.project_id)?;
    let sandbox = Sandbox::new(&project.root_path).map_err(|e| e.to_string())?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(req.data_base64.as_bytes())
        .map_err(|e| format!("invalid base64: {e}"))?;
    let strategy = parse_import_conflict(&req.on_conflict)?;
    import_project_file(&sandbox, &req.filename, &bytes, strategy)
}

#[tauri::command]
pub async fn pick_project_directory(app: AppHandle) -> Result<Option<String>, String> {
    let path = app.dialog().file().blocking_pick_folder();
    Ok(path.map(|p| p.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::provider::openai_compat::{is_upload_attachment_path, validate_attachments};
    use crate::agent::types::MessageAttachment;
    use crate::state::AppState;
    use tempfile::tempdir;

    #[test]
    fn list_models_exposes_public_catalog() {
        let models = list_models();
        assert!(models.len() >= 6);
        assert!(models.iter().any(|m| m.id == "deepseek-v4-flash"));
        assert!(models
            .iter()
            .any(|m| m.id == "kimi-k2.6" && m.supports_vision));
    }

    #[test]
    fn read_attachment_preview_rejects_non_cache_path() {
        let attachment = MessageAttachment {
            path: "report.docx".into(),
            mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".into(),
        };
        assert!(!is_upload_attachment_path(&attachment.path));
        assert!(validate_attachments(&[attachment]).is_err());
    }

    #[test]
    fn message_bundle_shape_for_empty_session() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf()).unwrap();
        let project_id = {
            let store = state.store.lock().unwrap();
            store
                .create_project("demo", dir.path().join("root").to_str().unwrap())
                .unwrap()
                .id
        };
        std::fs::create_dir_all(dir.path().join("root")).unwrap();
        let session_id = {
            let store = state.store.lock().unwrap();
            store
                .create_session(&project_id, "s1", "deepseek-v4-flash", true, "high")
                .unwrap()
                .id
        };

        let store = state.store.lock().unwrap();
        let bundle = MessageBundle {
            messages: store.list_messages(&session_id).unwrap(),
            tool_calls: store.list_tool_calls_for_session(&session_id).unwrap(),
            clarify_pending: store.get_clarify_pending(&session_id).unwrap(),
        };
        assert!(bundle.messages.is_empty());
        assert!(bundle.tool_calls.is_empty());
        assert!(bundle.clarify_pending.is_none());
    }
}
