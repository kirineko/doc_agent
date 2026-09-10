use super::openai_compat::{extra_body_for, OpenAiCompatClient, DEEPSEEK_CHAT_URL};
use super::{LlmProvider, ProviderError};
use crate::agent::types::{AgentEvent, AssistantTurn, ChatRequest};
use async_trait::async_trait;

pub struct DeepSeekProvider {
    client: OpenAiCompatClient,
}

impl Default for DeepSeekProvider {
    fn default() -> Self {
        Self {
            client: OpenAiCompatClient::new(DEEPSEEK_CHAT_URL),
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
        let extra = extra_body_for(&request);
        // session/turn ids are injected by loop runner via closure wrapper
        let session_id = request.session_id.clone();
        let turn_id = request.turn_id.clone();
        self.client
            .stream_chat(request, api_key, extra, &session_id, &turn_id, on_event)
            .await
    }
}
