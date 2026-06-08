use super::openai_compat::{OpenAiCompatClient, thinking_extra_body};
use super::{LlmProvider, ProviderError};
use crate::agent::types::{AgentEvent, AssistantTurn, ChatRequest};
use async_trait::async_trait;

pub struct KimiProvider {
    client: OpenAiCompatClient,
}

impl Default for KimiProvider {
    fn default() -> Self {
        Self {
            client: OpenAiCompatClient::new("https://api.moonshot.cn"),
        }
    }
}

#[async_trait]
impl LlmProvider for KimiProvider {
    async fn chat_stream(
        &self,
        request: ChatRequest,
        api_key: Option<&str>,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<AssistantTurn, ProviderError> {
        let api_key = api_key.ok_or(ProviderError::MissingApiKey)?;
        let extra = thinking_extra_body(&request, true);
        let session_id = request.session_id.clone();
        let turn_id = request.turn_id.clone();
        self.client
            .stream_chat(request, api_key, extra, &session_id, &turn_id, on_event)
            .await
    }
}
