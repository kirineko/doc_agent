use crate::agent::loop_support::*;
use crate::agent::provider::openai_compat::{effort_from_str, model_from_str};
use crate::agent::provider::provider_for;
use crate::agent::tool_args::{parse_tool_arguments, truncation_error};
use crate::agent::types::{AgentEvent, ChatMessage, ChatRequest, ModelId, ThinkingConfig};
use crate::core::sandbox::Sandbox;
use crate::state::AppState;
use crate::tools::ToolContext;
use serde_json::{json, Value};
use std::time::Instant;
use tauri::{AppHandle, Runtime};
use uuid::Uuid;

const MAX_TOOL_STEPS: usize = 32;

pub async fn run_turn<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    user_text: String,
) -> Result<(), String> {
    let turn_id = Uuid::new_v4().to_string();
    let (
        session_title,
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
            .list_messages(&session_id)
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

    {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        if store
            .get_clarify_pending(&session_id)
            .map_err(|e| e.to_string())?
            .is_some()
        {
            return Err("请先回答当前澄清问题，再发送新消息。".into());
        }
        store
            .add_message(&session_id, "user", Some(&user_text), None, None)
            .map_err(|e| e.to_string())?;
    }

    let user_count = history.iter().filter(|m| m.role == "user").count() + 1;
    let web_enabled = state
        .secrets
        .has_api_key("tavily")
        .map_err(|e| e.to_string())?;
    let mut working_messages =
        build_working_messages(&history, &tool_call_history, Some(&user_text), web_enabled);

    continue_loop(
        app,
        state,
        session_id,
        turn_id,
        session_title,
        user_count,
        user_text,
        project.root_path,
        model,
        thinking_enabled,
        thinking_effort,
        web_enabled,
        &mut working_messages,
    )
    .await
}

pub async fn resume_turn<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    turn_id: String,
) -> Result<(), String> {
    let (
        session_title,
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
            .list_messages(&session_id)
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
    let web_enabled = state
        .secrets
        .has_api_key("tavily")
        .map_err(|e| e.to_string())?;
    let mut working_messages =
        build_working_messages(&history, &tool_call_history, None, web_enabled);

    continue_loop(
        app,
        state,
        session_id,
        turn_id,
        session_title,
        user_count,
        user_text,
        project.root_path,
        model,
        thinking_enabled,
        thinking_effort,
        web_enabled,
        &mut working_messages,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn continue_loop<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
    turn_id: String,
    session_title: String,
    user_count: usize,
    user_text: String,
    project_root: String,
    model: String,
    thinking_enabled: bool,
    thinking_effort: String,
    web_enabled: bool,
    working_messages: &mut Vec<ChatMessage>,
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
    let tool_defs = state.tools.definitions(web_enabled);

    for _step in 0..MAX_TOOL_STEPS {
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

        let turn = provider
            .chat_stream(request, api_key.as_deref(), &mut on_event)
            .await
            .map_err(|e| e.to_string())?;

        if turn.tool_calls.is_empty() {
            cleanup_skill_run_tmp(&sandbox);
            let msg = persist_assistant(
                &state,
                &session_id,
                Some(turn.content.as_str()),
                Some(turn.reasoning_content.as_str()),
                None,
            )?;
            maybe_autotitle_session(
                &state,
                &session_id,
                &session_title,
                user_count,
                &user_text,
                Some(turn.content.as_str()),
            )?;
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
            Some(&turn.tool_calls),
        )?;
        emit_assistant_step_done(&app, &session_id, &turn_id, &assistant_msg);

        working_messages.push(ChatMessage {
            role: "assistant".into(),
            content: if turn.content.is_empty() {
                None
            } else {
                Some(turn.content.clone())
            },
            reasoning_content: Some(turn.reasoning_content.clone()),
            tool_calls: Some(turn.tool_calls.clone()),
            tool_call_id: None,
        });

        let mut has_pending_clarify = false;
        for call in &turn.tool_calls {
            let ctx = ToolContext::with_secrets(&sandbox, &state.secrets);
            let truncated = turn.finish_reason.as_deref() == Some("length");
            let args_result =
                match parse_tool_arguments(&call.function.name, &call.function.arguments) {
                    Ok(args) => Ok(args),
                    Err(_) if truncated => Err(truncation_error(
                        &call.function.name,
                        &call.function.arguments,
                    )),
                    Err(err) => Err(err),
                };

            let (args, prebuilt_error) = match args_result {
                Ok(args) => (args, None),
                Err(err) => (Value::Object(Default::default()), Some(err)),
            };

            if call.function.name == "clarify_ask" && prebuilt_error.is_none() {
                match crate::tools::clarify::parse_question(args.clone()) {
                    Ok(question) if !has_pending_clarify => {
                        has_pending_clarify = true;
                        let normalized_args =
                            serde_json::to_value(&question).map_err(|e| e.to_string())?;
                        let question_json =
                            serde_json::to_string(&normalized_args).map_err(|e| e.to_string())?;
                        persist_clarify_pending(
                            &state,
                            &session_id,
                            &turn_id,
                            &call.id,
                            &question_json,
                        )?;
                        emit(
                            &app,
                            AgentEvent::ToolCall {
                                session_id: session_id.clone(),
                                turn_id: turn_id.clone(),
                                id: call.id.clone(),
                                name: call.function.name.clone(),
                                args: normalized_args,
                                status: "awaiting_user".into(),
                            },
                        );
                        emit(
                            &app,
                            AgentEvent::ClarifyQuestion {
                                session_id: session_id.clone(),
                                turn_id: turn_id.clone(),
                                tool_call_id: call.id.clone(),
                                question,
                            },
                        );
                        continue;
                    }
                    Ok(_) => {
                        let value = json!({ "error": "一次只允许一个澄清问题" });
                        persist_tool_result(
                            &state,
                            &app,
                            &session_id,
                            &turn_id,
                            call,
                            false,
                            value.to_string(),
                            0,
                            Vec::new(),
                            working_messages,
                        )?;
                        continue;
                    }
                    Err(err) => {
                        let value = err.to_json_value();
                        persist_tool_result(
                            &state,
                            &app,
                            &session_id,
                            &turn_id,
                            call,
                            false,
                            value.to_string(),
                            0,
                            Vec::new(),
                            working_messages,
                        )?;
                        continue;
                    }
                }
            }

            emit(
                &app,
                AgentEvent::ToolCall {
                    session_id: session_id.clone(),
                    turn_id: turn_id.clone(),
                    id: call.id.clone(),
                    name: call.function.name.clone(),
                    args: args.clone(),
                    status: "running".into(),
                },
            );

            let started = Instant::now();
            let (ok, summary, changed_paths) = if let Some(err) = prebuilt_error {
                let result_json = err.to_string();
                (false, result_json, Vec::new())
            } else {
                match state
                    .tools
                    .execute(&ctx, &app, &call.function.name, args.clone())
                    .await
                {
                    Ok(value) => {
                        let paths = crate::tools::changed_paths::extract_changed_paths(
                            &call.function.name,
                            &args,
                            &value,
                        );
                        (true, value.to_string(), paths)
                    }
                    Err(err) => {
                        let value = err.to_json_value();
                        let text = value.to_string();
                        (false, text, Vec::new())
                    }
                }
            };
            let duration_ms = started.elapsed().as_millis() as i64;

            persist_tool_result(
                &state,
                &app,
                &session_id,
                &turn_id,
                call,
                ok,
                summary,
                duration_ms,
                changed_paths,
                working_messages,
            )?;
        }

        if has_pending_clarify {
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

/// Turn 结束兜底：无论 style_warnings 是否被处理，只要没有未修复的脚本失败
/// （`.skill-run/error.json` 不存在），就清理 `.skill-run/` 临时目录。
#[cfg(test)]
#[path = "loop_runner_tests.rs"]
mod loop_runner_tests;
