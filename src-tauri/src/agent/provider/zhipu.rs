use super::openai_compat::{extra_body_for, OpenAiCompatClient, ZHIPU_CHAT_URL};
use super::{LlmProvider, ProviderError};
use crate::agent::types::{AgentEvent, AssistantTurn, ChatRequest};
use async_trait::async_trait;

pub struct ZhipuProvider {
    client: OpenAiCompatClient,
}

impl Default for ZhipuProvider {
    fn default() -> Self {
        Self {
            client: OpenAiCompatClient::new(ZHIPU_CHAT_URL),
        }
    }
}

#[async_trait]
impl LlmProvider for ZhipuProvider {
    async fn chat_stream(
        &self,
        request: ChatRequest,
        api_key: Option<&str>,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<AssistantTurn, ProviderError> {
        let api_key = api_key.ok_or(ProviderError::MissingApiKey)?;
        let extra = extra_body_for(&request);
        let session_id = request.session_id.clone();
        let turn_id = request.turn_id.clone();
        self.client
            .stream_chat(request, api_key, extra, &session_id, &turn_id, on_event)
            .await
    }
}
