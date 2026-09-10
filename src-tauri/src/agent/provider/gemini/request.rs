use super::state::GoogleReplayState;
use crate::agent::model_config::{capped_output_budget, validate_thinking};
use crate::agent::provider::ProviderError;
use crate::agent::types::ChatRequest;
use serde_json::Value;

pub const GOOGLE_CHAT_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions";
pub const MAX_REQUEST_BYTES: usize = 20_000_000;

pub fn adapt_request(body: &mut Value, request: &ChatRequest) -> Result<(), ProviderError> {
    validate_thinking(
        request.model.info(),
        request.thinking.enabled,
        request.thinking.effort,
    )
    .map_err(ProviderError::Parse)?;
    if let Some(limit) = request.max_tokens {
        capped_output_budget(request.model, limit).map_err(ProviderError::Parse)?;
    }
    let wire_messages = body["messages"]
        .as_array_mut()
        .ok_or_else(|| ProviderError::Parse("Gemini messages 必须为数组".into()))?;
    for (wire, message) in wire_messages.iter_mut().zip(&request.messages) {
        wire.as_object_mut().unwrap().remove("reasoning_content");
        if message.role != "assistant" {
            continue;
        }
        let calls = message.tool_calls.as_deref().unwrap_or_default();
        let state = match message.provider_state.as_deref() {
            Some(raw) => GoogleReplayState::parse(raw, request.model.api_model())
                .map_err(ProviderError::Parse)?,
            None if calls.is_empty() => continue,
            None => return Err(ProviderError::Parse("Gemini 工具历史缺少签名元数据".into())),
        };
        state.validate_calls(calls).map_err(ProviderError::Parse)?;
        if state
            .message_extra_content
            .as_object()
            .is_some_and(|m| !m.is_empty())
        {
            wire["extra_content"] = state.message_extra_content;
        }
        if let Some(wire_calls) = wire["tool_calls"].as_array_mut() {
            for (call, extra) in wire_calls.iter_mut().zip(state.tool_extra_content) {
                if extra.as_object().is_some_and(|m| !m.is_empty()) {
                    call["extra_content"] = extra;
                }
            }
        }
    }
    if request.tools.is_empty() {
        body.as_object_mut().unwrap().remove("tools");
    }
    let bytes =
        serde_json::to_vec(body).map_err(|_| ProviderError::Parse("Gemini 请求编码失败".into()))?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(ProviderError::Parse(format!(
            "Gemini 请求超过 {MAX_REQUEST_BYTES} 字节，请缩小/移除图片或压缩历史。"
        )));
    }
    Ok(())
}
