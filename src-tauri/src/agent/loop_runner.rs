use crate::agent::compaction::{
    compact_session_if_needed, emit_context_usage, estimate_chat_message_tokens,
    estimate_chat_messages_tokens, MAX_TOOL_STEPS,
};
use crate::agent::loop_support::*;
use crate::agent::loop_tool_batch::run_tool_batch;
use crate::agent::provider::openai_compat::{
    effort_from_str, model_from_str, validate_attachments,
};
use crate::agent::provider::provider_for;
use crate::agent::provider::ProviderError;
use crate::agent::turn_control::{
    format_turn_start_error, is_project_busy_user_error, is_session_busy_user_error, CancelSignal,
    TurnRegistry, TURN_CANCELLED,
};
use tauri::Emitter;
use crate::agent::types::{
    AgentEvent, ChatMessage, ChatRequest, MessageAttachment, ModelId, ThinkingConfig,
};
use crate::core::sandbox::Sandbox;
use crate::state::AppState;
use std::sync::Arc;
use tauri::{AppHandle, Runtime};
use uuid::Uuid;

pub(crate) struct ActiveTurnGuard {
    turns: Arc<TurnRegistry>,
    session_id: String,
}

impl ActiveTurnGuard {
    pub(crate) fn new(state: &AppState, session_id: &str) -> Self {
        Self {
            turns: state.turns.clone(),
            session_id: session_id.to_string(),
        }
    }
}

impl Drop for ActiveTurnGuard {
    fn drop(&mut self) {
        self.turns.unregister(&self.session_id);
    }
}

fn ensure_turn_can_start(
    state: &AppState,
    session_id: &str,
    project_id: &str,
) -> Result<(), String> {
    state
        .turns
        .preflight_turn_start(session_id, project_id)
        .map_err(|err| format_turn_start_error(state, err))
}

pub(crate) fn register_active_turn(
    state: &AppState,
    session_id: &str,
    turn_id: &str,
    project_id: &str,
) -> Result<CancelSignal, String> {
    ensure_turn_can_start(state, session_id, project_id)?;
    state
        .turns
        .register(
            session_id.to_string(),
            turn_id.to_string(),
            project_id.to_string(),
        )
        .map_err(|err| format_turn_start_error(state, err))
}

pub(crate) fn register_reserved_turn(
    state: &AppState,
    session_id: &str,
    turn_id: &str,
    project_id: &str,
) -> Result<CancelSignal, String> {
    state
        .turns
        .register_reserved(
            session_id.to_string(),
            turn_id.to_string(),
            project_id.to_string(),
        )
        .map_err(|err| format_turn_start_error(state, err))
}

fn session_project_id(state: &AppState, session_id: &str) -> Result<String, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let session = store
        .get_session(session_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "session not found".to_string())?;
    Ok(session.project_id)
}

async fn resume_with_registration<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    turn_id: String,
    register: fn(&AppState, &str, &str, &str) -> Result<CancelSignal, String>,
) -> Result<(), String> {
    let project_id = session_project_id(&state, &session_id)?;
    let cancel = register(&state, &session_id, &turn_id, &project_id)?;
    let _turn_guard = ActiveTurnGuard::new(&state, &session_id);
    resume_loop_from_store(app, state, session_id, turn_id, cancel).await
}

pub(crate) fn spawn_reserved_resume_on_busy<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    turn_id: String,
    project_id: String,
) -> Result<(), String> {
    state
        .turns
        .reserve_resume(session_id.clone(), project_id)?;
    tauri::async_runtime::spawn(async move {
        for _attempt in 0..120 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            match resume_reserved_turn(
                app.clone(),
                state.clone(),
                session_id.clone(),
                turn_id.clone(),
            )
            .await
            {
                Ok(()) => return,
                Err(err)
                    if is_project_busy_user_error(&err) || is_session_busy_user_error(&err) =>
                {
                    continue;
                }
                Err(err) => {
                    let _ = app.emit(
                        "agent-event",
                        AgentEvent::Error {
                            session_id: session_id.clone(),
                            turn_id: turn_id.clone(),
                            message: err,
                        },
                    );
                    state.turns.unreserve(&session_id);
                    return;
                }
            }
        }
        let _ = app.emit(
            "agent-event",
            AgentEvent::Error {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                message: "澄清取消已保存，但等待恢复超时。".into(),
            },
        );
        state.turns.unreserve(&session_id);
    });
    Ok(())
}

fn finish_cancelled<R: Runtime>(
    app: &AppHandle<R>,
    sandbox: &Sandbox,
    session_id: String,
    turn_id: String,
) {
    cleanup_skill_run_tmp(sandbox);
    emit(
        app,
        AgentEvent::TurnCancelled {
            session_id,
            turn_id,
        },
    );
}

fn return_if_cancelled<R: Runtime>(
    app: &AppHandle<R>,
    sandbox: &Sandbox,
    session_id: &str,
    turn_id: &str,
    cancel: &CancelSignal,
) -> Option<Result<(), String>> {
    if cancel.is_cancelled() {
        finish_cancelled(
            app,
            sandbox,
            session_id.to_string(),
            turn_id.to_string(),
        );
        Some(Ok(()))
    } else {
        None
    }
}

pub async fn run_turn<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    user_text: String,
    attachments: Vec<MessageAttachment>,
) -> Result<(), String> {
    let turn_id = Uuid::new_v4().to_string();
    let (
        _session_title,
        project,
        model,
        thinking_enabled,
        thinking_effort,
    ) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let session = store
            .get_session(&session_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "session not found".to_string())?;
        let project = store
            .get_project(&session.project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "project not found".to_string())?;
        (
            session.title,
            project,
            session.model.clone(),
            session.thinking_enabled,
            session.thinking_effort.clone(),
        )
    };

    ensure_turn_can_start(&state, &session_id, &project.id)?;

    let (history, tool_call_history) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let history = store
            .list_active_messages(&session_id)
            .map_err(|e| e.to_string())?;
        let tool_call_history = store
            .list_tool_calls_for_session(&session_id)
            .map_err(|e| e.to_string())?;
        (history, tool_call_history)
    };

    if user_text.trim().is_empty() && attachments.is_empty() {
        return Err("消息不能为空".into());
    }

    let model_id = model_from_str(&model);
    validate_attachments(&attachments).map_err(|e| e.to_string())?;
    if !attachments.is_empty() && !model_id.supports_vision() {
        return Err("当前模型不支持图片输入，请选用 Kimi K2.6 或 MiMo v2.5".into());
    }

    let sandbox = Sandbox::new(&project.root_path).map_err(|e| e.to_string())?;
    let attachments_json = if attachments.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&attachments).map_err(|e| e.to_string())?)
    };

    {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        if store
            .get_clarify_pending(&session_id)
            .map_err(|e| e.to_string())?
            .is_some()
        {
            return Err("请先回答当前澄清问题，再发送新消息。".into());
        }
    }

    let user_count = history.iter().filter(|m| m.role == "user").count() + 1;
    let web_enabled = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        crate::core::web_search::is_web_search_active(&state.secrets, &store)?
    };
    let mut working_messages = build_working_messages(
        &history,
        &tool_call_history,
        Some(&user_text),
        &attachments,
        web_enabled,
        Some(&sandbox),
    )?;

    let cancel = register_active_turn(&state, &session_id, &turn_id, &project.id)?;
    let _turn_guard = ActiveTurnGuard::new(&state, &session_id);

    {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .add_message(
                &session_id,
                "user",
                Some(&user_text),
                None,
                None,
                attachments_json.as_deref(),
            )
            .map_err(|e| e.to_string())?;
    }

    continue_loop_inner(
        app,
        state.clone(),
        session_id.clone(),
        turn_id,
        user_count,
        user_text,
        project.root_path,
        model,
        thinking_enabled,
        thinking_effort,
        web_enabled,
        &mut working_messages,
        cancel,
    )
    .await
}

pub async fn resume_turn<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    turn_id: String,
) -> Result<(), String> {
    resume_with_registration(app, state, session_id, turn_id, register_active_turn).await
}

pub(crate) async fn resume_reserved_turn<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    turn_id: String,
) -> Result<(), String> {
    resume_with_registration(app, state, session_id, turn_id, register_reserved_turn).await
}

pub(crate) async fn resume_loop_from_store<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    turn_id: String,
    cancel: CancelSignal,
) -> Result<(), String> {
    let (
        _session_title,
        project,
        history,
        tool_call_history,
        model,
        thinking_enabled,
        thinking_effort,
    ) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let session = store
            .get_session(&session_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "session not found".to_string())?;
        let project = store
            .get_project(&session.project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "project not found".to_string())?;
        let history = store
            .list_active_messages(&session_id)
            .map_err(|e| e.to_string())?;
        let tool_call_history = store
            .list_tool_calls_for_session(&session_id)
            .map_err(|e| e.to_string())?;
        (
            session.title,
            project,
            history,
            tool_call_history,
            session.model.clone(),
            session.thinking_enabled,
            session.thinking_effort.clone(),
        )
    };

    let user_count = history.iter().filter(|m| m.role == "user").count();
    let user_text = history
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .and_then(|m| m.content.clone())
        .unwrap_or_default();
    let web_enabled = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        crate::core::web_search::is_web_search_active(&state.secrets, &store)?
    };
    let sandbox = Sandbox::new(&project.root_path).map_err(|e| e.to_string())?;
    let mut working_messages = build_working_messages(
        &history,
        &tool_call_history,
        None,
        &[],
        web_enabled,
        Some(&sandbox),
    )?;

    continue_loop_inner(
        app,
        state.clone(),
        session_id.clone(),
        turn_id,
        user_count,
        user_text,
        project.root_path,
        model,
        thinking_enabled,
        thinking_effort,
        web_enabled,
        &mut working_messages,
        cancel,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn continue_loop_inner<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    turn_id: String,
    user_count: usize,
    user_text: String,
    project_root: String,
    model: String,
    thinking_enabled: bool,
    thinking_effort: String,
    web_enabled: bool,
    working_messages: &mut Vec<ChatMessage>,
    cancel: CancelSignal,
) -> Result<(), String> {
    let sandbox = Sandbox::new(&project_root).map_err(|e| e.to_string())?;
    let model_id = model_from_str(&model);
    let api_key = if model_id == ModelId::Mock {
        None
    } else {
        state
            .secrets
            .get_api_key(model_id.provider_key())
            .map_err(|e| e.to_string())?
    };

    let provider = provider_for(model_id);
    let tool_defs = state.tools.tools_for_model(model_id, web_enabled);

    let mut token_count = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .get_session_token_count(&session_id)
            .map_err(|e| e.to_string())?
            .unwrap_or(0)
    };
    let full_estimate = estimate_chat_messages_tokens(working_messages);
    let mut pending_estimate = if token_count == 0 {
        full_estimate
    } else {
        full_estimate.saturating_sub(token_count)
    };

    for _step in 0..MAX_TOOL_STEPS {
        if let Some(result) =
            return_if_cancelled(&app, &sandbox, &session_id, &turn_id, &cancel)
        {
            return result;
        }

        let compaction_result = compact_session_if_needed(
            &app,
            &state,
            &session_id,
            &turn_id,
            model_id,
            api_key.as_deref(),
            token_count,
            pending_estimate,
            web_enabled,
            &cancel,
        )
        .await;
        if let Some(result) =
            return_if_cancelled(&app, &sandbox, &session_id, &turn_id, &cancel)
        {
            return result;
        }
        let (rebuilt, new_token_count, new_pending, compaction) = match compaction_result {
            Ok(v) => v,
            Err(err) if err == TURN_CANCELLED => {
                finish_cancelled(&app, &sandbox, session_id.clone(), turn_id.clone());
                return Ok(());
            }
            Err(err) => return Err(err),
        };
        if compaction.is_some() {
            *working_messages = rebuilt;
            token_count = new_token_count;
            pending_estimate = new_pending;
        }

        let request = ChatRequest {
            session_id: session_id.clone(),
            turn_id: turn_id.clone(),
            model: model_id,
            messages: working_messages.clone(),
            tools: tool_defs.clone(),
            thinking: ThinkingConfig {
                enabled: thinking_enabled,
                effort: effort_from_str(&thinking_effort),
            },
            response_format: None,
            max_tokens: None,
            cancel: Some(cancel.clone()),
        };

        let session_id_for_events = session_id.clone();
        let turn_id_for_events = turn_id.clone();
        let app_for_events = app.clone();
        let mut on_event = move |event: AgentEvent| {
            let mapped = match event {
                AgentEvent::ReasoningToken { delta, .. } => AgentEvent::ReasoningToken {
                    session_id: session_id_for_events.clone(),
                    turn_id: turn_id_for_events.clone(),
                    delta,
                },
                AgentEvent::ContentToken { delta, .. } => AgentEvent::ContentToken {
                    session_id: session_id_for_events.clone(),
                    turn_id: turn_id_for_events.clone(),
                    delta,
                },
                other => other,
            };
            emit(&app_for_events, mapped);
        };

        let turn = match provider
            .chat_stream(request, api_key.as_deref(), &mut on_event)
            .await
        {
            Ok(turn) => turn,
            Err(ProviderError::Cancelled) => {
                finish_cancelled(&app, &sandbox, session_id.clone(), turn_id.clone());
                return Ok(());
            }
            Err(e) => return Err(e.to_string()),
        };

        let mut tool_calls = turn.tool_calls;
        let stream_indices: Vec<usize> = tool_calls
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.function.name.is_empty())
            .map(|(index, _)| index)
            .collect();
        tool_calls.retain(|c| !c.function.name.is_empty());
        {
            let store = state.store.lock().map_err(|e| e.to_string())?;
            normalize_tool_call_ids(&mut tool_calls, |id| {
                store.tool_call_exists(id).unwrap_or(false)
            });
        }

        let usage_reported = turn.usage.is_some();
        if let Some(usage) = turn.usage {
            token_count = usage.total;
            pending_estimate = 0;
            {
                let store = state.store.lock().map_err(|e| e.to_string())?;
                store
                    .set_session_token_count(&session_id, token_count)
                    .map_err(|e| e.to_string())?;
            }
            emit_context_usage(&app, &session_id, token_count, model_id.max_context_size());
        }

        if tool_calls.is_empty() {
            cleanup_skill_run_tmp(&sandbox);
            let msg = persist_assistant(
                &state,
                &session_id,
                Some(turn.content.as_str()),
                Some(turn.reasoning_content.as_str()),
                None,
            )?;
            if !usage_reported {
                let mut estimated_messages = working_messages.clone();
                estimated_messages.push(ChatMessage {
                    role: "assistant".into(),
                    content: Some(turn.content.clone()),
                    image_urls: vec![],
                    reasoning_content: Some(turn.reasoning_content.clone()),
                    tool_calls: None,
                    tool_call_id: None,
                });
                token_count = estimate_chat_messages_tokens(&estimated_messages);
                let store = state.store.lock().map_err(|e| e.to_string())?;
                store
                    .set_session_token_count(&session_id, token_count)
                    .map_err(|e| e.to_string())?;
                emit_context_usage(&app, &session_id, token_count, model_id.max_context_size());
            }
            maybe_autotitle_session(&app, &state, &session_id, user_count, &user_text)?;
            emit_assistant_step_done(&app, &session_id, &turn_id, &msg);
            emit(
                &app,
                AgentEvent::TurnComplete {
                    session_id,
                    turn_id,
                },
            );
            return Ok(());
        }

        let assistant_msg = persist_assistant(
            &state,
            &session_id,
            if turn.content.is_empty() {
                None
            } else {
                Some(&turn.content)
            },
            Some(&turn.reasoning_content),
            Some(&tool_calls),
        )?;
        emit_assistant_step_done(&app, &session_id, &turn_id, &assistant_msg);

        working_messages.push(ChatMessage {
            role: "assistant".into(),
            content: if turn.content.is_empty() {
                None
            } else {
                Some(turn.content.clone())
            },
            image_urls: vec![],
            reasoning_content: Some(turn.reasoning_content.clone()),
            tool_calls: Some(tool_calls.clone()),
            tool_call_id: None,
        });
        if !usage_reported {
            pending_estimate += estimate_chat_message_tokens(working_messages.last().unwrap());
        }

        let outcome = run_tool_batch(
            &app,
            &state,
            &sandbox,
            &session_id,
            &turn_id,
            model_id,
            &tool_calls,
            &stream_indices,
            turn.finish_reason.as_deref() == Some("length"),
            working_messages,
            &mut pending_estimate,
            &cancel,
        )
        .await?;

        if outcome.cancelled {
            finish_cancelled(&app, &sandbox, session_id.clone(), turn_id.clone());
            return Ok(());
        }

        if outcome.has_pending_clarify {
            state.turns.unregister(&session_id);
            emit(
                &app,
                AgentEvent::TurnAwaitingUser {
                    session_id,
                    turn_id,
                },
            );
            return Ok(());
        }
    }

    cleanup_skill_run_tmp(&sandbox);
    emit(
        &app,
        AgentEvent::Error {
            session_id,
            turn_id,
            message: "Reached maximum tool steps".into(),
        },
    );
    Ok(())
}

#[cfg(test)]
mod token_accounting_tests {
    #[test]
    fn effective_count_counts_tools_not_assistant_after_usage() {
        let token_count = 1500u32;
        let assistant_tokens = 200u32;
        let tool_tokens = 100u32;
        let mut pending = 0u32;
        pending += tool_tokens;
        assert_eq!(
            token_count + pending,
            1600,
            "usage.total already includes assistant completion"
        );
        assert_ne!(
            token_count + pending + assistant_tokens,
            token_count + pending,
            "assistant must not be added to pending after usage update"
        );
    }
}

/// Turn 结束兜底：无论 style_warnings 是否被处理，只要没有未修复的脚本失败
/// （`.cache/skill-run/error.json` 不存在），就清理 `.cache/skill-run/` 临时目录。
#[cfg(test)]
#[path = "loop_runner_tests.rs"]
mod loop_runner_tests;
