use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    Mock,
}

impl ModelId {
    pub fn provider_key(self) -> &'static str {
        match self {
            Self::DeepSeekV4Flash | Self::DeepSeekV4Pro => "deepseek",
            Self::KimiK26 => "kimi",
            Self::Mock => "mock",
        }
    }

    pub fn api_model(self) -> &'static str {
        match self {
            Self::DeepSeekV4Flash => "deepseek-v4-flash",
            Self::DeepSeekV4Pro => "deepseek-v4-pro",
            Self::KimiK26 => "kimi-k2.6",
            Self::Mock => "mock",
        }
    }

    pub fn supports_effort(self) -> bool {
        matches!(self, Self::DeepSeekV4Flash | Self::DeepSeekV4Pro)
    }
}

impl std::str::FromStr for ModelId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deepseek-v4-flash" => Ok(Self::DeepSeekV4Flash),
            "deepseek-v4-pro" => Ok(Self::DeepSeekV4Pro),
            "kimi-k2.6" => Ok(Self::KimiK26),
            "mock" => Ok(Self::Mock),
            other => Err(format!("unknown model: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
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
    ToolCall {
        session_id: String,
        turn_id: String,
        id: String,
        name: String,
        args: Value,
        status: String,
    },
    ToolResult {
        session_id: String,
        turn_id: String,
        id: String,
        ok: bool,
        summary: String,
        duration_ms: i64,
    },
    TurnComplete {
        session_id: String,
        turn_id: String,
    },
    Error {
        session_id: String,
        turn_id: String,
        message: String,
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
}

#[derive(Debug, Clone)]
pub struct AssistantTurn {
    pub content: String,
    pub reasoning_content: String,
    pub tool_calls: Vec<ToolCall>,
}
