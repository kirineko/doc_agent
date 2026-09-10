use super::{request::GOOGLE_CHAT_URL, tests::request};
use crate::agent::provider::openai_compat::{extra_body_for, OpenAiCompatClient};
use crate::agent::provider::ProviderError;
use crate::agent::turn_control::CancelSignal;
use serde_json::Value;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn read_request(socket: &mut TcpStream) -> (String, Value) {
    let mut data = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let n = socket.read(&mut buf).await.unwrap();
        assert!(n > 0);
        data.extend_from_slice(&buf[..n]);
        if let Some(split) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            let headers = std::str::from_utf8(&data[..split]).unwrap();
            let length: usize = headers
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|s| s.trim().parse().unwrap())
                })
                .unwrap();
            if data.len() >= split + 4 + length {
                return (
                    headers.into(),
                    serde_json::from_slice(&data[split + 4..split + 4 + length]).unwrap(),
                );
            }
        }
    }
}

#[tokio::test]
async fn product_client_uses_bearer_and_compat_body() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let (headers, body) = read_request(&mut socket).await;
        assert!(headers
            .to_ascii_lowercase()
            .contains("authorization: bearer synthetic-test-key"));
        assert!(!headers.to_ascii_lowercase().contains("x-goog-api-key"));
        assert!(headers.starts_with("POST /v1beta/openai/chat/completions "));
        assert!(body.get("messages").is_some());
        assert!(body.get("input").is_none());
        let data = "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
        socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",data.len()).as_bytes()).await.unwrap();
        socket.write_all(data.as_bytes()).await.unwrap();
    });
    let client = OpenAiCompatClient {
        chat_url: format!("http://{addr}/v1beta/openai/chat/completions"),
        client: reqwest::Client::builder().no_proxy().build().unwrap(),
    };
    let req = request();
    let extra = extra_body_for(&req);
    let turn = client
        .stream_chat(req, "synthetic-test-key", extra, "s", "t", &mut |_| {})
        .await
        .unwrap();
    assert_eq!(turn.content, "你好");
    server.await.unwrap();
    assert!(GOOGLE_CHAT_URL.starts_with("https://"));
}

#[tokio::test]
async fn cancelling_while_waiting_for_headers_drops_request() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let cancel = CancelSignal::new();
    let server_cancel = cancel.clone();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        read_request(&mut socket).await;
        server_cancel.cancel();
        let mut byte = [0];
        tokio::time::timeout(Duration::from_secs(2), socket.read(&mut byte))
            .await
            .unwrap()
            .unwrap();
    });
    let client = OpenAiCompatClient {
        chat_url: format!("http://{addr}/v1beta/openai/chat/completions"),
        client: reqwest::Client::builder().no_proxy().build().unwrap(),
    };
    let mut req = request();
    req.cancel = Some(cancel);
    let extra = extra_body_for(&req);
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        client.stream_chat(req, "test", extra, "s", "t", &mut |_| {}),
    )
    .await
    .unwrap();
    assert!(matches!(result, Err(ProviderError::Cancelled)));
    server.await.unwrap();
}
