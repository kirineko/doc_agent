pub mod request;
pub mod state;
pub mod stream;

#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod live_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod transport_tests;

use super::openai_compat::{extra_body_for, OpenAiCompatClient};
use super::{LlmProvider, ProviderError};
use crate::agent::types::{AgentEvent, AssistantTurn, ChatRequest};
use async_trait::async_trait;
use request::GOOGLE_CHAT_URL;

#[derive(Default)]
pub struct GeminiProvider;

pub fn new_google_http_client() -> Result<reqwest::Client, ProviderError> {
    reqwest::Client::builder()
        .use_rustls_tls()
        .https_only(true)
        .build()
        .map_err(|_| ProviderError::Http("无法创建 Gemini HTTP 客户端".into()))
}

pub(super) fn map_transport_error(err: reqwest::Error) -> ProviderError {
    if err.is_timeout() {
        ProviderError::Http("Gemini 请求超时，请检查系统代理或 TUN".into())
    } else {
        ProviderError::Http("Gemini 网络连接失败，请检查系统代理、TUN 或网络".into())
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn chat_stream(
        &self,
        request: ChatRequest,
        api_key: Option<&str>,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<AssistantTurn, ProviderError> {
        let api_key = api_key
            .filter(|key| !key.is_empty())
            .ok_or(ProviderError::MissingApiKey)?;
        let client = OpenAiCompatClient {
            chat_url: GOOGLE_CHAT_URL.into(),
            client: new_google_http_client()?,
        };
        let extra = extra_body_for(&request);
        let session = request.session_id.clone();
        let turn = request.turn_id.clone();
        client
            .stream_chat(request, api_key, extra, &session, &turn, on_event)
            .await
    }
}

#[cfg(test)]
mod proxy_tests {
    use super::*;
    #[test]
    fn client_factory_has_no_hardcoded_proxy_port() {
        let client = new_google_http_client().unwrap();
        let debug = format!("{client:?}");
        assert!(!debug.contains("7890"));
        assert!(!debug.contains("7897"));
    }

    #[tokio::test]
    async fn explicit_http_proxy_sends_connect_to_google_host() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;
        use tokio::sync::oneshot;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = oneshot::channel::<String>();
        tokio::spawn(async move {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let mut buf = vec![0u8; 1024];
            let n = sock.read(&mut buf).await.unwrap_or(0);
            let seen = String::from_utf8_lossy(&buf[..n]).to_string();
            let _ = sock
                .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n")
                .await;
            let _ = tx.send(seen);
        });
        let proxy = format!("http://{addr}");
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .proxy(reqwest::Proxy::all(&proxy).unwrap())
            .build()
            .unwrap();
        let _ = client.post(GOOGLE_CHAT_URL).send().await;
        let seen = tokio::time::timeout(std::time::Duration::from_secs(2), rx)
            .await
            .expect("proxy accepted a connection")
            .unwrap_or_default();
        assert!(
            seen.to_ascii_uppercase().contains("CONNECT"),
            "expected CONNECT, got {seen:?}"
        );
        assert!(
            seen.contains("generativelanguage.googleapis.com"),
            "expected Google host, got {seen:?}"
        );
        assert!(!seen.to_ascii_lowercase().contains("authorization"));
    }
}
