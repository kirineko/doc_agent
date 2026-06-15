use crate::agent::provider::openai_compat::{
    encode_attachment_data_url, messages_from_store, messages_from_store_text,
};
use crate::agent::session_title::{
    is_autotitle_eligible_user_count, is_default_session_title, summarize_session_title,
};
use crate::agent::types::{AgentEvent, ChatMessage, MessageAttachment, ToolCall};
use crate::core::sandbox::Sandbox;
use crate::core::store::Message;
use crate::state::AppState;
use crate::tools::ToolContext;
use std::collections::HashSet;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Runtime};
use uuid::Uuid;

pub(super) fn cleanup_skill_run_tmp(sandbox: &Sandbox) {
    let ctx = ToolContext::new(sandbox);
    crate::tools::skill_run_tmp::cleanup_on_turn_end(&ctx);
}

pub(crate) fn build_working_messages(
    history: &[Message],
    tool_calls: &[crate::core::store::ToolCallRecord],
    user_text: Option<&str>,
    user_attachments: &[MessageAttachment],
    web_enabled: bool,
    sandbox: Option<&Sandbox>,
) -> Result<Vec<ChatMessage>, String> {
    let mut messages = if let Some(sandbox) = sandbox {
        messages_from_store(history, tool_calls, Some(sandbox))?
    } else {
        messages_from_store_text(history, tool_calls)
    };
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
                     需求不明确时（新建或编辑文档均适用），MUST 先 skill_read clarify 按流程澄清；\
                     澄清问题 MUST 通过 clarify_ask 工具逐问提出，禁止以纯文本罗列问题。\
                     生成 .docx/.pptx/.xlsx 交付物前，MUST 先 skill_read 对应 skill 获取规范；\
                     生成静态 HTML 报告前，MUST 先 skill_read html-report；\
                     html_to_pdf 可单独使用，不要求先生成报告；\
                     Typst PDF：调用 typst_to_pdf、typst_list_templates、typst_read_template（场景模板），或 fs_write/fs_patch 编写 .typ 前，\
                     同一会话 MUST 先 typst_read_template syntax/typst-guide；不得凭记忆臆造 Typst/LaTeX 语法。\
                     公式密集文档优先 typst_to_pdf；图文 HTML 报告可用 html_to_pdf。\
                     读取 PDF 内容：默认 pdf_read({{\"path\":\"...\"}})（所有模型；vision 模型内部 Judge，纯文本书快速返回文本）；\
                     仅当明确只要 PDFium 纯文本、跳过 Judge 时用 office_read_to_markdown。pdf_read 仅 path/pages/dpi。\
                     不得凭记忆直接编写 skill_run 代码。\n{}",
                    crate::core::skills::index_markdown()
                )),
                image_urls: vec![],
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            },
        );
    }
    let Some(user_text) = user_text else {
        return Ok(messages);
    };
    let image_urls = if user_attachments.is_empty() {
        Vec::new()
    } else {
        let sandbox = sandbox.ok_or_else(|| "attachments require project sandbox".to_string())?;
        user_attachments
            .iter()
            .map(|attachment| encode_attachment_data_url(sandbox, attachment).map(Arc::<str>::from))
            .collect::<Result<Vec<_>, _>>()?
    };
    if !messages
        .last()
        .map(|m| {
            m.role.as_str() == "user"
                && m.content.as_deref() == Some(user_text)
                && m.image_urls == image_urls
        })
        .unwrap_or(false)
    {
        messages.push(ChatMessage {
            role: "user".into(),
            content: Some(user_text.to_string()),
            image_urls,
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        });
    }
    Ok(messages)
}

/// 保证 tool call id 非空、批内唯一，且不与 DB 已有记录冲突。
/// 部分模型（如 Kimi）流式响应可能省略 `id` 或复用短 id，直接入库会触发 UNIQUE 约束。
pub(super) fn normalize_tool_call_ids(calls: &mut [ToolCall], id_exists: impl Fn(&str) -> bool) {
    let mut seen = HashSet::new();
    for call in calls.iter_mut() {
        if call.function.name.is_empty() {
            continue;
        }
        let needs_new = call.id.is_empty() || seen.contains(&call.id) || id_exists(&call.id);
        if needs_new {
            loop {
                let candidate = format!("call_{}", Uuid::new_v4());
                if !seen.contains(&candidate) && !id_exists(&candidate) {
                    call.id = candidate;
                    break;
                }
            }
        }
        seen.insert(call.id.clone());
    }
}

pub(super) fn persist_assistant(
    state: &AppState,
    session_id: &str,
    content: Option<&str>,
    reasoning_content: Option<&str>,
    tool_calls: Option<&[ToolCall]>,
) -> Result<Message, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let msg = store
        .add_message(
            session_id,
            "assistant",
            content,
            reasoning_content,
            None,
            None,
        )
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

pub(super) fn persist_clarify_pending(
    state: &AppState,
    session_id: &str,
    turn_id: &str,
    tool_call_id: &str,
    question_json: &str,
) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .update_tool_call_args(tool_call_id, question_json)
        .map_err(|e| e.to_string())?;
    store
        .update_tool_call_status(tool_call_id, "awaiting_user")
        .map_err(|e| e.to_string())?;
    store
        .save_clarify_pending(session_id, turn_id, tool_call_id, question_json)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn persist_tool_result<R: Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    session_id: &str,
    turn_id: &str,
    call: &ToolCall,
    ok: bool,
    summary: String,
    duration_ms: i64,
    changed_paths: Vec<String>,
    working_messages: &mut Vec<ChatMessage>,
) -> Result<(), String> {
    let status = if ok { "done" } else { "error" };
    {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .finish_tool_call(&call.id, &summary, status, duration_ms)
            .map_err(|e| e.to_string())?;
        store
            .add_message(
                session_id,
                "tool",
                Some(&summary),
                None,
                Some(&call.id),
                None,
            )
            .map_err(|e| e.to_string())?;
    }

    emit(
        app,
        AgentEvent::ToolResult {
            session_id: session_id.to_string(),
            turn_id: turn_id.to_string(),
            id: call.id.clone(),
            ok,
            summary: summary.clone(),
            duration_ms,
            changed_paths,
        },
    );

    working_messages.push(ChatMessage {
        role: "tool".into(),
        content: Some(summary),
        image_urls: vec![],
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: Some(call.id.clone()),
    });
    Ok(())
}

pub(super) fn maybe_autotitle_session(
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

pub(super) fn emit<R: Runtime>(app: &AppHandle<R>, event: AgentEvent) {
    let _ = app.emit("agent-event", event);
}

pub(super) fn emit_assistant_step_done<R: Runtime>(
    app: &AppHandle<R>,
    session_id: &str,
    turn_id: &str,
    message: &Message,
) {
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
    use super::*;
    use crate::agent::types::FunctionCall;

    fn sample_call(id: &str, name: &str) -> ToolCall {
        ToolCall {
            id: id.to_string(),
            call_type: "function".into(),
            function: FunctionCall {
                name: name.to_string(),
                arguments: "{}".into(),
            },
        }
    }

    #[test]
    fn normalize_fills_empty_ids() {
        let mut calls = vec![sample_call("", "pdf_read")];
        normalize_tool_call_ids(&mut calls, |_| false);
        assert!(!calls[0].id.is_empty());
    }

    #[test]
    fn normalize_replaces_duplicate_ids_in_batch() {
        let mut calls = vec![
            sample_call("dup", "pdf_read"),
            sample_call("dup", "fs_list"),
        ];
        normalize_tool_call_ids(&mut calls, |_| false);
        assert_ne!(calls[0].id, calls[1].id);
    }

    #[test]
    fn normalize_replaces_existing_db_ids() {
        let mut calls = vec![sample_call("taken", "pdf_read")];
        normalize_tool_call_ids(&mut calls, |id| id == "taken");
        assert_ne!(calls[0].id, "taken");
    }
}
