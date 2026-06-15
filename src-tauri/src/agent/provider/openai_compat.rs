use crate::agent::model_catalog::ProviderKind;
use crate::agent::provider::{sse, ProviderError};
use crate::agent::types::{
    AgentEvent, AssistantTurn, ChatMessage, ChatRequest, MessageAttachment, ModelId, ThinkingEffort,
};
use crate::core::sandbox::Sandbox;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::Client;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

pub const MAX_ATTACHMENTS_PER_MESSAGE: usize = 4;
pub const MAX_ATTACHMENT_BYTES: u64 = 50 * 1024 * 1024;

pub struct OpenAiCompatClient {
    pub base_url: String,
    pub client: Client,
}

impl OpenAiCompatClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    pub async fn stream_chat(
        &self,
        request: ChatRequest,
        api_key: &str,
        extra_body: Value,
        session_id: &str,
        turn_id: &str,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<AssistantTurn, ProviderError> {
        let mut body = json!({
            "model": request.model.api_model(),
            "messages": request.messages,
            "tools": request.tools.iter().map(|t| {
                let mut function = json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                });
                if t.strict == Some(true) {
                    if let Some(obj) = function.as_object_mut() {
                        obj.insert("strict".into(), json!(true));
                    }
                }
                json!({
                    "type": "function",
                    "function": function,
                })
            }).collect::<Vec<_>>(),
            "stream": true,
            "stream_options": { "include_usage": true },
        });

        if let Some(obj) = body.as_object_mut() {
            if let Some(response_format) = request.response_format {
                obj.insert("response_format".into(), response_format);
            }
            if let Some(limit) = request.max_tokens {
                apply_output_token_limit(obj, request.model.provider_kind(), limit);
            }
            if let Some(extra) = extra_body.as_object() {
                for (k, v) in extra {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        let response = self
            .client
            .post(format!(
                "{}/v1/chat/completions",
                self.base_url.trim_end_matches('/')
            ))
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::Http(format!("{status}: {text}")));
        }

        let mut tool_tracker = sse::ToolStreamTracker::new();
        sse::consume_openai_sse(response, |reasoning, content, tools| {
            if let Some(delta) = reasoning {
                on_event(AgentEvent::ReasoningToken {
                    session_id: session_id.to_string(),
                    turn_id: turn_id.to_string(),
                    delta: delta.to_string(),
                });
            }
            if let Some(delta) = content {
                on_event(AgentEvent::ContentToken {
                    session_id: session_id.to_string(),
                    turn_id: turn_id.to_string(),
                    delta: delta.to_string(),
                });
            }
            if let Some(items) = tools {
                if let Some((index, name, args_chars)) = tool_tracker.update(items) {
                    on_event(AgentEvent::ToolCallStream {
                        session_id: session_id.to_string(),
                        turn_id: turn_id.to_string(),
                        index,
                        name,
                        args_chars,
                    });
                }
            }
        })
        .await
        .map_err(|e| ProviderError::Parse(e.to_string()))
    }

    pub async fn complete_chat(
        &self,
        request: ChatRequest,
        api_key: &str,
        extra_body: Value,
    ) -> Result<AssistantTurn, ProviderError> {
        let mut body = json!({
            "model": request.model.api_model(),
            "messages": request.messages,
            "stream": false,
        });

        if let Some(obj) = body.as_object_mut() {
            if let Some(response_format) = request.response_format {
                obj.insert("response_format".into(), response_format);
            }
            if let Some(limit) = request.max_tokens {
                apply_output_token_limit(obj, request.model.provider_kind(), limit);
            }
            if let Some(extra) = extra_body.as_object() {
                for (k, v) in extra {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        let response = self
            .client
            .post(format!(
                "{}/v1/chat/completions",
                self.base_url.trim_end_matches('/')
            ))
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::Http(format!("{status}: {text}")));
        }

        let payload: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        parse_non_stream_turn(&payload)
    }
}

fn parse_non_stream_turn(payload: &Value) -> Result<AssistantTurn, ProviderError> {
    use crate::agent::types::{FunctionCall, TokenUsage, ToolCall};

    let choice = payload["choices"]
        .as_array()
        .and_then(|items| items.first())
        .ok_or_else(|| ProviderError::Parse("missing choices".into()))?;
    let message = &choice["message"];
    let content = message["content"].as_str().unwrap_or_default().to_string();
    let reasoning_content = message["reasoning_content"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let finish_reason = choice["finish_reason"].as_str().map(str::to_string);
    let tool_calls = message["tool_calls"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    Some(ToolCall {
                        id: item["id"].as_str()?.to_string(),
                        call_type: item["type"].as_str().unwrap_or("function").to_string(),
                        function: FunctionCall {
                            name: item["function"]["name"].as_str()?.to_string(),
                            arguments: item["function"]["arguments"].as_str()?.to_string(),
                        },
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let usage = payload["usage"].as_object().map(|usage| TokenUsage {
        prompt: usage
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        completion: usage
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        total: usage
            .get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
    });

    Ok(AssistantTurn {
        content,
        reasoning_content,
        tool_calls,
        finish_reason,
        usage,
    })
}

pub fn apply_output_token_limit(
    body: &mut serde_json::Map<String, Value>,
    provider: ProviderKind,
    limit: u32,
) {
    body.remove("max_tokens");
    body.remove("max_completion_tokens");
    match provider {
        ProviderKind::Deepseek => {
            body.insert("max_tokens".into(), json!(limit));
        }
        ProviderKind::Kimi | ProviderKind::Mimo => {
            body.insert("max_completion_tokens".into(), json!(limit));
        }
        ProviderKind::Mock => {
            body.insert("max_tokens".into(), json!(limit));
        }
    }
}

pub fn thinking_extra_body(request: &ChatRequest, kimi: bool) -> Value {
    if kimi {
        let mut thinking = json!({
            "type": if request.thinking.enabled { "enabled" } else { "disabled" }
        });
        if request.thinking.enabled {
            thinking["keep"] = json!("all");
        }
        json!({ "thinking": thinking })
    } else {
        let mut body = json!({
            "thinking": {
                "type": if request.thinking.enabled { "enabled" } else { "disabled" }
            }
        });
        if request.thinking.enabled && request.model.supports_effort() {
            body["reasoning_effort"] = json!(request.thinking.effort.as_str());
        }
        body
    }
}

pub fn mimo_thinking_extra_body(request: &ChatRequest) -> Value {
    json!({
        "thinking": {
            "type": if request.thinking.enabled { "enabled" } else { "disabled" }
        }
    })
}

pub fn parse_attachments_json(raw: Option<&str>) -> Result<Vec<MessageAttachment>, String> {
    let Some(raw) = raw else {
        return Ok(vec![]);
    };
    if raw.trim().is_empty() {
        return Ok(vec![]);
    }
    serde_json::from_str(raw).map_err(|e| format!("invalid attachments_json: {e}"))
}

pub fn is_upload_attachment_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let trimmed = normalized.trim_start_matches("./");
    trimmed.starts_with(".uploads/") && !trimmed.contains("..")
}

pub fn validate_attachments(attachments: &[MessageAttachment]) -> Result<(), String> {
    if attachments.len() > MAX_ATTACHMENTS_PER_MESSAGE {
        return Err(format!(
            "at most {MAX_ATTACHMENTS_PER_MESSAGE} image attachments per message"
        ));
    }
    for attachment in attachments {
        if !is_upload_attachment_path(&attachment.path) {
            return Err(format!(
                "attachment path must be under .uploads/: {}",
                attachment.path
            ));
        }
        if !is_allowed_image_mime(&attachment.mime) {
            return Err(format!("unsupported image mime: {}", attachment.mime));
        }
    }
    Ok(())
}

pub fn is_allowed_image_mime(mime: &str) -> bool {
    matches!(
        mime,
        "image/png" | "image/jpeg" | "image/webp" | "image/gif"
    )
}

pub fn encode_attachment_data_url(
    sandbox: &Sandbox,
    attachment: &MessageAttachment,
) -> Result<String, String> {
    if !is_allowed_image_mime(&attachment.mime) {
        return Err(format!("unsupported image mime: {}", attachment.mime));
    }
    let resolved = sandbox
        .resolve(&attachment.path)
        .map_err(|e| format!("attachment path error: {e}"))?;
    let metadata = std::fs::metadata(&resolved).map_err(|e| e.to_string())?;
    if metadata.len() > MAX_ATTACHMENT_BYTES {
        return Err(format!(
            "attachment exceeds {}MB limit",
            MAX_ATTACHMENT_BYTES / 1024 / 1024
        ));
    }
    let bytes = std::fs::read(&resolved).map_err(|e| e.to_string())?;
    let encoded = STANDARD.encode(bytes);
    Ok(format!("data:{};base64,{}", attachment.mime, encoded))
}

pub fn messages_from_store(
    messages: &[crate::core::store::Message],
    tool_calls: &[crate::core::store::ToolCallRecord],
    sandbox: Option<&Sandbox>,
) -> Result<Vec<ChatMessage>, String> {
    use crate::agent::types::{FunctionCall, ToolCall};
    use std::collections::HashMap;

    let mut tool_calls_by_message: HashMap<String, Vec<ToolCall>> = HashMap::new();
    for record in tool_calls {
        tool_calls_by_message
            .entry(record.message_id.clone())
            .or_default()
            .push(ToolCall {
                id: record.id.clone(),
                call_type: "function".into(),
                function: FunctionCall {
                    name: record.name.clone(),
                    arguments: record.args_json.clone(),
                },
            });
    }

    messages
        .iter()
        .map(|m| -> Result<ChatMessage, String> {
            let mut image_urls = Vec::new();
            if m.role == "user" {
                if let (Some(sandbox), Some(raw)) = (sandbox, m.attachments_json.as_deref()) {
                    let attachments = parse_attachments_json(Some(raw))?;
                    for attachment in attachments {
                        if !is_upload_attachment_path(&attachment.path) {
                            continue;
                        }
                        if let Ok(url) = encode_attachment_data_url(sandbox, &attachment) {
                            image_urls.push(Arc::from(url));
                        }
                    }
                }
            }
            Ok(ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                image_urls,
                reasoning_content: m.reasoning_content.clone(),
                tool_calls: if m.role == "assistant" {
                    tool_calls_by_message.get(&m.id).cloned()
                } else {
                    None
                },
                tool_call_id: m.tool_call_id.clone(),
            })
        })
        .collect()
}

pub fn messages_from_store_text(
    messages: &[crate::core::store::Message],
    tool_calls: &[crate::core::store::ToolCallRecord],
) -> Vec<ChatMessage> {
    messages_from_store(messages, tool_calls, None)
        .expect("text-only store messages do not read attachments")
}

pub fn effort_from_str(value: &str) -> ThinkingEffort {
    match value {
        "max" => ThinkingEffort::Max,
        _ => ThinkingEffort::High,
    }
}

pub fn model_from_str(value: &str) -> ModelId {
    value.parse().unwrap_or(ModelId::Mock)
}

pub fn is_image_path(path: &str) -> bool {
    matches!(
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("gif")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::{ChatRequest, ModelId, ThinkingConfig, ThinkingEffort};
    use crate::core::store::{Message, ToolCallRecord};

    fn sample_request(model: ModelId, enabled: bool, effort: ThinkingEffort) -> ChatRequest {
        ChatRequest {
            session_id: "s1".into(),
            turn_id: "t1".into(),
            model,
            messages: vec![],
            tools: vec![],
            thinking: ThinkingConfig { enabled, effort },
            response_format: None,
            max_tokens: None,
        }
    }

    #[test]
    fn deepseek_thinking_includes_reasoning_effort() {
        let body = thinking_extra_body(
            &sample_request(ModelId::DeepSeekV4Flash, true, ThinkingEffort::Max),
            false,
        );
        assert_eq!(body["thinking"]["type"], "enabled");
        assert_eq!(body["reasoning_effort"], "max");
    }

    #[test]
    fn kimi_thinking_includes_keep_all() {
        let body = thinking_extra_body(
            &sample_request(ModelId::KimiK26, true, ThinkingEffort::High),
            true,
        );
        assert_eq!(body["thinking"]["type"], "enabled");
        assert_eq!(body["thinking"]["keep"], "all");
        assert!(body.get("reasoning_effort").is_none());
    }

    #[test]
    fn mimo_thinking_has_no_keep_or_effort() {
        let body = mimo_thinking_extra_body(&sample_request(ModelId::MimoV25, true, ThinkingEffort::High));
        assert_eq!(body["thinking"]["type"], "enabled");
        assert!(body["thinking"].get("keep").is_none());
        assert!(body.get("reasoning_effort").is_none());
    }

    #[test]
    fn output_token_limit_maps_by_provider() {
        let mut body = serde_json::Map::new();
        apply_output_token_limit(&mut body, ProviderKind::Deepseek, 1024);
        assert_eq!(body.get("max_tokens").and_then(|v| v.as_u64()), Some(1024));
        assert!(body.get("max_completion_tokens").is_none());

        body.clear();
        apply_output_token_limit(&mut body, ProviderKind::Kimi, 2048);
        assert_eq!(
            body.get("max_completion_tokens").and_then(|v| v.as_u64()),
            Some(2048)
        );
        assert!(body.get("max_tokens").is_none());

        body.clear();
        apply_output_token_limit(&mut body, ProviderKind::Mimo, 4096);
        assert_eq!(
            body.get("max_completion_tokens").and_then(|v| v.as_u64()),
            Some(4096)
        );
    }

    #[test]
    fn model_and_effort_parsing() {
        assert_eq!(model_from_str("kimi-k2.6"), ModelId::KimiK26);
        assert_eq!(model_from_str("mimo-v2.5"), ModelId::MimoV25);
        assert_eq!(model_from_str("unknown"), ModelId::Mock);
        assert_eq!(effort_from_str("max"), ThinkingEffort::Max);
        assert_eq!(effort_from_str("high"), ThinkingEffort::High);
    }

    #[test]
    fn messages_from_store_reconstructs_tool_calls() {
        let assistant = Message {
            id: "assistant-1".into(),
            session_id: "session-1".into(),
            role: "assistant".into(),
            content: None,
            reasoning_content: Some("thinking".into()),
            tool_call_id: None,
            seq: 2,
            created_at: "now".into(),
            archived: false,
            attachments_json: None,
        };
        let tool = Message {
            id: "tool-1".into(),
            session_id: "session-1".into(),
            role: "tool".into(),
            content: Some(r#"{"entries":[]}"#.into()),
            reasoning_content: None,
            tool_call_id: Some("call_1".into()),
            seq: 3,
            created_at: "now".into(),
            archived: false,
            attachments_json: None,
        };
        let tool_calls = vec![ToolCallRecord {
            id: "call_1".into(),
            message_id: "assistant-1".into(),
            name: "fs_list".into(),
            args_json: r#"{"path":"."}"#.into(),
            result_json: Some(r#"{"entries":[]}"#.into()),
            status: "done".into(),
            duration_ms: 1,
            created_at: "now".into(),
        }];

        let chat = messages_from_store_text(&[assistant.clone(), tool], &tool_calls);
        assert_eq!(chat.len(), 2);
        let rebuilt = &chat[0].tool_calls.as_ref().unwrap()[0];
        assert_eq!(rebuilt.id, "call_1");
        assert_eq!(rebuilt.function.name, "fs_list");
        assert_eq!(chat[1].tool_call_id.as_deref(), Some("call_1"));
    }

    #[test]
    fn text_only_store_messages_skip_attachment_encoding() {
        let user = Message {
            id: "user-1".into(),
            session_id: "session-1".into(),
            role: "user".into(),
            content: Some("see image".into()),
            reasoning_content: None,
            tool_call_id: None,
            seq: 1,
            created_at: "now".into(),
            archived: false,
            attachments_json: Some(r#"[{"path":".uploads/a.png","mime":"image/png"}]"#.into()),
        };
        let chat = messages_from_store_text(&[user], &[]);
        assert!(chat[0].image_urls.is_empty());
    }

    #[test]
    fn validate_attachments_rejects_non_upload_paths() {
        let attachments = vec![MessageAttachment {
            path: "report.docx".into(),
            mime: "image/png".into(),
        }];
        assert!(validate_attachments(&attachments).is_err());
        assert!(is_upload_attachment_path(".uploads/a.png"));
        assert!(!is_upload_attachment_path("../.uploads/a.png"));
    }

    #[test]
    fn messages_from_store_skips_missing_upload_files() {
        use crate::core::sandbox::Sandbox;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".uploads")).unwrap();
        let sandbox = Sandbox::new(dir.path().to_str().unwrap()).unwrap();
        let user = Message {
            id: "user-1".into(),
            session_id: "session-1".into(),
            role: "user".into(),
            content: Some("see image".into()),
            reasoning_content: None,
            tool_call_id: None,
            seq: 1,
            created_at: "now".into(),
            archived: false,
            attachments_json: Some(r#"[{"path":".uploads/missing.png","mime":"image/png"}]"#.into()),
        };

        let chat = messages_from_store(&[user], &[], Some(&sandbox)).expect("missing file should not fail turn");
        assert!(chat[0].image_urls.is_empty());
    }

    #[test]
    fn messages_from_store_rejects_corrupt_attachments_json() {
        use crate::core::sandbox::Sandbox;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path().to_str().unwrap()).unwrap();
        let user = Message {
            id: "user-1".into(),
            session_id: "session-1".into(),
            role: "user".into(),
            content: Some("see image".into()),
            reasoning_content: None,
            tool_call_id: None,
            seq: 1,
            created_at: "now".into(),
            archived: false,
            attachments_json: Some("not-json".into()),
        };

        let err = messages_from_store(&[user], &[], Some(&sandbox)).unwrap_err();
        assert!(err.contains("invalid attachments_json"));
    }
}
