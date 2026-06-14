use crate::agent::loop_support::build_working_messages;
use crate::agent::provider::provider_for;
use crate::agent::types::{
    AgentEvent, ChatMessage, ChatRequest, ModelId, ThinkingConfig, ThinkingEffort,
};
use crate::core::store::{Message, ToolCallRecord};
use crate::state::AppState;
use tauri::{AppHandle, Emitter, Runtime};

pub const COMPACTION_TRIGGER_RATIO: f64 = 0.85;
pub const MAX_PRESERVED_MESSAGES: usize = 2;
pub const MAX_TOOL_STEPS: usize = 64;

const COMPACT_PROMPT: &str = include_str!("prompts/compact.md");
const SUMMARY_PREFIX: &str = "Previous context has been compacted. Continue from this summary:\n\n";

pub struct CompactionOutcome {
    pub compacted: bool,
    pub before_tokens: u32,
    pub after_tokens: u32,
}

pub fn reserved_context_size(max_context: u32) -> u32 {
    50_000.max(max_context / 10)
}

pub fn should_auto_compact(
    token_count: u32,
    max_context_size: u32,
    trigger_ratio: f64,
    reserved_context_size: u32,
) -> bool {
    if token_count == 0 || max_context_size == 0 {
        return false;
    }
    let token_count = token_count as f64;
    let max_context = max_context_size as f64;
    token_count >= max_context * trigger_ratio
        || token_count + reserved_context_size as f64 >= max_context
}

pub fn estimate_text_tokens(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    ((text.chars().count() / 4).max(1)) as u32
}

pub fn estimate_chat_message_tokens(message: &ChatMessage) -> u32 {
    let mut total = estimate_text_tokens(message.content.as_deref().unwrap_or(""))
        + estimate_text_tokens(message.reasoning_content.as_deref().unwrap_or(""));
    if let Some(calls) = &message.tool_calls {
        for call in calls {
            total += estimate_text_tokens(&call.function.name)
                + estimate_text_tokens(&call.function.arguments);
        }
    }
    total
}

pub fn estimate_chat_messages_tokens(messages: &[ChatMessage]) -> u32 {
    messages.iter().map(estimate_chat_message_tokens).sum()
}

pub fn estimate_store_message_tokens(message: &Message, tool_calls: &[ToolCallRecord]) -> u32 {
    estimate_store_messages_tokens(std::slice::from_ref(message), tool_calls)
}

pub fn estimate_store_messages_tokens(messages: &[Message], tool_calls: &[ToolCallRecord]) -> u32 {
    let chat = crate::agent::provider::openai_compat::messages_from_store(messages, tool_calls);
    estimate_chat_messages_tokens(&chat)
}

pub struct PreparedCompaction<'a> {
    pub to_compact: &'a [Message],
    pub to_preserve: &'a [Message],
}

pub fn prepare_compaction_split<'a>(
    messages: &'a [Message],
    tool_calls: &[ToolCallRecord],
    max_preserved_messages: usize,
) -> Option<PreparedCompaction<'a>> {
    if messages.is_empty() || max_preserved_messages == 0 {
        return None;
    }

    let mut preserve_start = messages.len();
    let mut seen = 0usize;
    for (index, message) in messages.iter().enumerate().rev() {
        if is_conversation_message(message) {
            seen += 1;
            if seen == max_preserved_messages {
                preserve_start = index;
                break;
            }
        }
    }
    if seen < max_preserved_messages {
        return None;
    }

    preserve_start = expand_preserve_start_for_tool_group(messages, tool_calls, preserve_start);
    let to_compact = &messages[..preserve_start];
    let to_preserve = &messages[preserve_start..];
    if to_compact.is_empty() {
        return None;
    }
    Some(PreparedCompaction {
        to_compact,
        to_preserve,
    })
}

fn is_conversation_message(message: &Message) -> bool {
    matches!(message.role.as_str(), "user" | "assistant") && !is_compaction_summary(message)
}

fn is_compaction_summary(message: &Message) -> bool {
    message.role == "user"
        && message
            .content
            .as_deref()
            .is_some_and(|content| content.starts_with(SUMMARY_PREFIX))
}

fn expand_preserve_start_for_tool_group(
    messages: &[Message],
    tool_calls: &[ToolCallRecord],
    mut preserve_start: usize,
) -> usize {
    while preserve_start > 0 && messages[preserve_start - 1].role == "tool" {
        preserve_start -= 1;
    }

    let mut needed_call_ids = std::collections::HashSet::new();
    for message in &messages[preserve_start..] {
        if message.role == "tool" {
            if let Some(call_id) = message.tool_call_id.as_deref() {
                needed_call_ids.insert(call_id.to_string());
            }
        }
    }
    if needed_call_ids.is_empty() {
        return preserve_start;
    }

    for index in (0..preserve_start).rev() {
        let message = &messages[index];
        if message.role == "assistant" {
            let calls_for_message: Vec<&str> = tool_calls
                .iter()
                .filter(|record| record.message_id == message.id)
                .map(|record| record.id.as_str())
                .collect();
            if calls_for_message
                .iter()
                .any(|id| needed_call_ids.contains(*id))
            {
                preserve_start = index;
                for id in calls_for_message {
                    needed_call_ids.remove(id);
                }
                if needed_call_ids.is_empty() {
                    break;
                }
            }
        } else if message.role == "tool" {
            if let Some(call_id) = message.tool_call_id.as_deref() {
                if needed_call_ids.contains(call_id) {
                    preserve_start = index;
                    needed_call_ids.remove(call_id);
                    if needed_call_ids.is_empty() {
                        break;
                    }
                }
            }
        }
    }
    preserve_start
}

fn build_compact_input(to_compact: &[Message], tool_calls: &[ToolCallRecord]) -> String {
    let chat = crate::agent::provider::openai_compat::messages_from_store(to_compact, tool_calls);
    let mut body = String::new();
    for (index, message) in chat.iter().enumerate() {
        body.push_str(&format!(
            "## Message {}\nRole: {}\nContent:\n",
            index + 1,
            message.role
        ));
        if let Some(content) = &message.content {
            body.push_str(content);
            body.push('\n');
        }
        if let Some(reasoning) = &message.reasoning_content {
            body.push_str("Reasoning:\n");
            body.push_str(reasoning);
            body.push('\n');
        }
        if let Some(calls) = &message.tool_calls {
            for call in calls {
                body.push_str(&format!(
                    "ToolCall {}({}): {}\n",
                    call.id, call.function.name, call.function.arguments
                ));
            }
        }
    }
    body.push_str(COMPACT_PROMPT);
    truncate_for_compaction_input(&mut body, 200_000);
    body
}

fn truncate_for_compaction_input(body: &mut String, max_chars: usize) {
    if body.len() <= max_chars {
        return;
    }
    let head = max_chars / 2;
    let tail = max_chars - head - 64;
    let omitted = body.len() - head - tail;
    let suffix = body.split_off(head);
    body.push_str("\n\n[... omitted ");
    body.push_str(&omitted.to_string());
    body.push_str(" chars ...]\n\n");
    body.push_str(&suffix[suffix.len().saturating_sub(tail)..]);
}

pub fn build_summary_message_content(summary: &str) -> String {
    format!("{SUMMARY_PREFIX}{}", summary.trim())
}

pub fn estimate_compacted_tokens(
    summary_content: &str,
    preserved: &[Message],
    tool_calls: &[ToolCallRecord],
) -> u32 {
    estimate_text_tokens(summary_content) + estimate_store_messages_tokens(preserved, tool_calls)
}

#[allow(clippy::too_many_arguments)]
pub async fn compact_session_if_needed<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    session_id: &str,
    turn_id: &str,
    model: ModelId,
    api_key: Option<&str>,
    token_count: u32,
    pending_estimate: u32,
    web_enabled: bool,
) -> Result<(Vec<ChatMessage>, u32, u32, Option<CompactionOutcome>), String> {
    let effective = token_count.saturating_add(pending_estimate);
    let max_context = model.max_context_size();
    let reserved = reserved_context_size(max_context);
    if !should_auto_compact(effective, max_context, COMPACTION_TRIGGER_RATIO, reserved) {
        return Ok((Vec::new(), token_count, pending_estimate, None));
    }

    let (history, tool_call_history) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let history = store
            .list_active_messages(session_id)
            .map_err(|e| e.to_string())?;
        let tool_call_history = store
            .list_tool_calls_for_session(session_id)
            .map_err(|e| e.to_string())?;
        (history, tool_call_history)
    };

    let before_tokens = effective;
    let prepared =
        match prepare_compaction_split(&history, &tool_call_history, MAX_PRESERVED_MESSAGES) {
            Some(prepared) => prepared,
            None => {
                truncate_fallback(
                    state,
                    session_id,
                    &history,
                    &tool_call_history,
                    max_context,
                    reserved,
                )?;
                let working = rebuild_working_messages(state, session_id, web_enabled)?;
                let after = estimate_chat_messages_tokens(&working);
                emit_context_compacted(app, session_id, before_tokens, after);
                emit_context_usage(app, session_id, after, max_context);
                return Ok((
                    working,
                    after,
                    0,
                    Some(CompactionOutcome {
                        compacted: true,
                        before_tokens,
                        after_tokens: after,
                    }),
                ));
            }
        };

    let compact_input = build_compact_input(prepared.to_compact, &tool_call_history);
    let summary =
        match run_compaction_llm(session_id, turn_id, model, api_key, &compact_input).await {
            Ok(summary) => summary,
            Err(_) => {
                truncate_fallback_compact_only(
                    state,
                    session_id,
                    prepared.to_compact,
                    &tool_call_history,
                    max_context,
                    reserved,
                )?;
                let working = rebuild_working_messages(state, session_id, web_enabled)?;
                let after = estimate_chat_messages_tokens(&working);
                emit_context_compacted(app, session_id, before_tokens, after);
                emit_context_usage(app, session_id, after, max_context);
                return Ok((
                    working,
                    after,
                    0,
                    Some(CompactionOutcome {
                        compacted: true,
                        before_tokens,
                        after_tokens: after,
                    }),
                ));
            }
        };

    let archive_ids: Vec<String> = prepared.to_compact.iter().map(|m| m.id.clone()).collect();
    let summary_content = build_summary_message_content(&summary);
    let insert_before_seq = prepared
        .to_preserve
        .first()
        .map(|m| m.seq)
        .ok_or_else(|| "compaction preserve segment is empty".to_string())?;

    {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .mark_messages_archived(&archive_ids)
            .map_err(|e| e.to_string())?;
        store
            .add_compaction_summary(session_id, &summary_content, insert_before_seq)
            .map_err(|e| e.to_string())?;
    }

    let working = rebuild_working_messages(state, session_id, web_enabled)?;
    let after_tokens = estimate_chat_messages_tokens(&working);

    {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .set_session_token_count(session_id, after_tokens)
            .map_err(|e| e.to_string())?;
    }

    emit_context_compacted(app, session_id, before_tokens, after_tokens);
    emit_context_usage(app, session_id, after_tokens, max_context);

    Ok((
        working,
        after_tokens,
        0,
        Some(CompactionOutcome {
            compacted: true,
            before_tokens,
            after_tokens,
        }),
    ))
}

fn rebuild_working_messages(
    state: &AppState,
    session_id: &str,
    web_enabled: bool,
) -> Result<Vec<ChatMessage>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let history = store
        .list_active_messages(session_id)
        .map_err(|e| e.to_string())?;
    let tool_call_history = store
        .list_tool_calls_for_session(session_id)
        .map_err(|e| e.to_string())?;
    Ok(build_working_messages(
        &history,
        &tool_call_history,
        None,
        web_enabled,
    ))
}

fn fallback_preserve_start(messages: &[Message], tool_calls: &[ToolCallRecord]) -> usize {
    if messages.is_empty() {
        return 0;
    }
    if let Some(prepared) = prepare_compaction_split(messages, tool_calls, MAX_PRESERVED_MESSAGES) {
        return messages
            .iter()
            .position(|m| m.id == prepared.to_preserve[0].id)
            .unwrap_or(messages.len());
    }
    let mut preserve_start = messages.len().saturating_sub(1);
    if let Some(summary_index) = messages.iter().rposition(is_compaction_summary) {
        preserve_start = preserve_start.min(summary_index);
    }
    expand_preserve_start_for_tool_group(messages, tool_calls, preserve_start)
}

async fn run_compaction_llm(
    session_id: &str,
    turn_id: &str,
    model: ModelId,
    api_key: Option<&str>,
    compact_input: &str,
) -> Result<String, String> {
    let provider = provider_for(model);
    let request = ChatRequest {
        session_id: session_id.to_string(),
        turn_id: turn_id.to_string(),
        model,
        messages: vec![
            ChatMessage {
                role: "system".into(),
                content: Some(
                    "You are a helpful assistant that compacts conversation context. Do not call tools."
                        .into(),
                ),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: "user".into(),
                content: Some(compact_input.to_string()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        tools: vec![],
        thinking: ThinkingConfig {
            enabled: false,
            effort: ThinkingEffort::High,
        },
        response_format: None,
        max_tokens: Some(8192),
    };

    let turn = provider
        .chat_stream(request, api_key, &mut |_| {})
        .await
        .map_err(|e| e.to_string())?;
    let summary = turn.content.trim();
    if summary.is_empty() {
        return Err("compaction produced empty summary".into());
    }
    Ok(summary.to_string())
}

fn truncate_fallback(
    state: &AppState,
    session_id: &str,
    history: &[Message],
    tool_calls: &[ToolCallRecord],
    max_context: u32,
    reserved: u32,
) -> Result<(), String> {
    let preserve_start = fallback_preserve_start(history, tool_calls);
    truncate_from_candidates(
        state,
        session_id,
        history,
        tool_calls,
        preserve_start,
        max_context,
        reserved,
    )
}

fn truncate_fallback_compact_only(
    state: &AppState,
    session_id: &str,
    to_compact: &[Message],
    tool_calls: &[ToolCallRecord],
    max_context: u32,
    reserved: u32,
) -> Result<(), String> {
    truncate_from_candidates(
        state,
        session_id,
        to_compact,
        tool_calls,
        0,
        max_context,
        reserved,
    )
}

fn expand_archive_ids_for_tool_pairs(
    candidates: &[Message],
    tool_calls: &[ToolCallRecord],
    preserve_start: usize,
    to_archive: Vec<String>,
) -> Vec<String> {
    let mut archive_set: std::collections::HashSet<String> = to_archive.into_iter().collect();
    for message in candidates.iter().take(preserve_start) {
        if message.role != "assistant" || !archive_set.contains(&message.id) {
            continue;
        }
        let call_ids: std::collections::HashSet<&str> = tool_calls
            .iter()
            .filter(|record| record.message_id == message.id)
            .map(|record| record.id.as_str())
            .collect();
        if call_ids.is_empty() {
            continue;
        }
        for tool_message in candidates.iter().take(preserve_start) {
            if tool_message.role != "tool" {
                continue;
            }
            if let Some(call_id) = tool_message.tool_call_id.as_deref() {
                if call_ids.contains(call_id) {
                    archive_set.insert(tool_message.id.clone());
                }
            }
        }
    }
    archive_set.into_iter().collect()
}

fn truncate_from_candidates(
    state: &AppState,
    session_id: &str,
    candidates: &[Message],
    tool_calls: &[ToolCallRecord],
    preserve_start: usize,
    max_context: u32,
    reserved: u32,
) -> Result<(), String> {
    let target = max_context.saturating_sub(reserved);
    let mut remaining = estimate_store_messages_tokens(candidates, tool_calls);
    let mut to_archive = Vec::new();
    for (index, message) in candidates.iter().enumerate() {
        if index >= preserve_start {
            break;
        }
        if remaining <= target {
            break;
        }
        to_archive.push(message.id.clone());
        remaining = remaining.saturating_sub(estimate_store_message_tokens(message, tool_calls));
    }
    to_archive =
        expand_archive_ids_for_tool_pairs(candidates, tool_calls, preserve_start, to_archive);
    if to_archive.is_empty() {
        return Ok(());
    }
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .mark_messages_archived(&to_archive)
        .map_err(|e| e.to_string())?;
    store
        .set_session_token_count(session_id, target.min(remaining))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn emit_context_usage<R: Runtime>(
    app: &AppHandle<R>,
    session_id: &str,
    used_tokens: u32,
    max_tokens: u32,
) {
    let ratio = if max_tokens == 0 {
        0.0
    } else {
        used_tokens as f64 / max_tokens as f64
    };
    let _ = app.emit(
        "agent-event",
        AgentEvent::ContextUsage {
            session_id: session_id.to_string(),
            used_tokens,
            max_tokens,
            ratio,
        },
    );
}

pub fn emit_context_compacted<R: Runtime>(
    app: &AppHandle<R>,
    session_id: &str,
    before_tokens: u32,
    after_tokens: u32,
) {
    let _ = app.emit(
        "agent-event",
        AgentEvent::ContextCompacted {
            session_id: session_id.to_string(),
            before_tokens,
            after_tokens,
        },
    );
}

#[cfg(test)]
#[path = "compaction_tests.rs"]
mod compaction_tests;
