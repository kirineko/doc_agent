use super::gemini::{
    state::{merge_extra, GoogleReplayState},
    stream::ThoughtText,
};
use super::sse::SseError;
use super::sse_tools::ToolCalls;
use crate::agent::types::{AssistantTurn, TokenUsage};
use serde_json::{json, Value};

#[derive(Debug, Default)]
pub struct Delta {
    pub reasoning: String,
    pub content: String,
    pub tools: Vec<Value>,
}

pub struct Accumulator {
    content: String,
    reasoning: String,
    tools: ToolCalls,
    finish_reason: Option<String>,
    usage: Option<TokenUsage>,
    google: Option<GoogleReplayState>,
    thought: ThoughtText,
}

impl Accumulator {
    pub fn new(google_model: Option<&str>) -> Self {
        Self {
            content: String::new(),
            reasoning: String::new(),
            tools: ToolCalls::default(),
            finish_reason: None,
            usage: None,
            google: google_model.map(GoogleReplayState::new),
            thought: ThoughtText::default(),
        }
    }

    pub fn apply(&mut self, value: &Value) -> Result<Delta, SseError> {
        if value.get("error").is_some() {
            return Err(SseError::Http(
                super::failure::ProviderFailure::from_stream_error(value, self.google.is_some()),
            ));
        }
        if let Some(usage) = parse_usage(value, self.google.is_some()) {
            self.usage = Some(usage);
        }
        let Some(choice) = value["choices"].as_array().and_then(|items| items.first()) else {
            return Ok(Delta::default());
        };
        if let Some(reason) = choice["finish_reason"].as_str() {
            self.finish_reason = Some(reason.into());
        }
        let delta = &choice["delta"];
        let text = delta["content"].as_str().unwrap_or_default();
        let mut out = Delta::default();
        if let Some(state) = &mut self.google {
            if let Some(extra) = delta.get("extra_content") {
                let mut extra = extra.clone();
                if let Some(google) = extra.get_mut("google").and_then(Value::as_object_mut) {
                    google.remove("thought");
                }
                merge_extra(&mut state.message_extra_content, &extra).map_err(SseError::Json)?;
            }
            (out.reasoning, out.content) = self.thought.push(
                text,
                delta
                    .pointer("/extra_content/google/thought")
                    .and_then(Value::as_bool)
                    == Some(true),
            );
        } else {
            out.reasoning = delta["reasoning_content"]
                .as_str()
                .unwrap_or_default()
                .into();
            out.content = text.into();
        }
        if let Some(items) = delta["tool_calls"].as_array() {
            out.tools = self.tools.merge(items, self.google.is_some())?;
        }
        self.content.push_str(&out.content);
        self.reasoning.push_str(&out.reasoning);
        Ok(out)
    }

    pub fn flush_text(&mut self) -> Delta {
        let (reasoning, content) = self.thought.finish();
        self.content.push_str(&content);
        self.reasoning.push_str(&reasoning);
        Delta {
            reasoning,
            content,
            tools: vec![],
        }
    }

    pub fn finish(mut self) -> Result<AssistantTurn, SseError> {
        let reason = self.finish_reason.as_deref().ok_or_else(|| {
            SseError::Http(super::failure::ProviderFailure::new(
                super::failure::FailureKind::StreamIncomplete,
                "模型响应未正常结束",
            ))
        })?;
        if let Some(state) = &mut self.google {
            if reason == "length" {
                self.tools.calls.clear();
                self.tools.extras.clear();
                if !self.content.is_empty() {
                    self.content.push('\n');
                }
                self.content.push_str("输出被截断，未执行本步工具。");
                self.finish_reason = Some("incomplete".into());
            } else if !matches!(reason, "stop" | "tool_calls") {
                return Err(SseError::Http(super::failure::ProviderFailure::new(
                    super::failure::FailureKind::StreamIncomplete,
                    "Gemini 响应未正常完成",
                )));
            } else {
                for call in &self.tools.calls {
                    if call.call_type != "function"
                        || call.function.name.is_empty()
                        || !serde_json::from_str::<Value>(&call.function.arguments)
                            .is_ok_and(|v| v.is_object())
                    {
                        return Err(SseError::Json(
                            "Gemini 工具调用不完整或参数不是 JSON 对象".into(),
                        ));
                    }
                }
                if reason == "tool_calls" && self.tools.calls.is_empty() {
                    return Err(SseError::Json("Gemini tool_calls 终态缺少调用".into()));
                }
            }
            state.tool_extra_content = self.tools.extras;
            state
                .validate_calls(&self.tools.calls)
                .map_err(SseError::Json)?;
        }
        Ok(AssistantTurn {
            content: self.content,
            reasoning_content: self.reasoning,
            tool_calls: self.tools.calls,
            finish_reason: self.finish_reason,
            usage: self.usage,
            provider_state: self
                .google
                .map(|s| s.to_json())
                .transpose()
                .map_err(SseError::Json)?,
        })
    }
}

pub fn parse_usage(value: &Value, google: bool) -> Option<TokenUsage> {
    let usage = &value["usage"];
    let prompt = u32::try_from(usage["prompt_tokens"].as_u64()?).ok()?;
    let mut completion = u32::try_from(usage["completion_tokens"].as_u64()?).ok()?;
    let total = usage["total_tokens"]
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(prompt.saturating_add(completion));
    if google {
        completion = completion.max(total.saturating_sub(prompt));
    }
    Some(TokenUsage {
        prompt,
        completion,
        total,
    })
}

pub fn parse_complete(
    payload: &Value,
    google_model: Option<&str>,
) -> Result<AssistantTurn, SseError> {
    let choice = payload["choices"]
        .as_array()
        .and_then(|v| v.first())
        .ok_or_else(|| SseError::Json("missing choices".into()))?;
    let mut acc = Accumulator::new(google_model);
    acc.apply(&json!({"choices":[{"delta":choice["message"],"finish_reason":choice["finish_reason"]}],"usage":payload["usage"]}))?;
    acc.flush_text();
    acc.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::provider::failure::FailureKind;
    use crate::agent::provider::sse::SseError;

    #[test]
    fn stream_error_keeps_provider_message() {
        let mut acc = Accumulator::new(None);
        let value = json!({"error":{"code":"1210","message":"messages 过长"}});
        let err = acc.apply(&value).unwrap_err();
        let SseError::Http(failure) = err else {
            panic!("expected Http, got {err:?}");
        };
        assert_eq!(failure.kind, FailureKind::StreamError);
        assert_eq!(failure.provider_code.as_deref(), Some("1210"));
        assert!(failure.message.contains("messages 过长"));
        assert!(failure
            .detail
            .as_deref()
            .unwrap_or("")
            .contains("messages 过长"));
    }

    #[test]
    fn missing_finish_reason_is_stream_incomplete() {
        let acc = Accumulator::new(None);
        let err = acc.finish().unwrap_err();
        let SseError::Http(failure) = err else {
            panic!("expected Http, got {err:?}");
        };
        assert_eq!(failure.kind, FailureKind::StreamIncomplete);
    }
}
