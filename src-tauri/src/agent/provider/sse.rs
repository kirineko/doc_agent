use crate::agent::turn_control::CancelSignal;
use crate::agent::types::AssistantTurn;
use futures_util::{Stream, StreamExt};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SseError {
    #[error("http stream error: {0}")]
    Http(String),
    #[error("json error: {0}")]
    Json(String),
    #[error("cancelled")]
    Cancelled,
}

impl From<SseError> for super::ProviderError {
    fn from(error: SseError) -> Self {
        match error {
            SseError::Http(message) => Self::Http(message),
            SseError::Json(message) => Self::Parse(message),
            SseError::Cancelled => Self::Cancelled,
        }
    }
}

pub async fn cancelable<T>(
    cancel: Option<&CancelSignal>,
    future: impl std::future::Future<Output = T>,
) -> Result<T, SseError> {
    tokio::select! {
        biased;
        _ = wait_for_cancel(cancel), if cancel.is_some() => Err(SseError::Cancelled),
        result = future => Ok(result),
    }
}

pub(crate) async fn consume_stream<S, B, E, F>(
    mut stream: S,
    cancel: Option<&CancelSignal>,
    google_model: Option<&str>,
    mut on_delta: F,
) -> Result<AssistantTurn, SseError>
where
    S: Stream<Item = Result<B, E>> + Unpin,
    B: AsRef<[u8]>,
    E: std::fmt::Display,
    F: FnMut(Option<&str>, Option<&str>, Option<&[Value]>) + Send,
{
    let mut buffer = super::sse_frames::Frames::default();
    let mut acc = super::openai_stream::Accumulator::new(google_model);
    let mut emit = |delta: super::openai_stream::Delta| {
        on_delta(
            (!delta.reasoning.is_empty()).then_some(delta.reasoning.as_str()),
            (!delta.content.is_empty()).then_some(delta.content.as_str()),
            (!delta.tools.is_empty()).then_some(delta.tools.as_slice()),
        );
    };
    while let Some(chunk) = cancelable(cancel, stream.next()).await? {
        let chunk = chunk.map_err(|e| {
            SseError::Http(if google_model.is_some() {
                "Gemini 连接中断，请检查系统代理或 TUN".into()
            } else {
                e.to_string()
            })
        })?;
        for data in buffer.push(chunk.as_ref())? {
            if data == "[DONE]" {
                continue;
            }
            let value: Value = serde_json::from_str(&data)
                .map_err(|_| SseError::Json("invalid SSE JSON".into()))?;
            emit(acc.apply(&value)?);
        }
    }
    if cancel.is_some_and(|s| s.is_cancelled()) {
        return Err(SseError::Cancelled);
    }
    buffer.finish()?;
    emit(acc.flush_text());
    acc.finish()
}

async fn wait_for_cancel(cancel: Option<&CancelSignal>) {
    let Some(signal) = cancel else {
        std::future::pending::<()>().await;
        return;
    };
    while !signal.is_cancelled() {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

/// 跟踪工具调用参数的流式接收进度，按时间节流产出 UI 更新信号，
/// 避免长参数（如大段 skill_run 代码）期间界面长时间无反馈。
pub struct ToolStreamTracker {
    names: Vec<String>,
    chars: Vec<usize>,
    last_emit: Option<std::time::Instant>,
    throttle_ms: u128,
}

impl ToolStreamTracker {
    pub fn new() -> Self {
        Self::with_throttle(120)
    }

    pub fn with_throttle(throttle_ms: u128) -> Self {
        Self {
            names: Vec::new(),
            chars: Vec::new(),
            last_emit: None,
            throttle_ms,
        }
    }

    /// 吸收一批 tool_calls delta，返回需要上报的 (index, name, 已接收字符数)。
    /// 节流窗口内返回 None；新工具名首次出现时立即上报。
    pub fn update(&mut self, items: &[Value]) -> Option<(usize, String, usize)> {
        let mut latest: Option<usize> = None;
        let mut new_name = false;
        for item in items {
            let index = item["index"].as_u64().unwrap_or(0) as usize;
            while self.names.len() <= index {
                self.names.push(String::new());
                self.chars.push(0);
            }
            if let Some(name) = item["function"]["name"].as_str() {
                if self.names[index].is_empty() && !name.is_empty() {
                    new_name = true;
                }
                self.names[index] = name.to_string();
            }
            if let Some(args) = item["function"]["arguments"].as_str() {
                self.chars[index] += args.chars().count();
            }
            latest = Some(index);
        }
        let index = latest?;
        let now = std::time::Instant::now();
        let due = new_name
            || self
                .last_emit
                .is_none_or(|t| now.duration_since(t).as_millis() >= self.throttle_ms);
        if !due {
            return None;
        }
        self.last_emit = Some(now);
        Some((index, self.names[index].clone(), self.chars[index]))
    }
}

impl Default for ToolStreamTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
fn merge_tool_call_deltas(calls: &mut Vec<crate::agent::types::ToolCall>, items: &[Value]) {
    let mut acc = super::sse_tools::ToolCalls {
        calls: std::mem::take(calls),
        extras: vec![],
    };
    acc.extras.resize(acc.calls.len(), serde_json::json!({}));
    acc.merge(items, false).unwrap();
    *calls = acc.calls;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_tool_call_deltas_without_provider_id_keeps_generated_id() {
        let mut tool_calls = Vec::new();
        merge_tool_call_deltas(
            &mut tool_calls,
            &[json!({
                "index": 0,
                "function": { "name": "pdf_read", "arguments": "{\"path\":" }
            })],
        );
        assert_eq!(tool_calls.len(), 1);
        assert!(!tool_calls[0].id.is_empty());
        assert!(tool_calls[0].id.starts_with("call_"));
        merge_tool_call_deltas(
            &mut tool_calls,
            &[json!({
                "index": 0,
                "function": { "arguments": "\"a.pdf\"}" }
            })],
        );
        assert_eq!(tool_calls[0].function.arguments, r#"{"path":"a.pdf"}"#);
    }

    #[test]
    fn merge_tool_call_deltas_accumulates_chunks() {
        let mut tool_calls = Vec::new();
        merge_tool_call_deltas(
            &mut tool_calls,
            &[json!({
                "index": 0,
                "id": "call_1",
                "function": { "name": "fs_list", "arguments": "{\"path\":" }
            })],
        );
        merge_tool_call_deltas(
            &mut tool_calls,
            &[json!({
                "index": 0,
                "function": { "arguments": "\".\"}" }
            })],
        );

        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_1");
        assert_eq!(tool_calls[0].function.name, "fs_list");
        assert_eq!(tool_calls[0].function.arguments, r#"{"path":"."}"#);
    }

    #[test]
    fn tool_stream_tracker_reports_progress() {
        let mut tracker = ToolStreamTracker::with_throttle(0);

        // 新工具名首次出现立即上报
        let update = tracker.update(&[json!({
            "index": 0,
            "function": { "name": "skill_run", "arguments": "{\"code\":\"" }
        })]);
        assert_eq!(update, Some((0, "skill_run".to_string(), 9)));

        // 后续参数累计字符数
        let update = tracker.update(&[json!({
            "index": 0,
            "function": { "arguments": "0123456789" }
        })]);
        assert_eq!(update, Some((0, "skill_run".to_string(), 19)));
    }

    #[test]
    fn tool_stream_tracker_throttles_between_emits() {
        let mut tracker = ToolStreamTracker::with_throttle(10_000);
        let delta = json!({
            "index": 0,
            "function": { "name": "skill_run", "arguments": "abc" }
        });
        // 首次（新工具名）立即上报，节流窗口内的后续 delta 不上报
        assert!(tracker.update(std::slice::from_ref(&delta)).is_some());
        let follow_up = json!({
            "index": 0,
            "function": { "arguments": "def" }
        });
        assert!(tracker.update(std::slice::from_ref(&follow_up)).is_none());
    }

    #[test]
    fn finish_reason_length_is_read_from_choice() {
        let choice = json!({
            "delta": { "tool_calls": [{ "index": 0, "function": { "name": "skill_run", "arguments": "{\"code\":\"" } }] },
            "finish_reason": "length"
        });
        assert_eq!(choice["finish_reason"].as_str(), Some("length"));
    }

    #[test]
    fn parse_usage_reads_prompt_completion_total() {
        let chunk = json!({
            "usage": {
                "prompt_tokens": 1200,
                "completion_tokens": 300,
                "total_tokens": 1500
            }
        });
        let usage = super::super::openai_stream::parse_usage(&chunk, false).unwrap();
        assert_eq!(usage.prompt, 1200);
        assert_eq!(usage.completion, 300);
        assert_eq!(usage.total, 1500);
    }
}
