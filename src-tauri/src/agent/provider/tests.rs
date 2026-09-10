#[cfg(test)]
pub(crate) mod provider_tests {
    use crate::agent::provider::mock::MockProvider;
    use crate::agent::provider::openai_compat::{extra_body_for, OpenAiCompatClient};
    use crate::agent::provider::{FailureKind, LlmProvider, ProviderError, ProviderFailure};
    use crate::agent::provider_retry::{chat_stream_with_retry, RetryPolicy};
    use crate::agent::types::{
        AgentEvent, AssistantTurn, ChatMessage, ChatRequest, ModelId, ThinkingConfig,
        ThinkingEffort, ToolDefinition,
    };

    pub(crate) fn base_request(user_text: &str) -> ChatRequest {
        ChatRequest {
            session_id: "session-1".into(),
            turn_id: "turn-1".into(),
            model: ModelId::Mock,
            messages: vec![ChatMessage {
                role: "user".into(),
                content: Some(user_text.into()),
                image_urls: vec![],
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                provider_state: None,
            }],
            tools: vec![ToolDefinition {
                name: "fs_list".into(),
                description: "list".into(),
                parameters: serde_json::json!({}),
                strict: None,
            }],
            thinking: ThinkingConfig {
                enabled: true,
                effort: ThinkingEffort::High,
            },
            response_format: None,
            max_tokens: None,
            cancel: None,
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
    async fn ultraspeed_does_not_send_http() {
        use crate::agent::provider::mimo::MimoProvider;
        let provider = MimoProvider::default();
        let mut request = base_request("hello");
        request.model = ModelId::MimoV25ProUltraspeed;
        let err = provider
            .chat_stream(request, Some("sk-test"), &mut |_| {})
            .await
            .unwrap_err();
        assert!(err.to_string().contains("MiMo v2.5 Pro"));
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

    /// 极简假 HTTP 服务：按顺序对每个连接回一条响应（status, content_type, extra_headers, body）。
    async fn serve_http(responses: Vec<(u16, &'static str, &'static str, String)>) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            for (status, content_type, extra_headers, body) in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                let resp = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n{extra_headers}\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(resp.as_bytes()).await;
            }
        });
        format!("http://{addr}/v1/chat/completions")
    }

    fn compat_client(url: String) -> OpenAiCompatClient {
        OpenAiCompatClient {
            chat_url: url,
            client: reqwest::Client::builder().no_proxy().build().unwrap(),
        }
    }

    fn unwrap_http(err: ProviderError) -> ProviderFailure {
        match err {
            ProviderError::Http(failure) => failure,
            other => panic!("expected Http, got {other:?}"),
        }
    }

    /// 让真实 `complete_chat` 打到一条 JSON 错误响应，返回分类结果。
    async fn classify_response(
        model: ModelId,
        status: u16,
        extra_headers: &'static str,
        body: &'static str,
    ) -> ProviderFailure {
        let url = serve_http(vec![(
            status,
            "application/json",
            extra_headers,
            body.into(),
        )])
        .await;
        let mut req = base_request("hello");
        req.model = model;
        let extra = extra_body_for(&req);
        unwrap_http(
            compat_client(url)
                .complete_chat(req, "sk-test", extra)
                .await
                .unwrap_err(),
        )
    }

    #[tokio::test]
    async fn classify_429_with_retry_after() {
        let failure = classify_response(
            ModelId::DeepSeekV4Flash,
            429,
            "Retry-After: 7\r\n",
            r#"{"error":{"message":"Rate limit reached","type":"rate_limit_error","code":"rate_limit_exceeded"}}"#,
        )
        .await;
        assert_eq!(failure.kind, FailureKind::RateLimit);
        assert_eq!(failure.status, Some(429));
        assert_eq!(
            failure.provider_code.as_deref(),
            Some("rate_limit_exceeded")
        );
        assert_eq!(failure.retry_after_ms, Some(7000));
        assert!(failure.message.contains("Rate limit reached"));
    }

    #[tokio::test]
    async fn classify_400_context_length() {
        let failure = classify_response(
            ModelId::DeepSeekV4Flash,
            400,
            "",
            r#"{"error":{"message":"This model's maximum context length is 128k tokens"}}"#,
        )
        .await;
        assert_eq!(failure.kind, FailureKind::ContextLength);
        assert_eq!(failure.status, Some(400));
    }

    #[tokio::test]
    async fn classify_413_payload_too_large() {
        let failure = classify_response(
            ModelId::DeepSeekV4Flash,
            413,
            "",
            r#"{"error":"too large"}"#,
        )
        .await;
        assert_eq!(failure.kind, FailureKind::PayloadTooLarge);
        assert_eq!(failure.status, Some(413));
    }

    #[tokio::test]
    async fn gemini_400_has_no_detail() {
        let failure = classify_response(
            ModelId::Gemini38Flash,
            400,
            "",
            r#"{"error":{"message":"echoed request metadata secret"}}"#,
        )
        .await;
        assert_eq!(failure.kind, FailureKind::BadRequest);
        assert_eq!(failure.status, Some(400));
        assert!(failure.detail.is_none());
        assert!(!failure.message.contains("secret"));
    }

    /// 把 `OpenAiCompatClient` 包成 `LlmProvider`，用真实 HTTP + SSE 路径验证重试包装。
    struct CompatProbe(OpenAiCompatClient);

    #[async_trait::async_trait]
    impl LlmProvider for CompatProbe {
        async fn chat_stream(
            &self,
            request: ChatRequest,
            api_key: Option<&str>,
            on_event: &mut (dyn FnMut(AgentEvent) + Send),
        ) -> Result<AssistantTurn, ProviderError> {
            let extra = extra_body_for(&request);
            let session_id = request.session_id.clone();
            let turn_id = request.turn_id.clone();
            self.0
                .stream_chat(
                    request,
                    api_key.unwrap_or("sk-test"),
                    extra,
                    &session_id,
                    &turn_id,
                    on_event,
                )
                .await
        }
    }

    async fn run_with_retry(
        responses: Vec<(u16, &'static str, &'static str, String)>,
    ) -> (Result<AssistantTurn, (ProviderError, u32)>, Vec<AgentEvent>) {
        let provider = CompatProbe(compat_client(serve_http(responses).await));
        let mut req = base_request("hello");
        req.model = ModelId::DeepSeekV4Flash;
        let policy = RetryPolicy {
            max_retries: 2,
            base_delay_ms: 1,
            max_delay_ms: 5,
        };
        let mut events = Vec::new();
        let result = chat_stream_with_retry(&provider, req, Some("sk-test"), &policy, &mut |e| {
            events.push(e)
        })
        .await;
        (result, events)
    }

    #[tokio::test]
    async fn retries_503_then_succeeds() {
        let sse = "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
        let (result, events) = run_with_retry(vec![
            (
                503,
                "application/json",
                "",
                r#"{"error":{"message":"busy"}}"#.into(),
            ),
            (200, "text/event-stream", "", sse.into()),
        ])
        .await;
        assert_eq!(result.unwrap().content, "hi");
        let retries = events
            .iter()
            .filter(|e| matches!(e, AgentEvent::ProviderRetry { .. }))
            .count();
        assert_eq!(retries, 1);
    }

    #[tokio::test]
    async fn incomplete_stream_after_token_does_not_retry() {
        let sse = "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n";
        let (result, events) =
            run_with_retry(vec![(200, "text/event-stream", "", sse.into())]).await;
        match result.unwrap_err() {
            (ProviderError::Http(failure), attempts) => {
                assert_eq!(failure.kind, FailureKind::StreamIncomplete);
                assert_eq!(attempts, 0);
            }
            other => panic!("{other:?}"),
        }
        assert!(events.iter().any(|e| matches!(
            e,
            AgentEvent::ContentToken { delta, .. } if delta == "hi"
        )));
    }
}
