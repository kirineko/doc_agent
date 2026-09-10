//! Opaque replay metadata only; text, arguments and IDs live in normal messages.
use crate::agent::types::ToolCall;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoogleReplayState {
    pub protocol: String,
    pub version: u32,
    pub model: String,
    pub message_extra_content: Value,
    pub tool_extra_content: Vec<Value>,
}

impl GoogleReplayState {
    pub fn new(model: &str) -> Self {
        Self {
            protocol: "google_openai".into(),
            version: 1,
            model: model.into(),
            message_extra_content: json!({}),
            tool_extra_content: Vec::new(),
        }
    }

    pub fn parse(raw: &str, model: &str) -> Result<Self, String> {
        // Do not include serde's offending values: they can contain opaque signatures.
        let state: Self =
            serde_json::from_str(raw).map_err(|_| "Gemini 签名元数据损坏".to_string())?;
        if state.protocol != "google_openai" || state.version != 1 || state.model != model {
            return Err("Gemini 签名元数据协议、版本或模型不匹配".into());
        }
        state.validate_shape()?;
        Ok(state)
    }

    fn validate_shape(&self) -> Result<(), String> {
        if !self.message_extra_content.is_object()
            || self
                .tool_extra_content
                .iter()
                .any(|extra| !extra.is_object())
        {
            return Err("Gemini extra_content 必须为对象".into());
        }
        for extra in std::iter::once(&self.message_extra_content).chain(&self.tool_extra_content) {
            if let Some(google) = extra.get("google") {
                if !google.is_object()
                    || google
                        .get("thought_signature")
                        .is_some_and(|v| v.as_str().is_none_or(str::is_empty))
                {
                    return Err("Gemini 签名元数据格式无效".into());
                }
            }
        }
        Ok(())
    }

    pub fn validate_calls(&self, calls: &[ToolCall]) -> Result<(), String> {
        self.validate_shape()?;
        if self.tool_extra_content.len() != calls.len() {
            return Err("Gemini 工具与签名元数据数量不匹配".into());
        }
        if !calls.is_empty()
            && self.tool_extra_content[0]
                .pointer("/google/thought_signature")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
        {
            return Err("Gemini 工具调用缺少 thought_signature".into());
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate_shape()?;
        serde_json::to_string(self).map_err(|_| "Gemini 签名元数据序列化失败".into())
    }
}

/// Metadata values are snapshots, not text deltas. Never concatenate signatures.
pub fn merge_extra(target: &mut Value, source: &Value) -> Result<(), String> {
    let source = source
        .as_object()
        .ok_or("Gemini extra_content 必须为对象")?;
    let target = target
        .as_object_mut()
        .ok_or("Gemini extra_content 必须为对象")?;
    for (key, value) in source {
        if let Some(previous) = target.get_mut(key) {
            if previous.is_object() && value.is_object() {
                merge_extra(previous, value)?;
            } else if previous != value {
                return Err("Gemini 收到冲突的签名元数据".into());
            }
        } else {
            target.insert(key.clone(), value.clone());
        }
    }
    Ok(())
}
