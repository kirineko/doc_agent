use super::{LlmProvider, ProviderError};
use crate::agent::types::{
    AgentEvent, AssistantTurn, ChatMessage, ChatRequest, TokenUsage, ToolCall,
};
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

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
            return Ok(finish_turn(
                &request.messages,
                answer,
                "收到 clarify tool result 后继续回答。",
                vec![],
                None,
            ));
        }

        let wants_clarify =
            user_text.contains("澄清") || user_text.to_lowercase().contains("clarify");
        let wants_list = user_text.contains("列出") || user_text.to_lowercase().contains("list");
        let wants_slow = user_text.contains("慢工具");

        if wants_slow {
            for chunk in ["执行", "慢", "工具", "中", "…"] {
                if request
                    .cancel
                    .as_ref()
                    .is_some_and(|signal| signal.is_cancelled())
                {
                    return Err(ProviderError::Cancelled);
                }
                sleep(Duration::from_millis(80)).await;
                emit(
                    &mut on_event,
                    AgentEvent::ContentToken {
                        session_id: session_id.clone(),
                        turn_id: turn_id.clone(),
                        delta: chunk.to_string(),
                    },
                );
            }
            if request.tools.iter().any(|t| t.name == "fs_list") {
                let tool_id = "call_mock_slow_1".to_string();
                return Ok(finish_turn(
                    &request.messages,
                    String::new(),
                    "慢工具场景。",
                    vec![ToolCall {
                        id: tool_id,
                        call_type: "function".into(),
                        function: crate::agent::types::FunctionCall {
                            name: "fs_list".into(),
                            arguments: json!({ "path": "." }).to_string(),
                        },
                    }],
                    None,
                ));
            }
        }

        let wants_clarify_first = user_text.contains("先澄清再列出");
        let wants_pdf_clarify = user_text.contains("读取PDF并澄清");

        if wants_clarify_first
            && request.tools.iter().any(|t| t.name == "clarify_ask")
            && request.tools.iter().any(|t| t.name == "fs_list")
        {
            let args = clarify_question_args();
            return Ok(finish_turn(
                &request.messages,
                String::new(),
                "先澄清，同时也返回后续目录读取调用。",
                vec![
                    ToolCall {
                        id: "call_mock_clarify_1".into(),
                        call_type: "function".into(),
                        function: crate::agent::types::FunctionCall {
                            name: "clarify_ask".into(),
                            arguments: args.to_string(),
                        },
                    },
                    ToolCall {
                        id: "call_mock_fs_1".into(),
                        call_type: "function".into(),
                        function: crate::agent::types::FunctionCall {
                            name: "fs_list".into(),
                            arguments: json!({ "path": "." }).to_string(),
                        },
                    },
                ],
                None,
            ));
        }

        if wants_pdf_clarify
            && request.tools.iter().any(|t| t.name == "clarify_ask")
            && request.tools.iter().any(|t| t.name == "pdf_read")
        {
            let args = clarify_question_args();
            return Ok(finish_turn(
                &request.messages,
                String::new(),
                "先读取两个 PDF，再澄清文档类型。",
                vec![
                    ToolCall {
                        id: "call_mock_pdf_1".into(),
                        call_type: "function".into(),
                        function: crate::agent::types::FunctionCall {
                            name: "pdf_read".into(),
                            arguments: json!({ "path": "a.pdf" }).to_string(),
                        },
                    },
                    ToolCall {
                        id: "call_mock_pdf_2".into(),
                        call_type: "function".into(),
                        function: crate::agent::types::FunctionCall {
                            name: "pdf_read".into(),
                            arguments: json!({ "path": "b.pdf" }).to_string(),
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
                None,
            ));
        }

        // 混合场景：同一轮返回普通工具 + clarify_ask，用于验证「先执行普通工具再暂停」
        if wants_clarify
            && wants_list
            && request.tools.iter().any(|t| t.name == "clarify_ask")
            && request.tools.iter().any(|t| t.name == "fs_list")
        {
            let args = clarify_question_args();
            return Ok(finish_turn(
                &request.messages,
                String::new(),
                "先查看目录，同时向用户澄清文档类型。",
                vec![
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
                None,
            ));
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
                    index: 0,
                },
            );
            return Ok(finish_turn(
                &request.messages,
                String::new(),
                "需要向用户澄清文档类型。",
                vec![ToolCall {
                    id: tool_id,
                    call_type: "function".into(),
                    function: crate::agent::types::FunctionCall {
                        name: "clarify_ask".into(),
                        arguments: args.to_string(),
                    },
                }],
                None,
            ));
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
                    index: 0,
                },
            );
            return Ok(finish_turn(
                &request.messages,
                String::new(),
                "需要列出目录内容。",
                vec![ToolCall {
                    id: tool_id,
                    call_type: "function".into(),
                    function: crate::agent::types::FunctionCall {
                        name: "fs_list".into(),
                        arguments: json!({ "path": "." }).to_string(),
                    },
                }],
                None,
            ));
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

        Ok(finish_turn(
            &request.messages,
            answer.clone(),
            "直接回答。",
            vec![],
            None,
        ))
    }
}

fn finish_turn(
    messages: &[ChatMessage],
    content: String,
    reasoning_content: &str,
    tool_calls: Vec<ToolCall>,
    finish_reason: Option<String>,
) -> AssistantTurn {
    let prompt = estimate_messages_tokens(messages);
    let completion = estimate_text_tokens(&content) + estimate_text_tokens(reasoning_content);
    AssistantTurn {
        content,
        reasoning_content: reasoning_content.to_string(),
        tool_calls,
        finish_reason,
        usage: Some(TokenUsage {
            prompt,
            completion,
            total: prompt.saturating_add(completion),
        }),
    }
}

fn estimate_text_tokens(text: &str) -> u32 {
    (text.chars().count() / 4).max(if text.is_empty() { 0 } else { 1 }) as u32
}

fn estimate_messages_tokens(messages: &[ChatMessage]) -> u32 {
    messages
        .iter()
        .map(|m| {
            estimate_text_tokens(m.content.as_deref().unwrap_or(""))
                + estimate_text_tokens(m.reasoning_content.as_deref().unwrap_or(""))
                + m.tool_calls
                    .as_ref()
                    .map(|calls| {
                        calls
                            .iter()
                            .map(|c| {
                                estimate_text_tokens(&c.function.name)
                                    + estimate_text_tokens(&c.function.arguments)
                            })
                            .sum::<u32>()
                    })
                    .unwrap_or(0)
        })
        .sum()
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
