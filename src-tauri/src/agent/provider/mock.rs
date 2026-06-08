use super::{LlmProvider, ProviderError};
use crate::agent::types::{AgentEvent, AssistantTurn, ChatRequest, ToolCall};
use async_trait::async_trait;
use serde_json::json;

pub struct MockProvider;

#[async_trait]
impl LlmProvider for MockProvider {
    async fn chat_stream(
        &self,
        request: ChatRequest,
        _api_key: Option<&str>,
        mut on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<AssistantTurn, ProviderError> {
        let session_id = request.session_id.clone();
        let turn_id = request.turn_id.clone();
        let user_text = request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .and_then(|m| m.content.clone())
            .unwrap_or_default();

        emit(&mut on_event, AgentEvent::ReasoningToken {
            session_id: session_id.clone(),
            turn_id: turn_id.clone(),
            delta: "分析用户请求并决定是否调用工具…".into(),
        });

        let wants_tool = user_text.contains("列出") || user_text.to_lowercase().contains("list");
        if wants_tool && request.tools.iter().any(|t| t.name == "fs_list") {
            let tool_id = "call_mock_1".to_string();
            emit(
                &mut on_event,
                AgentEvent::ToolCall {
                    session_id: session_id.clone(),
                    turn_id: turn_id.clone(),
                    id: tool_id.clone(),
                    name: "fs_list".into(),
                    args: json!({ "path": "." }),
                    status: "running".into(),
                },
            );
            return Ok(AssistantTurn {
                content: String::new(),
                reasoning_content: "需要列出目录内容。".into(),
                tool_calls: vec![ToolCall {
                    id: tool_id,
                    call_type: "function".into(),
                    function: crate::agent::types::FunctionCall {
                        name: "fs_list".into(),
                        arguments: json!({ "path": "." }).to_string(),
                    },
                }],
            });
        }

        let answer = format!("Mock 回复：已收到「{user_text}」。你可以试试发送「列出目录」触发工具调用。");
        for chunk in answer.split_inclusive(' ') {
            emit(
                &mut on_event,
                AgentEvent::ContentToken {
                    session_id: session_id.clone(),
                    turn_id: turn_id.clone(),
                    delta: chunk.to_string(),
                },
            );
        }

        Ok(AssistantTurn {
            content: answer,
            reasoning_content: "直接回答。".into(),
            tool_calls: vec![],
        })
    }
}

fn emit(on_event: &mut (dyn FnMut(AgentEvent) + Send), event: AgentEvent) {
    on_event(event);
}
