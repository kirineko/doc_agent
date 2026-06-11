use crate::agent::provider::openai_compat::{effort_from_str, messages_from_store, model_from_str};
use crate::agent::provider::provider_for;
use crate::agent::session_title::{
    is_autotitle_eligible_user_count, is_default_session_title, summarize_session_title,
};
use crate::agent::tool_args::{parse_tool_arguments, truncation_error};
use crate::agent::types::{
    AgentEvent, ChatMessage, ChatRequest, ModelId, ThinkingConfig, ToolCall,
};
use crate::core::sandbox::Sandbox;
use crate::core::store::Message;
use crate::state::AppState;
use crate::tools::ToolContext;
use serde_json::Value;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

const MAX_TOOL_STEPS: usize = 32;

pub async fn run_turn(
    app: AppHandle,
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
        store
            .add_message(&session_id, "user", Some(&user_text), None, None)
            .map_err(|e| e.to_string())?;
    }

    let user_count = history.iter().filter(|m| m.role == "user").count() + 1;

    let sandbox = Sandbox::new(&project.root_path).map_err(|e| e.to_string())?;
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
    let web_enabled = state
        .secrets
        .has_api_key("tavily")
        .map_err(|e| e.to_string())?;
    let tool_defs = state.tools.definitions(web_enabled);
    let mut working_messages =
        build_working_messages(&history, &tool_call_history, &user_text, web_enabled);

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
            let (ok, summary, result_json, changed_paths) = if let Some(err) = prebuilt_error {
                let result_json = err.to_string();
                (false, result_json.clone(), result_json, Vec::new())
            } else {
                match state
                    .tools
                    .execute(&ctx, &call.function.name, args.clone())
                    .await
                {
                    Ok(value) => {
                        let paths = crate::tools::changed_paths::extract_changed_paths(
                            &call.function.name,
                            &args,
                            &value,
                        );
                        (true, value.to_string(), value.to_string(), paths)
                    }
                    Err(err) => {
                        let value = err.to_json_value();
                        let text = value.to_string();
                        (false, text.clone(), text, Vec::new())
                    }
                }
            };
            let duration_ms = started.elapsed().as_millis() as i64;

            {
                let store = state.store.lock().map_err(|e| e.to_string())?;
                store
                    .finish_tool_call(
                        &call.id,
                        &result_json,
                        if ok { "done" } else { "error" },
                        duration_ms,
                    )
                    .map_err(|e| e.to_string())?;
                store
                    .add_message(
                        &session_id,
                        "tool",
                        Some(&result_json),
                        None,
                        Some(&call.id),
                    )
                    .map_err(|e| e.to_string())?;
            }

            emit(
                &app,
                AgentEvent::ToolResult {
                    session_id: session_id.clone(),
                    turn_id: turn_id.clone(),
                    id: call.id.clone(),
                    ok,
                    summary,
                    duration_ms,
                    changed_paths,
                },
            );

            working_messages.push(ChatMessage {
                role: "tool".into(),
                content: Some(result_json),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: Some(call.id.clone()),
            });
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
fn cleanup_skill_run_tmp(sandbox: &Sandbox) {
    let ctx = ToolContext::new(sandbox);
    crate::tools::skill_run_tmp::cleanup_on_turn_end(&ctx);
}

fn build_working_messages(
    history: &[Message],
    tool_calls: &[crate::core::store::ToolCallRecord],
    user_text: &str,
    web_enabled: bool,
) -> Vec<ChatMessage> {
    let mut messages = messages_from_store(history, tool_calls);
    if !messages.iter().any(|m| m.role == "system") {
        let web_hint = if web_enabled {
            "\nWeb 搜索已启用：需要项目外实时信息时用 web_search(query)；已知 URL 需读正文时用 web_extract(urls)。\n"
        } else {
            ""
        };
        messages.insert(
            0,
            ChatMessage {
                role: "system".into(),
                content: Some(format!(
                    "You are doc-agent, an office document assistant.\n\
                     用户消息中 `@路径` 指代项目内文件，可直接用 fs / office 工具读取。{web_hint}\n\
                     生成 .docx/.pptx/.xlsx 交付物前，MUST 先 skill_read 对应 skill 获取规范；\
                     不得凭记忆直接编写 skill_run 代码。\n{}",
                    crate::core::skills::index_markdown()
                )),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            },
        );
    }
    if messages
        .last()
        .map(|m| m.role.as_str() == "user" && m.content.as_deref() == Some(user_text))
        .unwrap_or(false)
    {
        return messages;
    }
    messages.push(ChatMessage {
        role: "user".into(),
        content: Some(user_text.to_string()),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
    });
    messages
}

fn persist_assistant(
    state: &AppState,
    session_id: &str,
    content: Option<&str>,
    reasoning_content: Option<&str>,
    tool_calls: Option<&[ToolCall]>,
) -> Result<Message, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let msg = store
        .add_message(session_id, "assistant", content, reasoning_content, None)
        .map_err(|e| e.to_string())?;
    if let Some(calls) = tool_calls {
        for call in calls {
            store
                .add_tool_call(
                    &msg.id,
                    &call.id,
                    &call.function.name,
                    &call.function.arguments,
                )
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(msg)
}

fn maybe_autotitle_session(
    state: &AppState,
    session_id: &str,
    session_title: &str,
    user_count: usize,
    user_text: &str,
    assistant_text: Option<&str>,
) -> Result<(), String> {
    if !is_default_session_title(session_title) || !is_autotitle_eligible_user_count(user_count) {
        return Ok(());
    }

    let title = if user_count == 2 {
        summarize_session_title(user_text, None)
    } else {
        summarize_session_title(user_text, assistant_text)
    };
    let Some(title) = title else {
        return Ok(());
    };

    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .update_session(session_id, Some(&title), None, None, None)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn emit(app: &AppHandle, event: AgentEvent) {
    let _ = app.emit("agent-event", event);
}

fn emit_assistant_step_done(app: &AppHandle, session_id: &str, turn_id: &str, message: &Message) {
    emit(
        app,
        AgentEvent::AssistantStepDone {
            session_id: session_id.to_string(),
            turn_id: turn_id.to_string(),
            message: message.clone(),
        },
    );
}

#[cfg(test)]
mod tests {
    use crate::agent::types::AgentEvent;
    use crate::core::store::{Message, Store};
    use tempfile::tempdir;

    #[test]
    fn assistant_step_done_event_serializes() {
        let event = AgentEvent::AssistantStepDone {
            session_id: "s1".into(),
            turn_id: "t1".into(),
            message: Message {
                id: "m1".into(),
                session_id: "s1".into(),
                role: "assistant".into(),
                content: Some("answer".into()),
                reasoning_content: Some("thought".into()),
                tool_call_id: None,
                seq: 1,
                created_at: "2026-01-01".into(),
            },
        };
        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(value["kind"], "assistant_step_done");
        assert_eq!(value["message"]["id"], "m1");
    }

    #[test]
    fn reasoning_content_is_persisted_with_assistant() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store
            .create_project("demo", dir.path().to_str().unwrap())
            .unwrap();
        let session = store
            .create_session(&project.id, "s1", "mock", true, "high")
            .unwrap();
        store
            .add_message(
                &session.id,
                "assistant",
                Some("answer"),
                Some("thought"),
                None,
            )
            .unwrap();
        let messages = store.list_messages(&session.id).unwrap();
        assert_eq!(messages[0].reasoning_content.as_deref(), Some("thought"));
    }
}
