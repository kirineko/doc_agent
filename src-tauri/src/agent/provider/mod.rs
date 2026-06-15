pub mod deepseek;
pub mod kimi;
pub mod mimo;
pub mod mock;
pub mod openai_compat;
pub mod sse;

#[cfg(test)]
mod tests;

use crate::agent::types::{AssistantTurn, ChatRequest, ModelId};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("missing api key")]
    MissingApiKey,
    #[error("http error: {0}")]
    Http(String),
    #[error("parse error: {0}")]
    Parse(String),
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat_stream(
        &self,
        request: ChatRequest,
        api_key: Option<&str>,
        on_event: &mut (dyn FnMut(crate::agent::types::AgentEvent) + Send),
    ) -> Result<AssistantTurn, ProviderError>;
}

pub fn provider_for(model: ModelId) -> Box<dyn LlmProvider> {
    match model {
        ModelId::DeepSeekV4Flash | ModelId::DeepSeekV4Pro => {
            Box::new(deepseek::DeepSeekProvider::default())
        }
        ModelId::KimiK26 => Box::new(kimi::KimiProvider::default()),
        ModelId::MimoV25 | ModelId::MimoV25Pro | ModelId::MimoV25ProUltraspeed => {
            Box::new(mimo::MimoProvider::default())
        }
        ModelId::Mock => Box::new(mock::MockProvider),
    }
}
