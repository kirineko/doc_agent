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

        emit(
            &mut on_event,
            AgentEvent::ReasoningToken {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                delta: "分析用户请求并决定是否调用工具…".into(),
            },
        );

        if request
            .messages
            .iter()
            .rev()
            .any(|m| m.role == "tool" && m.content.as_deref().unwrap_or("").contains("question_id"))
        {
            let answer = "Mock 回复：已收到澄清答案，继续完成任务。".to_string();
            emit(
                &mut on_event,
                AgentEvent::ContentToken {
                    session_id: session_id.clone(),
                    turn_id: turn_id.clone(),
                    delta: answer.clone(),
                },
            );
            return Ok(AssistantTurn {
                content: answer,
                reasoning_content: "收到 clarify tool result 后继续回答。".into(),
                tool_calls: vec![],
                finish_reason: None,
            });
        }

        let wants_clarify =
            user_text.contains("澄清") || user_text.to_lowercase().contains("clarify");
        let wants_list = user_text.contains("列出") || user_text.to_lowercase().contains("list");

        // 混合场景：同一轮返回普通工具 + clarify_ask，用于验证「先执行普通工具再暂停」
        if wants_clarify
            && wants_list
            && request.tools.iter().any(|t| t.name == "clarify_ask")
            && request.tools.iter().any(|t| t.name == "fs_list")
        {
            let args = clarify_question_args();
            return Ok(AssistantTurn {
                content: String::new(),
                reasoning_content: "先查看目录，同时向用户澄清文档类型。".into(),
                tool_calls: vec![
                    ToolCall {
                        id: "call_mock_fs_1".into(),
                        call_type: "function".into(),
                        function: crate::agent::types::FunctionCall {
                            name: "fs_list".into(),
                            arguments: json!({ "path": "." }).to_string(),
                        },
                    },
                    ToolCall {
                        id: "call_mock_clarify_1".into(),
                        call_type: "function".into(),
                        function: crate::agent::types::FunctionCall {
                            name: "clarify_ask".into(),
                            arguments: args.to_string(),
                        },
                    },
                ],
                finish_reason: None,
            });
        }

        if wants_clarify && request.tools.iter().any(|t| t.name == "clarify_ask") {
            let tool_id = "call_mock_clarify_1".to_string();
            let args = clarify_question_args();
            emit(
                &mut on_event,
                AgentEvent::ToolCall {
                    session_id: session_id.clone(),
                    turn_id: turn_id.clone(),
                    id: tool_id.clone(),
                    name: "clarify_ask".into(),
                    args: args.clone(),
                    status: "running".into(),
                },
            );
            return Ok(AssistantTurn {
                content: String::new(),
                reasoning_content: "需要向用户澄清文档类型。".into(),
                tool_calls: vec![ToolCall {
                    id: tool_id,
                    call_type: "function".into(),
                    function: crate::agent::types::FunctionCall {
                        name: "clarify_ask".into(),
                        arguments: args.to_string(),
                    },
                }],
                finish_reason: None,
            });
        }

        if wants_list && request.tools.iter().any(|t| t.name == "fs_list") {
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
                finish_reason: None,
            });
        }

        let answer =
            format!("Mock 回复：已收到「{user_text}」。你可以试试发送「列出目录」触发工具调用。");
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
            finish_reason: None,
        })
    }
}

fn emit(on_event: &mut (dyn FnMut(AgentEvent) + Send), event: AgentEvent) {
    on_event(event);
}

fn clarify_question_args() -> serde_json::Value {
    json!({
        "id": "mock_doc_type",
        "kind": "single",
        "prompt": "你想创建哪类文档？",
        "options": [
            { "id": "docx", "label": "Word 文档" },
            { "id": "pptx", "label": "PPT 演示" },
            { "id": "xlsx", "label": "Excel 表格" },
            { "id": "html-report", "label": "HTML 报告" }
        ],
        "allow_custom": true
    })
}
