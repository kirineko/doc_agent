use reqwest::header::{HeaderMap, RETRY_AFTER};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_DETAIL_BYTES: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    Auth,
    RateLimit,
    BadRequest,
    PayloadTooLarge,
    ContextLength,
    Server,
    Network,
    Timeout,
    StreamError,
    StreamIncomplete,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderFailure {
    pub kind: FailureKind,
    pub status: Option<u16>,
    pub provider_code: Option<String>,
    pub message: String,
    pub detail: Option<String>,
    pub retry_after_ms: Option<u64>,
}

impl std::fmt::Display for ProviderFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl FailureKind {
    pub fn retryable(self) -> bool {
        matches!(
            self,
            Self::RateLimit | Self::Server | Self::Network | Self::Timeout | Self::StreamIncomplete
        )
    }

    pub fn headline(self) -> &'static str {
        match self {
            Self::Auth => "模型服务鉴权失败",
            Self::RateLimit => "模型服务限流",
            Self::BadRequest => "模型服务拒绝了请求",
            Self::PayloadTooLarge => "请求体过大",
            Self::ContextLength => "对话上下文超出模型上限",
            Self::Server => "模型服务内部错误",
            Self::Network => "网络连接失败",
            Self::Timeout => "模型服务响应超时",
            Self::StreamError => "模型服务在响应中返回错误",
            Self::StreamIncomplete => "模型响应未正常结束",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Auth => "请在「密钥与服务」中检查该模型的 API Key",
            Self::RateLimit => "已自动重试仍失败，请稍后再试或切换模型",
            Self::ContextLength => "使用 /compact 压缩历史，或新建会话",
            Self::PayloadTooLarge => "缩小或减少图片附件后重试",
            Self::Network | Self::Timeout => "检查网络、系统代理或 TUN 后重试",
            _ => "可直接重新发送；若持续失败请切换模型",
        }
    }
}

impl ProviderFailure {
    pub fn new(kind: FailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            status: None,
            provider_code: None,
            message: message.into(),
            detail: None,
            retry_after_ms: None,
        }
    }

    pub fn from_status_body(status: u16, body: &str, retry_after_ms: Option<u64>) -> Self {
        let (msg, code) = serde_json::from_str::<Value>(body)
            .map(|v| parse_error_body(&v))
            .unwrap_or_default();
        let kind = classify_status(status, msg.as_deref(), code.as_deref());
        Self {
            status: Some(status),
            provider_code: code,
            detail: Some(truncate_detail(body)),
            retry_after_ms,
            ..Self::new(kind, msg.unwrap_or_else(|| kind.headline().to_string()))
        }
    }

    pub fn gemini_status(status: u16, message: impl Into<String>) -> Self {
        Self {
            status: Some(status),
            ..Self::new(classify_status(status, None, None), message)
        }
    }

    /// `redact`（Gemini）时只保留固定中文文案，不回显传输层错误原文。
    pub fn from_transport_err(err: &reqwest::Error, redact: bool) -> Self {
        let (kind, redacted) = if err.is_timeout() {
            (
                FailureKind::Timeout,
                "Gemini 请求超时，请检查系统代理或 TUN",
            )
        } else {
            (
                FailureKind::Network,
                "Gemini 网络连接失败，请检查系统代理、TUN 或网络",
            )
        };
        if redact {
            Self::new(kind, redacted)
        } else {
            Self {
                detail: Some(err.to_string()),
                ..Self::new(kind, err.to_string())
            }
        }
    }

    /// SSE 流中的 `{"error": …}` 帧。`redact`（Gemini）时不保留帧原文与 provider 消息。
    pub fn from_stream_error(frame: &Value, redact: bool) -> Self {
        let (msg, code) = parse_error_body(frame);
        let kind = if is_context_length(msg.as_deref(), code.as_deref()) {
            FailureKind::ContextLength
        } else {
            FailureKind::StreamError
        };
        if redact {
            return Self::new(kind, kind.headline());
        }
        Self {
            provider_code: code,
            detail: Some(truncate_detail(&frame.to_string())),
            ..Self::new(kind, msg.unwrap_or_else(|| kind.headline().to_string()))
        }
    }
}

/// 解析 OpenAI 兼容错误体：`{"error": {"message","code"}}` 或 `{"error": "..."}`，返回 (message, code)。
pub fn parse_error_body(v: &Value) -> (Option<String>, Option<String>) {
    let e = &v["error"];
    let message = e["message"]
        .as_str()
        .or_else(|| e.as_str())
        .map(str::to_string);
    let code = e["code"]
        .as_str()
        .map(str::to_string)
        .or_else(|| e["code"].as_i64().map(|n| n.to_string()));
    (message, code)
}

pub fn classify_status(status: u16, message: Option<&str>, code: Option<&str>) -> FailureKind {
    if is_context_length(message, code) {
        return FailureKind::ContextLength;
    }
    match status {
        401 | 403 => FailureKind::Auth,
        429 => FailureKind::RateLimit,
        413 => FailureKind::PayloadTooLarge,
        400 | 404 | 422 => FailureKind::BadRequest,
        500..=599 => FailureKind::Server,
        _ => FailureKind::BadRequest,
    }
}

/// 仅支持整数秒形式的 `Retry-After`；HTTP-date 形式返回 None 并回落默认退避。
pub fn parse_retry_after(headers: &HeaderMap) -> Option<u64> {
    let secs: u64 = headers
        .get(RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(secs.saturating_mul(1000))
}

pub fn truncate_detail(s: &str) -> String {
    const SUFFIX: &str = "…(truncated)";
    if s.len() <= MAX_DETAIL_BYTES {
        return s.to_string();
    }
    let end = crate::agent::compaction::floor_char_boundary(s, MAX_DETAIL_BYTES);
    format!("{}{SUFFIX}", &s[..end])
}

fn is_context_length(message: Option<&str>, code: Option<&str>) -> bool {
    let text = format!("{} {}", message.unwrap_or(""), code.unwrap_or("")).to_lowercase();
    text.contains("context length")
        || text.contains("context_length")
        || text.contains("maximum context")
        || text.contains("too many tokens")
        || text.contains("prompt is too long")
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};

    #[test]
    fn rate_limit_json_keeps_provider_message() {
        let body = r#"{"error":{"message":"Rate limit reached","type":"rate_limit_error","code":"rate_limit_exceeded"}}"#;
        let f = ProviderFailure::from_status_body(429, body, None);
        assert_eq!(f.kind, FailureKind::RateLimit);
        assert_eq!(f.status, Some(429));
        assert_eq!(f.provider_code.as_deref(), Some("rate_limit_exceeded"));
        assert!(f.message.contains("Rate limit reached"));
        assert!(f
            .detail
            .as_deref()
            .unwrap_or("")
            .contains("Rate limit reached"));
    }

    #[test]
    fn zhipu_numeric_code_is_stringified() {
        let frame = serde_json::json!({"error":{"code":1210,"message":"messages 过长"}});
        let f = ProviderFailure::from_stream_error(&frame, false);
        assert_eq!(f.kind, FailureKind::StreamError);
        assert_eq!(f.provider_code.as_deref(), Some("1210"));
        assert!(f.message.contains("messages 过长"));
        let redacted = ProviderFailure::from_stream_error(&frame, true);
        assert!(redacted.detail.is_none());
        assert!(redacted.provider_code.is_none());
        assert_eq!(redacted.message, FailureKind::StreamError.headline());
    }

    #[test]
    fn context_length_text_overrides_400() {
        let kind = classify_status(400, Some("maximum context length exceeded"), None);
        assert_eq!(kind, FailureKind::ContextLength);
        let body = r#"{"error":{"message":"This model's maximum context length is 128k tokens"}}"#;
        let f = ProviderFailure::from_status_body(400, body, None);
        assert_eq!(f.kind, FailureKind::ContextLength);
    }

    #[test]
    fn detail_truncates_at_2kb() {
        let body = "x".repeat(3000);
        let out = truncate_detail(&body);
        assert!(out.ends_with("…(truncated)"));
        assert!(out.len() > MAX_DETAIL_BYTES);
        assert!(out[..MAX_DETAIL_BYTES].bytes().all(|b| b == b'x'));
    }

    #[test]
    fn retry_after_integer_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("7"));
        assert_eq!(parse_retry_after(&headers), Some(7000));
    }

    #[test]
    fn retry_after_non_integer_is_none() {
        for raw in ["Wed, 21 Oct 2015 07:28:00 GMT", "7.5", "soon"] {
            let mut headers = HeaderMap::new();
            headers.insert(RETRY_AFTER, HeaderValue::from_static(raw));
            assert_eq!(parse_retry_after(&headers), None, "{raw}");
        }
    }
}
