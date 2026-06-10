use crate::agent::types::{AssistantTurn, ToolCall};
use futures_util::StreamExt;
use reqwest::Response;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SseError {
    #[error("http stream error: {0}")]
    Http(String),
    #[error("json error: {0}")]
    Json(String),
}

pub async fn consume_openai_sse<F>(
    response: Response,
    mut on_delta: F,
) -> Result<AssistantTurn, SseError>
where
    F: FnMut(Option<&str>, Option<&str>, Option<&[Value]>) + Send,
{
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut content = String::new();
    let mut reasoning = String::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| SseError::Http(e.to_string()))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find("\n\n") {
            let frame = buffer[..pos].to_string();
            buffer.drain(..pos + 2);

            for line in frame.lines() {
                if !line.starts_with("data: ") {
                    continue;
                }
                let data = &line[6..];
                if data == "[DONE]" {
                    continue;
                }
                let value: Value =
                    serde_json::from_str(data).map_err(|e| SseError::Json(e.to_string()))?;
                let choice = &value["choices"][0];
                let delta = &choice["delta"];
                let reasoning_delta = delta["reasoning_content"].as_str();
                let content_delta = delta["content"].as_str();
                let delta_tools = delta["tool_calls"].as_array();

                if let Some(r) = reasoning_delta {
                    reasoning.push_str(r);
                }
                if let Some(c) = content_delta {
                    content.push_str(c);
                }
                if let Some(items) = delta_tools {
                    merge_tool_call_deltas(&mut tool_calls, items);
                }

                on_delta(
                    reasoning_delta,
                    content_delta,
                    delta_tools.map(|v| v.as_slice()),
                );
            }
        }
    }

    Ok(AssistantTurn {
        content,
        reasoning_content: reasoning,
        tool_calls,
    })
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

fn merge_tool_call_deltas(tool_calls: &mut Vec<ToolCall>, items: &[Value]) {
    for item in items {
        let index = item["index"].as_u64().unwrap_or(0) as usize;
        while tool_calls.len() <= index {
            tool_calls.push(ToolCall {
                id: String::new(),
                call_type: "function".into(),
                function: crate::agent::types::FunctionCall {
                    name: String::new(),
                    arguments: String::new(),
                },
            });
        }
        if let Some(id) = item["id"].as_str() {
            tool_calls[index].id = id.to_string();
        }
        if let Some(name) = item["function"]["name"].as_str() {
            tool_calls[index].function.name = name.to_string();
        }
        if let Some(args) = item["function"]["arguments"].as_str() {
            tool_calls[index].function.arguments.push_str(args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
