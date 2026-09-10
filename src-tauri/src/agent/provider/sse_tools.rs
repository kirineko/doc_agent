use super::gemini::state::merge_extra;
use super::sse::SseError;
use crate::agent::types::{FunctionCall, ToolCall};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Default)]
pub struct ToolCalls {
    pub calls: Vec<ToolCall>,
    pub extras: Vec<Value>,
}

impl ToolCalls {
    pub fn merge(&mut self, items: &[Value], google: bool) -> Result<Vec<Value>, SseError> {
        let mut normalized = Vec::new();
        for item in items {
            let id = item["id"].as_str().filter(|s| !s.is_empty());
            let index = if let Some(index) = item["index"].as_u64() {
                usize::try_from(index).map_err(|_| SseError::Json("tool index overflow".into()))?
            } else if let Some(id) = id {
                self.calls
                    .iter()
                    .position(|call| call.id == id)
                    .unwrap_or(self.calls.len())
            } else if self.calls.len() <= 1 {
                0
            } else {
                return Err(SseError::Json(
                    "ambiguous tool delta without index or id".into(),
                ));
            };
            if index > 4096 {
                return Err(SseError::Json("tool index too large".into()));
            }
            while self.calls.len() <= index {
                self.calls.push(ToolCall {
                    id: format!("call_{}", Uuid::new_v4()),
                    call_type: "function".into(),
                    function: FunctionCall {
                        name: String::new(),
                        arguments: String::new(),
                    },
                });
                self.extras.push(json!({}));
            }
            let call = &mut self.calls[index];
            if let Some(id) = id {
                call.id = id.into();
            }
            if let Some(kind) = item["type"].as_str() {
                call.call_type = kind.into();
            }
            if let Some(name) = item["function"]["name"].as_str() {
                call.function.name = name.into();
            }
            if let Some(args) = item["function"]["arguments"].as_str() {
                call.function.arguments.push_str(args);
            }
            if google {
                if let Some(extra) = item.get("extra_content") {
                    merge_extra(&mut self.extras[index], extra).map_err(SseError::Json)?;
                }
            }
            // Only public progress fields reach UI callbacks, never signatures.
            normalized.push(json!({"index": index, "function": item["function"]}));
        }
        Ok(normalized)
    }
}
