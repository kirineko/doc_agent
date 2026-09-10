## Context

见 `proposal.md - Why`。当前错误链路：

```
provider 非2xx --> Http("{status}: {body}")            \
流内 {"error"} --> Http("Provider returned an error…")  |  ProviderError::Http(String)
连接中断      --> Http(reqwest.to_string())             |        |
无finish_reason -> Http("stream ended before…")        /        v
                                          loop_runner: Err(e) => return Err(e.to_string())
                                                                 |
                                          ipc::send_message invoke 失败 (reject)
                                                                 |
                                          useWorkspace catch -> 本地合成 {kind:"error", message}
                                                                 |
                                          agentEvents: streamingContent += "\n\n> " + message
```

约束：

- `ChatRequest` 已 `#[derive(Clone)]`，可重放；`LlmProvider::chat_stream(request, api_key, on_event)` 按值消费 request。
- `AgentEvent` 用 `#[serde(tag = "kind", rename_all = "snake_case")]`，前端 `src/types.ts` 手写对应联合类型；新增字段用 `Option` + `#[serde(skip_serializing_if)]` 即向后兼容。
- `ActiveTurnGuard` 的 `Drop` 负责 `turns.unregister`，所以 `continue_loop_inner` 无论 `Ok`/`Err` 返回都会清理；改为 emit Error 后 `Ok(())` 不影响清理（与 `Reached maximum tool steps` 路径一致）。
- Gemini 已有脱敏策略（响应体可能回显请求元数据/凭据），新结构必须保留该边界。
- `OpenAiCompatClient::new` 用 `Client::new()`（无任何超时）；Gemini 走 `new_google_http_client()`。
- `AppState` 目前不保存 `data_dir`；落盘日志需要它。

## Goals / Non-Goals

**Goals:**

- 用户一眼看出「是谁的问题、能不能重试、该做什么」。
- 瞬时故障在用户无感知（或仅看到一行状态）的情况下自愈。
- 每次 provider 失败都有一条可事后检索的记录。

**Non-Goals:**

- 有 token 已输出后的「续写式」恢复（需要 provider 支持或复杂去重）。
- 前端「一键重发」按钮（后续 UX change）。
- 工具执行失败的重试（工具错误已回传模型自修，不在此范围）。
- 通用日志框架（`tracing`）引入——本次只做单一 JSONL 文件。

## Decisions

### D1. `ProviderFailure` 结构与分类

```rust
// agent/provider/failure.rs（新文件）
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    Auth, RateLimit, BadRequest, PayloadTooLarge, ContextLength,
    Server, Network, Timeout, StreamError, StreamIncomplete,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderFailure {
    pub kind: FailureKind,
    pub status: Option<u16>,
    pub provider_code: Option<String>,
    pub message: String,          // 面向用户的一句话（中文，按 kind 生成，可含 provider message 摘要）
    pub detail: Option<String>,   // 原始响应体/传输错误，≤2KB；Gemini 为 None
    pub retry_after_ms: Option<u64>,
}

impl FailureKind {
    pub fn retryable(self) -> bool {
        matches!(self, Self::RateLimit | Self::Server | Self::Network | Self::Timeout | Self::StreamIncomplete)
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

/// 解析 OpenAI 兼容错误体：{"error": {"message","code"}} 或 {"error": "..."}，返回 (message, code)。
/// 入参为已解析的 `serde_json::Value`，HTTP 错误体与 SSE 错误帧共用，避免 Value→String→Value 往返。
pub fn parse_error_body(v: &serde_json::Value) -> (Option<String>, Option<String>) {
    let e = &v["error"];
    let message = e["message"].as_str().or_else(|| e.as_str()).map(str::to_string);
    let code = e["code"].as_str().map(str::to_string)
        .or_else(|| e["code"].as_i64().map(|n| n.to_string()));
    (message, code)
}

pub fn classify_status(status: u16, message: Option<&str>, code: Option<&str>) -> FailureKind {
    let text = format!("{} {}", message.unwrap_or(""), code.unwrap_or("")).to_lowercase();
    if text.contains("context length") || text.contains("context_length") || text.contains("maximum context")
        || text.contains("too many tokens") || text.contains("prompt is too long") {
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

impl ProviderFailure {
    pub fn new(kind: FailureKind, message: impl Into<String>) -> Self;               // 其余字段 None
    pub fn from_status_body(status: u16, body: &str, retry_after_ms: Option<u64>) -> Self;
    pub fn gemini_status(status: u16, message: impl Into<String>) -> Self;          // detail 恒为 None
    pub fn from_transport_err(err: &reqwest::Error, redact: bool) -> Self;          // is_timeout → Timeout，否则 Network；redact 用固定中文文案
    pub fn from_stream_error(frame: &serde_json::Value, redact: bool) -> Self;      // StreamError / ContextLength；redact 不带 detail 与 provider 消息
}

pub const MAX_DETAIL_BYTES: usize = 2048;
pub fn truncate_detail(s: &str) -> String { /* 复用 compaction::floor_char_boundary 截到 2KB，尾部加 "…(truncated)" */ }
```

```rust
// agent/provider/mod.rs
pub enum ProviderError {
    #[error("missing api key")] MissingApiKey,
    #[error("{}", .0.message)] Http(ProviderFailure),   // 从 String 改为结构
    #[error("parse error: {0}")] Parse(String),
    #[error("cancelled")] Cancelled,
}
```

`Display` 输出 `message`，保证所有现有 `e.to_string()` 调用点（含 mock/测试）无需同步修改。

**替代**：新增变体 `Failed(ProviderFailure)` 与 `Http(String)` 并存——留下两条路径，后续必然漂移；不采用。

### D2. 各失败点的映射

| 位置 | 现状 | 改为 |
|---|---|---|
| `openai_compat.rs::send` 非 2xx（非 Gemini） | `Http(format!("{status}: {text}"))` | `parse_error_body(&text)` → `classify_status` → `ProviderFailure { detail: Some(truncate_detail(&text)), retry_after_ms: 解析 Retry-After 头 }` |
| 同上，Gemini | 固定中文 + status | `ProviderFailure { kind: classify_status(status, None, None), status, message: 现有中文, detail: None }` |
| `send` 传输错误 | `Http(e.to_string())` / `map_transport_error` | `ProviderFailure::from_transport_err(&e, google)`；Gemini `detail: None`，其他 `detail: Some(e.to_string())` |
| `openai_stream.rs::apply` 流内 `error` | 固定英文句 | `ProviderFailure::from_stream_error(value, google)` → `kind: StreamError`（若 message 命中 context-length 规则则 `ContextLength`），非 Gemini `detail` 为该 JSON 帧 |
| `sse.rs` chunk 读取错误 | `e.to_string()` | `Network` |
| `sse.rs` 90s 无 chunk | 无 | `Timeout`，message「90 秒内未收到任何数据」 |
| `openai_stream.rs::finish` 无 `finish_reason` | 英文句 | `StreamIncomplete` |
| `sse_frames` 半截帧 | `Json(...)` | 保持 `Parse`（不重试） |

`SseError` 同步改为 `Http(ProviderFailure)`，`From<SseError> for ProviderError` 直接搬运。

### D3. 重试包装

```rust
// agent/provider_retry.rs（新文件）
pub struct RetryPolicy { pub max_retries: u32, pub base_delay_ms: u64, pub max_delay_ms: u64 }
impl Default for RetryPolicy { fn default() -> Self { Self { max_retries: 2, base_delay_ms: 1000, max_delay_ms: 10_000 } } }
impl RetryPolicy { pub fn from_env() -> Self { /* DOC_AGENT_PROVIDER_RETRIES 覆盖 max_retries；loop 入口读一次 */ } }

// 不依赖 AppHandle：ProviderRetry 与 token 事件同走 on_event（loop_runner 的 on_event 对非 token 事件透传 emit）
pub async fn chat_stream_with_retry(
    provider: &dyn LlmProvider,
    request: ChatRequest,
    api_key: Option<&str>,
    policy: &RetryPolicy,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> Result<AssistantTurn, (ProviderError, u32 /* attempts */)> {
    let mut attempt = 0u32;
    loop {
        let emitted = AtomicBool::new(false);
        let result = {
            let mut tap = |ev: AgentEvent| {
                if matches!(ev, AgentEvent::ContentToken { .. } | AgentEvent::ReasoningToken { .. } | AgentEvent::ToolCallStream { .. }) {
                    emitted.store(true, Ordering::Relaxed);
                }
                on_event(ev);
            };
            provider.chat_stream(request.clone(), api_key, &mut tap).await
        };
        match result {
            Ok(turn) => return Ok(turn),
            Err(ProviderError::Http(f)) if f.kind.retryable() && !emitted.load(Ordering::Relaxed) && attempt < policy.max_retries => {
                attempt += 1;
                let delay = f.retry_after_ms.unwrap_or(policy.base_delay_ms * 3u64.pow(attempt - 1)).min(policy.max_delay_ms);
                on_event(AgentEvent::ProviderRetry {
                    session_id: request.session_id.clone(), turn_id: request.turn_id.clone(),
                    attempt, max: policy.max_retries, kind: f.kind, delay_ms: delay,
                });
                // 复用 sse::cancelable：退避期间可被 CancelSignal 打断
                if sse::cancelable(request.cancel.as_ref(), tokio::time::sleep(Duration::from_millis(delay))).await.is_err() {
                    return Err((ProviderError::Cancelled, attempt));
                }
            }
            Err(e) => return Err((e, attempt)),
        }
    }
}
```

- **为什么「已输出 token 就不重试」**：重放会让 UI 出现重复正文；provider 也可能已计费。`ToolCallStream` 也算已输出（工具参数正在流）。
- **为什么退避 1s/3s 而非指数到 30s**：桌面交互场景，用户在等；两次共 4s 内决出胜负。429 的 `Retry-After` 优先但上限 10s。
- **替代**：在 `OpenAiCompatClient` 内部重试——看不到 `on_event` 是否已输出，且 Gemini/Mock 各自实现要重复；放在 loop 层一处解决。
- `loop_runner` 调用点：

```rust
let retry_policy = RetryPolicy::from_env();   // for 循环外读一次
let turn = match chat_stream_with_retry(provider.as_ref(), request, api_key.as_deref(), &retry_policy, &mut on_event).await {
    Ok(turn) => turn,
    Err((ProviderError::Cancelled, _)) => { finish_cancelled(...); return Ok(()); }
    Err((ProviderError::Http(f), attempts)) => {
        // ProviderErrorRecord { ts, session_id, turn_id, model, attempts, #[serde(flatten)] failure: &f }
        error_log::append(&state.data_dir, &record);
        emit(&app, AgentEvent::Error {
            session_id, turn_id,
            message: format!("{}：{}", f.kind.headline(), f.message),
            code: Some(f.kind), retryable: Some(f.kind.retryable()),
            detail: f.detail.clone(), hint: Some(f.kind.hint().into()),
        });
        return Ok(());   // 与 max steps 路径一致；前端不再需要 catch 合成
    }
    Err((e, _)) => return Err(e.to_string()),   // MissingApiKey / Parse 维持现状
};
```

### D4. 事件契约与落盘

```rust
// agent/types.rs
Error {
    session_id: String, turn_id: String, message: String,
    #[serde(skip_serializing_if = "Option::is_none")] code: Option<FailureKind>,
    #[serde(skip_serializing_if = "Option::is_none")] retryable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")] detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] hint: Option<String>,
},
ProviderRetry { session_id: String, turn_id: String, attempt: u32, max: u32, kind: FailureKind, delay_ms: u64 },
```

```rust
// core/error_log.rs（新文件，无 HTTP/Tauri 依赖）
pub fn append(data_dir: &Path, record: &impl Serialize) -> std::io::Result<()> {
    let dir = data_dir.join("logs");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("provider-errors.jsonl");
    if path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false) {
        let _ = std::fs::rename(&path, dir.join("provider-errors.1.jsonl"));
    }
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{}", serde_json::to_string(record)?)
}
```

`AppState` 新增 `pub data_dir: PathBuf`。落盘失败仅 `eprintln!`，不影响事件发出。

**替代**：写入 SQLite `messages` 表（role=assistant, archived=1）——会进入 `list_messages` 并可能被上下文重建误用；单独建表则要迁移。JSONL 零迁移、可直接 `tail -f`。

### D5. 前端呈现

```ts
// agentEvents.ts
case "error":
  return { ...state, busy: false, turnError: { message, code, detail, hint, retryable } }; // 不再拼 streamingContent
case "provider_retry":
  return { ...state, retryNotice: { attempt, max, kind, delayMs } };
// 任何 content/reasoning token、turn_complete 或 turn_cancelled 清空 retryNotice；新 user 消息发送时清空 turnError
```

`ChatPanel` 在流式区域之后渲染 `TurnErrorCard`（标题 = `message`，`hint` 次级文字，`detail` `<details>` 折叠 + 「复制详情」），`retryNotice` 渲染为一行灰色状态「网络波动，正在重试（1/2）…」。`useWorkspace.sendMessage` 的 catch 分支保留，用于 `send_message` 在进入 loop 之前的 invoke 级失败（附件校验、并行上限等）。

### D6. HTTP 超时

```rust
// openai_compat.rs
pub fn new(chat_url: impl Into<String>) -> Self {
    Self { chat_url: chat_url.into(), client: Client::builder().connect_timeout(Duration::from_secs(15)).build().expect("reqwest client") }
}
// sse.rs read loop
const IDLE_TIMEOUT: Duration = Duration::from_secs(90);
while let Some(chunk) = cancelable(cancel, tokio::time::timeout(idle, stream.next())).await?
    .map_err(|_| SseError::Http(ProviderFailure::new(FailureKind::Timeout, format!("{} 秒内未收到任何数据", idle.as_secs()))))? { ... }
// complete_chat：.timeout(Duration::from_secs(120)) 于 RequestBuilder
```

`gemini/new_google_http_client` 同样加 `connect_timeout`。

## Risks / Trade-offs

- [重试导致 provider 双倍计费] → 仅在未收到任何 token 时重试，且上限 2 次；`provider_retry` 事件让用户可见。
- [`Retry-After` 为 HTTP-date 格式] → 仅解析整数秒；解析失败回落默认退避。
- [响应体 `detail` 含敏感信息] → Gemini 保持 `None`；其他 provider 截断 2KB 且只在折叠区展示；落盘文件位于用户本机 app data。
- [前端不再拼正文，旧版本后端配新前端] → 新字段 optional，旧事件无 `code` 时卡片标题回落 `message`；`applyAgentEvent` 对无 `code` 的 error 保持可渲染。
- [90s 空闲超时误伤长思考模型] → DeepSeek/Kimi 思考期间仍持续发 `reasoning_content` 帧或 keep-alive 注释行（`sse_frames` 会 push 到 buffer，需确认注释帧也重置计时——以「收到任意字节」为准而非「收到 data 帧」）。
- [`ProviderError::Http` 签名变化波及测试] → `Display` 保持输出 `message`，`tests.rs` 中 `contains("400")` 类断言需改为断言 `kind`/`status`；tasks 中列出。

## Migration Plan

1. D1 + D2（结构化，无行为变化，`Display` 兼容）→ 合并。
2. D4 + D5（后端 emit、落盘、前端卡片）→ 合并；此时用户已能看到分类与 hint。
3. D3 重试 + D6 超时 → 合并。
4. 回滚：每步独立 revert；D3 有 `RetryPolicy { max_retries: 0 }` 可作为紧急关闭开关（tasks 中加 env `DOC_AGENT_PROVIDER_RETRIES` 覆盖）。

## Open Questions

- 空闲超时 90s 是否需要按模型差异化（Gemini 3.8 深度思考可能更久）：先统一 90s，观察日志中 `timeout` 记录再调，不影响 spec。
- 是否把 `provider-errors.jsonl` 暴露到设置抽屉「导出诊断」：后续 UX change。
