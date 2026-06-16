use crate::agent::model_catalog::{ModelCatalog, ProviderKind};
use crate::agent::turn_control::CancelSignal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    pub enabled: bool,
    pub effort: ThinkingEffort,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingEffort {
    High,
    Max,
}

impl ThinkingEffort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Max => "max",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelId {
    DeepSeekV4Flash,
    DeepSeekV4Pro,
    KimiK26,
    MimoV25,
    MimoV25Pro,
    MimoV25ProUltraspeed,
    Mock,
}

impl ModelId {
    pub fn provider_key(self) -> &'static str {
        self.provider_kind().secrets_key()
    }

    pub fn api_model(self) -> &'static str {
        self.info().api_model
    }

    pub fn supports_effort(self) -> bool {
        self.info().supports_effort
    }

    pub fn supports_vision(self) -> bool {
        self.info().supports_vision
    }

    pub fn max_context_size(self) -> u32 {
        self.info().max_context
    }

    pub fn provider_kind(self) -> ProviderKind {
        self.info().provider
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeepSeekV4Flash => "deepseek-v4-flash",
            Self::DeepSeekV4Pro => "deepseek-v4-pro",
            Self::KimiK26 => "kimi-k2.6",
            Self::MimoV25 => "mimo-v2.5",
            Self::MimoV25Pro => "mimo-v2.5-pro",
            Self::MimoV25ProUltraspeed => "mimo-v2.5-pro-ultraspeed",
            Self::Mock => "mock",
        }
    }

    fn info(self) -> &'static crate::agent::model_catalog::ModelInfo {
        if self == Self::Mock {
            static MOCK: crate::agent::model_catalog::ModelInfo =
                crate::agent::model_catalog::ModelInfo {
                    id: "mock",
                    label: "Mock",
                    provider: ProviderKind::Mock,
                    api_model: "mock",
                    supports_vision: false,
                    supports_effort: false,
                    max_context: 100_000,
                };
            return &MOCK;
        }
        ModelCatalog::find(self.as_str()).expect("catalog entry for model id")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

impl std::str::FromStr for ModelId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deepseek-v4-flash" => Ok(Self::DeepSeekV4Flash),
            "deepseek-v4-pro" => Ok(Self::DeepSeekV4Pro),
            "kimi-k2.6" => Ok(Self::KimiK26),
            "mimo-v2.5" => Ok(Self::MimoV25),
            "mimo-v2.5-pro" => Ok(Self::MimoV25Pro),
            "mimo-v2.5-pro-ultraspeed" => Ok(Self::MimoV25ProUltraspeed),
            "mock" => Ok(Self::Mock),
            other => Err(format!("unknown model: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub path: String,
    pub mime: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip)]
    pub image_urls: Vec<Arc<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyOption {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyQuestion {
    pub id: String,
    pub kind: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<ClarifyOption>,
    #[serde(default)]
    pub allow_custom: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_selections: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_selections: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyAnswer {
    pub question_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<String>,
    pub display_text: String,
    /// confirm_brief 确认时回传创作简报，保证 tool result 自含结构化 brief
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentEvent {
    ReasoningToken {
        session_id: String,
        turn_id: String,
        delta: String,
    },
    ContentToken {
        session_id: String,
        turn_id: String,
        delta: String,
    },
    /// 工具参数仍在流式生成中（长参数场景下避免 UI 假死）
    ToolCallStream {
        session_id: String,
        turn_id: String,
        index: usize,
        name: String,
        args_chars: usize,
    },
    ToolCall {
        session_id: String,
        turn_id: String,
        id: String,
        name: String,
        args: Value,
        status: String,
        #[serde(default)]
        index: usize,
    },
    ToolResult {
        session_id: String,
        turn_id: String,
        id: String,
        ok: bool,
        summary: String,
        duration_ms: i64,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        changed_paths: Vec<String>,
    },
    TurnComplete {
        session_id: String,
        turn_id: String,
    },
    TurnCancelled {
        session_id: String,
        turn_id: String,
    },
    TurnAwaitingUser {
        session_id: String,
        turn_id: String,
    },
    ClarifyQuestion {
        session_id: String,
        turn_id: String,
        tool_call_id: String,
        question: ClarifyQuestion,
    },
    AssistantStepDone {
        session_id: String,
        turn_id: String,
        message: crate::core::store::Message,
    },
    Error {
        session_id: String,
        turn_id: String,
        message: String,
    },
    ContextUsage {
        session_id: String,
        used_tokens: u32,
        max_tokens: u32,
        ratio: f64,
    },
    ContextCompacted {
        session_id: String,
        before_tokens: u32,
        after_tokens: u32,
    },
    SessionTitleUpdated {
        session_id: String,
        title: String,
    },
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub session_id: String,
    pub turn_id: String,
    pub model: ModelId,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolDefinition>,
    pub thinking: ThinkingConfig,
    pub response_format: Option<Value>,
    pub max_tokens: Option<u32>,
    pub cancel: Option<CancelSignal>,
}

#[derive(Debug, Clone)]
pub struct AssistantTurn {
    pub content: String,
    pub reasoning_content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<String>,
    pub usage: Option<TokenUsage>,
}

/// API-only placeholder when the user sends images without text.
/// Persisted message content stays empty; injected only at LLM request serialization.
pub const IMAGE_ONLY_USER_API_TEXT: &str = "请描述用户发送的图片。";

impl ChatMessage {
    pub fn text_content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    fn user_multimodal_api_text(&self) -> &str {
        self.content
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .unwrap_or(IMAGE_ONLY_USER_API_TEXT)
    }
}

impl Serialize for ChatMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ChatMessage", 6)?;
        state.serialize_field("role", &self.role)?;
        if self.role == "user" && !self.image_urls.is_empty() {
            let mut parts = Vec::new();
            parts.push(json!({
                "type": "text",
                "text": self.user_multimodal_api_text(),
            }));
            for url in &self.image_urls {
                parts.push(json!({ "type": "image_url", "image_url": { "url": url } }));
            }
            state.serialize_field("content", &parts)?;
        } else if let Some(content) = &self.content {
            state.serialize_field("content", content)?;
        } else {
            state.serialize_field("content", &Value::Null)?;
        }
        if let Some(reasoning) = &self.reasoning_content {
            state.serialize_field("reasoning_content", reasoning)?;
        }
        if let Some(tool_calls) = &self.tool_calls {
            state.serialize_field("tool_calls", tool_calls)?;
        }
        if let Some(tool_call_id) = &self.tool_call_id {
            state.serialize_field("tool_call_id", tool_call_id)?;
        }
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::ChatMessage;
    use super::IMAGE_ONLY_USER_API_TEXT;

    fn image_user_message(content: Option<&str>) -> ChatMessage {
        ChatMessage {
            role: "user".into(),
            content: content.map(str::to_string),
            image_urls: vec!["data:image/png;base64,abc".into()],
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn image_only_user_message_serializes_with_placeholder_text() {
        let msg = image_user_message(Some(""));
        let value = serde_json::to_value(&msg).expect("serialize");
        let parts = value["content"].as_array().expect("content array");
        assert_eq!(parts[0]["type"], "text");
        assert_eq!(parts[0]["text"], IMAGE_ONLY_USER_API_TEXT);
        assert_eq!(parts[1]["type"], "image_url");
    }

    #[test]
    fn image_user_message_with_text_keeps_user_text() {
        let msg = image_user_message(Some("  看图  "));
        let value = serde_json::to_value(&msg).expect("serialize");
        let parts = value["content"].as_array().expect("content array");
        assert_eq!(parts[0]["text"], "看图");
    }
}
