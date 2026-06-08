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

                on_delta(reasoning_delta, content_delta, delta_tools.map(|v| v.as_slice()));
            }
        }
    }

    Ok(AssistantTurn {
        content,
        reasoning_content: reasoning,
        tool_calls,
    })
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
}
