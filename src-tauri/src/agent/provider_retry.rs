use crate::agent::provider::sse::cancelable;
use crate::agent::provider::{LlmProvider, ProviderError};
use crate::agent::types::{AgentEvent, AssistantTurn, ChatRequest};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_delay_ms: 1000,
            max_delay_ms: 10_000,
        }
    }
}

impl RetryPolicy {
    /// `DOC_AGENT_PROVIDER_RETRIES` 可覆盖重试次数（`0` 关闭），作为紧急开关。
    pub fn from_env() -> Self {
        let max_retries = std::env::var("DOC_AGENT_PROVIDER_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok());
        Self {
            max_retries: max_retries.unwrap_or(Self::default().max_retries),
            ..Self::default()
        }
    }
}

/// 对瞬时失败（限流 / 5xx / 网络 / 超时 / 断流）做有限重试；一旦任何 token 或工具参数已
/// 输出到 UI 就不再重试，避免正文重复与双重计费。重试通知经 `on_event` 与其他事件同路发出。
pub async fn chat_stream_with_retry(
    provider: &dyn LlmProvider,
    request: ChatRequest,
    api_key: Option<&str>,
    policy: &RetryPolicy,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> Result<AssistantTurn, (ProviderError, u32)> {
    let mut attempt = 0u32;
    loop {
        let emitted = AtomicBool::new(false);
        let result = {
            let mut tap = |ev: AgentEvent| {
                if matches!(
                    ev,
                    AgentEvent::ContentToken { .. }
                        | AgentEvent::ReasoningToken { .. }
                        | AgentEvent::ToolCallStream { .. }
                ) {
                    emitted.store(true, Ordering::Relaxed);
                }
                on_event(ev);
            };
            provider
                .chat_stream(request.clone(), api_key, &mut tap)
                .await
        };
        match result {
            Ok(turn) => return Ok(turn),
            Err(ProviderError::Http(failure))
                if failure.kind.retryable()
                    && !emitted.load(Ordering::Relaxed)
                    && attempt < policy.max_retries =>
            {
                attempt += 1;
                let delay = failure
                    .retry_after_ms
                    .unwrap_or(policy.base_delay_ms * 3u64.pow(attempt - 1))
                    .min(policy.max_delay_ms);
                on_event(AgentEvent::ProviderRetry {
                    session_id: request.session_id.clone(),
                    turn_id: request.turn_id.clone(),
                    attempt,
                    max: policy.max_retries,
                    kind: failure.kind,
                    delay_ms: delay,
                });
                let sleep = tokio::time::sleep(Duration::from_millis(delay));
                if cancelable(request.cancel.as_ref(), sleep).await.is_err() {
                    return Err((ProviderError::Cancelled, attempt));
                }
            }
            Err(err) => return Err((err, attempt)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::provider::mock::{script_calls_for, MockProvider, ScriptedCall};
    use crate::agent::provider::tests::provider_tests::base_request;
    use crate::agent::provider::FailureKind;
    use crate::agent::turn_control::CancelSignal;
    use serial_test::serial;

    fn request() -> ChatRequest {
        let mut request = base_request("hi");
        request.session_id = "s".into();
        request
    }

    fn policy() -> RetryPolicy {
        RetryPolicy {
            max_retries: 2,
            base_delay_ms: 1,
            max_delay_ms: 5,
        }
    }

    async fn run(
        script: Vec<ScriptedCall>,
    ) -> (Result<AssistantTurn, (ProviderError, u32)>, Vec<AgentEvent>) {
        script_calls_for("s", script);
        let mut events = Vec::new();
        let result = chat_stream_with_retry(&MockProvider, request(), None, &policy(), &mut |e| {
            events.push(e)
        })
        .await;
        (result, events)
    }

    fn retries(events: &[AgentEvent]) -> usize {
        events
            .iter()
            .filter(|e| matches!(e, AgentEvent::ProviderRetry { .. }))
            .count()
    }

    #[tokio::test]
    #[serial]
    async fn server_fails_twice_then_succeeds() {
        let (result, events) = run(vec![
            ScriptedCall::fails(FailureKind::Server),
            ScriptedCall::fails(FailureKind::Server),
            ScriptedCall::ok(),
        ])
        .await;
        assert_eq!(result.unwrap().content, "ok");
        assert_eq!(retries(&events), 2);
        assert!(!events.iter().any(|e| matches!(e, AgentEvent::Error { .. })));
    }

    #[tokio::test]
    #[serial]
    async fn network_exhausts_retries() {
        let (result, _) = run(vec![
            ScriptedCall::fails(FailureKind::Network),
            ScriptedCall::fails(FailureKind::Network),
            ScriptedCall::fails(FailureKind::Network),
        ])
        .await;
        match result.unwrap_err() {
            (ProviderError::Http(f), attempts) => {
                assert_eq!(f.kind, FailureKind::Network);
                assert_eq!(attempts, 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn token_then_incomplete_does_not_retry() {
        let (result, events) = run(vec![ScriptedCall::token_then_fails(
            FailureKind::StreamIncomplete,
        )])
        .await;
        let err = result.unwrap_err();
        assert!(matches!(
            err.0,
            ProviderError::Http(ref f) if f.kind == FailureKind::StreamIncomplete
        ));
        assert_eq!(err.1, 0);
        assert_eq!(retries(&events), 0);
        assert!(events
            .iter()
            .any(|e| matches!(e, AgentEvent::ContentToken { .. })));
    }

    #[tokio::test]
    #[serial]
    async fn bad_request_does_not_retry() {
        let (result, events) = run(vec![ScriptedCall::fails(FailureKind::BadRequest)]).await;
        let err = result.unwrap_err();
        assert!(matches!(
            err.0,
            ProviderError::Http(ref f) if f.kind == FailureKind::BadRequest
        ));
        assert_eq!(err.1, 0);
        assert_eq!(retries(&events), 0);
    }

    #[tokio::test]
    #[serial]
    async fn cancel_during_backoff_stops() {
        script_calls_for(
            "s",
            vec![
                ScriptedCall::fails(FailureKind::Network),
                ScriptedCall::ok(),
            ],
        );
        let cancel = CancelSignal::new();
        let mut req = request();
        req.cancel = Some(cancel.clone());
        let slow = RetryPolicy {
            max_retries: 2,
            base_delay_ms: 400,
            max_delay_ms: 400,
        };
        let handle = tokio::spawn(async move {
            chat_stream_with_retry(&MockProvider, req, None, &slow, &mut |_| {}).await
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        cancel.cancel();
        let err = handle.await.unwrap().unwrap_err();
        assert!(matches!(err.0, ProviderError::Cancelled));
    }

    #[test]
    #[serial]
    fn env_zero_disables_retries() {
        unsafe {
            std::env::set_var("DOC_AGENT_PROVIDER_RETRIES", "0");
        }
        assert_eq!(RetryPolicy::from_env().max_retries, 0);
        unsafe {
            std::env::remove_var("DOC_AGENT_PROVIDER_RETRIES");
        }
        assert_eq!(RetryPolicy::from_env().max_retries, 2);
    }
}
