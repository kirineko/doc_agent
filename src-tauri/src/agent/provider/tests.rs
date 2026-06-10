#[cfg(test)]
mod provider_tests {
    use crate::agent::provider::mock::MockProvider;
    use crate::agent::provider::LlmProvider;
    use crate::agent::types::{
        ChatMessage, ChatRequest, ModelId, ThinkingConfig, ThinkingEffort, ToolDefinition,
    };

    fn base_request(user_text: &str) -> ChatRequest {
        ChatRequest {
            session_id: "session-1".into(),
            turn_id: "turn-1".into(),
            model: ModelId::Mock,
            messages: vec![ChatMessage {
                role: "user".into(),
                content: Some(user_text.into()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            tools: vec![ToolDefinition {
                name: "fs_list".into(),
                description: "list".into(),
                parameters: serde_json::json!({}),
            }],
            thinking: ThinkingConfig {
                enabled: true,
                effort: ThinkingEffort::High,
            },
            response_format: None,
            max_tokens: None,
        }
    }

    #[tokio::test]
    async fn mock_returns_tool_call_for_list_keyword() {
        let provider = MockProvider;
        let mut events = Vec::new();
        let turn = provider
            .chat_stream(base_request("请列出目录"), None, &mut |event| {
                events.push(event)
            })
            .await
            .unwrap();

        assert_eq!(turn.tool_calls.len(), 1);
        assert_eq!(turn.tool_calls[0].function.name, "fs_list");
        assert!(events
            .iter()
            .any(|e| matches!(e, crate::agent::types::AgentEvent::ReasoningToken { .. })));
    }

    #[tokio::test]
    async fn mock_returns_content_for_generic_prompt() {
        let provider = MockProvider;
        let turn = provider
            .chat_stream(base_request("你好"), None, &mut |_| {})
            .await
            .unwrap();

        assert!(turn.tool_calls.is_empty());
        assert!(turn.content.contains("Mock 回复"));
    }
}
