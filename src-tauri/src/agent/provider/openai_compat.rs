use crate::agent::provider::{sse, ProviderError};
use crate::agent::types::{
    AgentEvent, AssistantTurn, ChatMessage, ChatRequest, ModelId, ThinkingEffort,
};
use reqwest::Client;
use serde_json::{json, Value};

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
        });

        if let Some(obj) = body.as_object_mut() {
            if let Some(response_format) = request.response_format {
                obj.insert("response_format".into(), response_format);
            }
            if let Some(max_tokens) = request.max_tokens {
                obj.insert("max_tokens".into(), json!(max_tokens));
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

pub fn messages_from_store(
    messages: &[crate::core::store::Message],
    tool_calls: &[crate::core::store::ToolCallRecord],
) -> Vec<ChatMessage> {
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
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
            reasoning_content: m.reasoning_content.clone(),
            tool_calls: if m.role == "assistant" {
                tool_calls_by_message.get(&m.id).cloned()
            } else {
                None
            },
            tool_call_id: m.tool_call_id.clone(),
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::{ChatRequest, ModelId, ThinkingConfig, ThinkingEffort};

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
    fn model_and_effort_parsing() {
        assert_eq!(model_from_str("kimi-k2.6"), ModelId::KimiK26);
        assert_eq!(model_from_str("unknown"), ModelId::Mock);
        assert_eq!(effort_from_str("max"), ThinkingEffort::Max);
        assert_eq!(effort_from_str("high"), ThinkingEffort::High);
    }

    #[test]
    fn messages_from_store_reconstructs_tool_calls() {
        use crate::core::store::{Message, ToolCallRecord};

        let assistant = Message {
            id: "assistant-1".into(),
            session_id: "session-1".into(),
            role: "assistant".into(),
            content: None,
            reasoning_content: Some("thinking".into()),
            tool_call_id: None,
            seq: 2,
            created_at: "now".into(),
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

        let chat = messages_from_store(&[assistant.clone(), tool], &tool_calls);
        assert_eq!(chat.len(), 2);
        let rebuilt = &chat[0].tool_calls.as_ref().unwrap()[0];
        assert_eq!(rebuilt.id, "call_1");
        assert_eq!(rebuilt.function.name, "fs_list");
        assert_eq!(chat[1].tool_call_id.as_deref(), Some("call_1"));
    }
}
