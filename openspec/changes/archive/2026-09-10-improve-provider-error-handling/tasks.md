## 1. 结构化错误（无行为变化）

- [x] 1.1 新建 `agent/provider/failure.rs`：`FailureKind`（含 `retryable()` / `headline()` / `hint()`）、`ProviderFailure`、`parse_error_body(&Value)`、`classify_status`、`ProviderFailure::from_transport_err`、`truncate_detail`、`parse_retry_after(headers) -> Option<u64>`。验证：单测覆盖 spec「Provider 错误结构化分类」六个 scenario 的纯函数部分（429 JSON、Zhipu 数字 code、context length 文本、2KB 截断、`Retry-After: 7` → 7000ms、非整数 Retry-After → None）。
- [x] 1.2 `agent/provider/mod.rs`：`ProviderError::Http(String)` → `Http(ProviderFailure)`，`Display` 输出 `message`；`sse.rs` 的 `SseError::Http` 同步改为 `ProviderFailure`。验证：`cargo build` 通过后修正所有编译错误处（`openai_compat.rs`、`openai_stream.rs`、`sse.rs`、`gemini/mod.rs`、`mock.rs`、`live_smoke.rs`、`tests.rs`）。
- [x] 1.3 `openai_compat.rs::send`：非 2xx 分支按 design D2 构造 `ProviderFailure`（非 Gemini 含 `detail` 与 `retry_after_ms`；Gemini `detail: None`）；传输错误用 `ProviderFailure::from_transport_err(&e, google)`。验证：`provider/tests.rs` 用 mock server 返回 429+Retry-After、400 context-length 文本、413，断言 `kind`/`status`/`provider_code`/`retry_after_ms`；Gemini 400 断言 `detail.is_none()`。
- [x] 1.4 `openai_stream.rs::apply`：流内 `error` 走 `ProviderFailure::from_stream_error(value, google)`，`kind: StreamError`（命中 context-length 规则则 `ContextLength`），`detail` 为该帧 JSON。`finish` 无 `finish_reason` → `StreamIncomplete`。验证：单测 `stream_error_keeps_provider_message`（帧 `{"error":{"code":"1210","message":"messages 过长"}}`）与 `missing_finish_reason_is_stream_incomplete`。
- [x] 1.5 `sse.rs` chunk 错误 → `Network`。既有测试中 `to_string().contains("400")` 类断言改为断言 `kind`/`status`。验证：`cargo test provider` 全绿，`cargo clippy -- -D warnings` 无告警。

## 2. 事件、落盘与前端卡片

- [x] 2.1 `agent/types.rs`：`AgentEvent::Error` 新增 `code: Option<FailureKind>`、`retryable: Option<bool>`、`detail: Option<String>`、`hint: Option<String>`（`skip_serializing_if`）；新增 `ProviderRetry { session_id, turn_id, attempt, max, kind, delay_ms }`。验证：序列化单测：新字段全为 `None` 的 `Error` 输出 JSON 与旧格式逐字节一致；`ProviderRetry` 序列化为 `"kind":"provider_retry"`，其中 `FailureKind` 字段为 snake_case（如 `"rate_limit"`）。
- [x] 2.2 新建 `core/error_log.rs`：`append(data_dir, &impl Serialize)`，1MB 轮转；`state.rs` `AppState` 新增 `pub data_dir: PathBuf`。验证：单测在 tempdir 追加两行并解析；预写 1MB+ 文件后追加，断言产生 `.1.jsonl` 且新文件仅一行；`core/` 不引入 tauri/reqwest 依赖（`rg "use tauri|use reqwest" src-tauri/src/core/error_log.rs` 为空）。
- [x] 2.3 `loop_runner.rs`：provider `Http` 失败分支改为 design D3 末段——`error_log::append` + emit 扩展 `Error` + `return Ok(())`；定义 `ProviderErrorRecord { ts, session_id, turn_id, model, attempts, #[serde(flatten)] failure }`（JSON 行含 kind/status/provider_code/message/detail）。验证：Mock Provider 返回 `auth` 失败，断言 `send_message` 返回 `Ok`、收到 `Error` 事件 `code == Some(Auth)`、日志文件新增一行。
- [x] 2.4 `src/types.ts`：扩展 `error` 事件类型与新增 `provider_retry`；`FailureKind` 字面量联合类型。验证：`npm run typecheck`。
- [x] 2.5 `src/lib/agentEvents.ts`：`error` 分支写 `turnError` 不再拼 `streamingContent`；`provider_retry` 写 `retryNotice`；token/`tool_call_stream`/`turn_complete` 清空 `retryNotice`；发送新消息时 `markAgentBusy` 顺带清空 `turnError`（无需单独 action）。验证：`agentEvents.test.ts` 新增 4 用例（结构化 error、旧格式 error、retryNotice 生命周期、streamingContent 不变）。
- [x] 2.6 新建 `src/components/TurnErrorCard.tsx`（≤150 行）：标题 / hint / `<details>` detail / 「复制详情」（`navigator.clipboard.writeText`）；`ChatPanel.tsx` 渲染卡片与 `retryNotice` 状态行，`useWorkspace.sendMessage` 的 `busy` dispatch 即清空上一轮 `turnError`。验证：Testing Library 用例——鉴权失败卡片文案与展开、复制按钮调用 clipboard mock、重试状态行文本「网络波动，正在重试（1/2）…」；`ChatPanel.tsx` 保持 ≤250 行。
- [x] 2.7 `useWorkspace.ts` catch 分支：仅当 error 不是 provider 失败（现在 provider 失败已 resolve）时合成本地 `error`；保留 `isParallelLimitError` 处理。验证：现有 `useWorkspace` 测试通过；新增用例 invoke reject（如 `消息不能为空`）仍展示错误卡片。

## 3. 重试与超时

- [x] 3.1 新建 `agent/provider_retry.rs`：`RetryPolicy`（`Default` 为纯常量，`from_env()` 读取 env `DOC_AGENT_PROVIDER_RETRIES` 覆盖 `max_retries`，loop 入口读一次）、`chat_stream_with_retry`（`ProviderRetry` 经 `on_event` 发出，退避用 `sse::cancelable` 包 `sleep`）。验证：单测用 `MockProvider` 脚本化失败序列覆盖 spec「瞬时错误有限重试」五个 scenario；`RetryPolicy` 测试 env `=0` 时不重试。
- [x] 3.2 `loop_runner.rs` 调用点切换到 `chat_stream_with_retry`，`ProviderErrorRecord.attempts` 取返回的 attempts。验证：2.3 的测试改为断言 `attempts` 字段；agent-loop 既有 Mock 多轮测试全绿。
- [x] 3.3 `openai_compat.rs`：`Client::builder().connect_timeout(15s)`；`complete_chat` 请求 `.timeout(120s)`；`gemini/new_google_http_client` 加 `connect_timeout`。验证：单测断言 client 构建成功；`live_smoke.rs`（ignored）路径编译通过。
- [x] 3.4 `sse.rs`：读循环包 `tokio::time::timeout(IDLE_TIMEOUT, stream.next())`，任何字节（含注释行）重置计时；超时 → `ProviderFailure::new(Timeout, "{idle 秒数} 秒内未收到任何数据")`。`IDLE_TIMEOUT` 抽为 `pub(crate) const` 并允许测试注入更短值（`read_stream_with_idle(…, Duration)`）。验证：mock stream 用 `tokio::time::pause` + 短 idle（200ms）覆盖「空闲超时」与「keep-alive 重置计时」两个 scenario。
- [x] 3.5 `provider/tests.rs` 端到端：mock server 先 503 再 200 → loop 成功且收到一次 `ProviderRetry`；mock server 返回 `data: {"choices":[{"delta":{"content":"hi"}}]}` 后断开 → 不重试、`Error.code == StreamIncomplete`、UI 已收到 `hi`。验证：两用例通过。

## 4. 文档与收尾

- [x] 4.1 `openspec/specs/project-backlog/spec.md`：BL-004 追加「2026-09 `improve-provider-error-handling` 覆盖 provider 失败路径；send/load 其余 `.catch(console.error)` 仍待办」。验证：diff 仅一处追加。
- [x] 4.2 `README.md` 或 `docs/`（若有故障排查章节）追加 `provider-errors.jsonl` 位置：macOS `~/Library/Application Support/com.kirineko.doc-agent/logs/`、Windows `%APPDATA%\com.kirineko.doc-agent\logs\`。验证：文件存在且路径与 `app_data_dir` 实际一致。
- [x] 4.3 全量门禁：`cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`；`npm run typecheck && npm test && npm run build`。验证：全部通过。
- [x] 4.4 手动验证：(a) 填错 API Key 发消息 → 卡片「模型服务鉴权失败」+ hint，无重试；(b) 断网发消息 → 状态行「正在重试（1/2）」后卡片「网络连接失败」；(c) 检查 `logs/provider-errors.jsonl` 两条记录。验证：截图附 PR。

- [x] 4.5 修复 review 发现的取消后重试提示残留：`turn_cancelled` 清空 `retryNotice`，补充 `provider_retry → turn_cancelled` reducer 回归测试并同步设计与 spec。验证：`npm run typecheck`、358 项前端测试、`npm run build` 通过。
