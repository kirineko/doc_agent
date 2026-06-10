use super::openai_compat::{thinking_extra_body, OpenAiCompatClient};
use super::{LlmProvider, ProviderError};
use crate::agent::types::{AgentEvent, AssistantTurn, ChatRequest};
use async_trait::async_trait;

pub struct DeepSeekProvider {
    client: OpenAiCompatClient,
}

impl Default for DeepSeekProvider {
    fn default() -> Self {
        Self {
            client: OpenAiCompatClient::new("https://api.deepseek.com"),
        }
    }
}

impl DeepSeekProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl LlmProvider for DeepSeekProvider {
    async fn chat_stream(
        &self,
        request: ChatRequest,
        api_key: Option<&str>,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<AssistantTurn, ProviderError> {
        let api_key = api_key.ok_or(ProviderError::MissingApiKey)?;
        let extra = thinking_extra_body(&request, false);
        // session/turn ids are injected by loop runner via closure wrapper
        let session_id = request.session_id.clone();
        let turn_id = request.turn_id.clone();
        self.client
            .stream_chat(request, api_key, extra, &session_id, &turn_id, on_event)
            .await
    }
}
