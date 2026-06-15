use super::openai_compat::{mimo_thinking_extra_body, OpenAiCompatClient};
use super::{LlmProvider, ProviderError};
use crate::agent::types::{AgentEvent, AssistantTurn, ChatRequest};
use async_trait::async_trait;

pub struct MimoProvider {
    client: OpenAiCompatClient,
}

impl Default for MimoProvider {
    fn default() -> Self {
        Self {
            client: OpenAiCompatClient::new("https://api.xiaomimimo.com"),
        }
    }
}

#[async_trait]
impl LlmProvider for MimoProvider {
    async fn chat_stream(
        &self,
        request: ChatRequest,
        api_key: Option<&str>,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<AssistantTurn, ProviderError> {
        let api_key = api_key.ok_or(ProviderError::MissingApiKey)?;
        let extra = mimo_thinking_extra_body(&request);
        let session_id = request.session_id.clone();
        let turn_id = request.turn_id.clone();
        self.client
            .stream_chat(request, api_key, extra, &session_id, &turn_id, on_event)
            .await
    }
}
