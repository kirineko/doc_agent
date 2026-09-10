//! Opt-in networked smoke. Default CI stays offline.
//! Enable with `DOC_AGENT_LIVE_SMOKE=1` and provider API keys in the environment.

#[cfg(test)]
mod tests {
    use crate::agent::provider::openai_compat::{
        extra_body_for, OpenAiCompatClient, DEEPSEEK_CHAT_URL, KIMI_CHAT_URL, MIMO_CHAT_URL,
        ZHIPU_CHAT_URL,
    };
    use crate::agent::types::{ChatMessage, ChatRequest, ModelId, ThinkingConfig, ThinkingEffort};

    fn live_enabled() -> bool {
        matches!(
            std::env::var("DOC_AGENT_LIVE_SMOKE").as_deref(),
            Ok("1") | Ok("true")
        )
    }

    fn text_request(model: ModelId, effort: ThinkingEffort) -> ChatRequest {
        ChatRequest {
            session_id: "smoke".into(),
            turn_id: "smoke".into(),
            model,
            messages: vec![ChatMessage {
                role: "user".into(),
                content: Some("Reply with the single word pong.".into()),
                image_urls: vec![],
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                provider_state: None,
            }],
            tools: vec![],
            thinking: ThinkingConfig {
                enabled: true,
                effort,
            },
            response_format: None,
            max_tokens: Some(32),
            cancel: None,
        }
    }

    #[tokio::test]
    async fn live_smoke_is_skipped_without_env() {
        if live_enabled() {
            return;
        }
        // Default cargo test / CI path: no network.
    }

    #[tokio::test]
    async fn live_smoke_six_models_when_enabled() {
        if !live_enabled() {
            return;
        }
        let cases: &[(&str, ModelId, ThinkingEffort)] = &[
            (
                "DEEPSEEK_API_KEY",
                ModelId::DeepSeekV4Flash,
                ThinkingEffort::Low,
            ),
            ("MIMO_API_KEY", ModelId::MimoV25, ThinkingEffort::High),
            ("KIMI_API_KEY", ModelId::KimiK3, ThinkingEffort::Low),
            ("ZHIPU_API_KEY", ModelId::Glm53Flash, ThinkingEffort::Low),
        ];
        for (env_name, model, effort) in cases {
            let key = std::env::var(env_name).unwrap_or_default();
            if key.is_empty() {
                println!("SKIP {env_name}: key missing");
                continue;
            }
            let url = match model.provider_kind() {
                crate::agent::model_catalog::ProviderKind::Deepseek => DEEPSEEK_CHAT_URL,
                crate::agent::model_catalog::ProviderKind::Kimi => KIMI_CHAT_URL,
                crate::agent::model_catalog::ProviderKind::Mimo => MIMO_CHAT_URL,
                crate::agent::model_catalog::ProviderKind::Zhipu => ZHIPU_CHAT_URL,
                _ => continue,
            };
            let client = OpenAiCompatClient::new(url);
            let request = text_request(*model, *effort);
            let extra = extra_body_for(&request);
            let result = client.complete_chat(request, &key, extra).await;
            match result {
                Ok(turn) => println!(
                    "OK {} content_chars={}",
                    model.as_str(),
                    turn.content.chars().count()
                ),
                Err(err) => println!("FAIL {} {}", model.as_str(), err),
            }
        }
        if let Ok(key) = std::env::var("GOOGLE_API_KEY") {
            if key.is_empty() {
                println!("SKIP GOOGLE_API_KEY: key missing");
            } else {
                use crate::agent::provider::LlmProvider;
                let mut request = text_request(ModelId::Gemini38Flash, ThinkingEffort::Low);
                request.max_tokens = Some(1024);
                let result = crate::agent::provider::gemini::GeminiProvider
                    .chat_stream(request, Some(&key), &mut |_| {})
                    .await;
                assert!(result.is_ok(), "Gemini live request failed");
            }
        }
    }
}
