## Why

Agent 循环运行中 provider 请求一旦失败，用户只看到一句 `http error: …` 拼在正文末尾，整轮立即中止；错误不落库、无分类、无重试。四类完全不同的故障（非 2xx、流内 `error` 对象、连接中断、流未正常结束）共用一个字符串，其中流内错误还把 provider 返回的 `code/message` 直接丢弃，只留固定英文句 `Provider returned an error in the response stream`。429 / 5xx / 网络抖动这类瞬时故障本可自动恢复，现在却让已跑十几步工具的 turn 付诸东流，且事后在数据库里查不到任何痕迹。

## What Changes

- **结构化 provider 错误**：`ProviderError::Http(String)` 改为携带 `ProviderFailure { kind, status, provider_code, message, detail, retryable }`；`kind` 枚举：`auth`、`rate_limit`、`bad_request`、`payload_too_large`、`context_length`、`server`、`network`、`timeout`、`stream_error`、`stream_incomplete`。非 2xx 响应体与流内 `error` 对象 MUST 解析出 `error.message` / `error.code` / `error.type`（best-effort），响应体截断到 2KB 放入 `detail`。Gemini 保持现有脱敏策略（仅 `kind` + `status`，无 `detail`）。
- **有限自动重试**：`rate_limit`、`server`、`network`、`timeout`、`stream_incomplete` 且**尚未向 UI 输出任何 token** 时，最多重试 2 次（退避 1s → 3s，429 遵守 `Retry-After`，上限 10s），重试期间发 `provider_retry` 事件；`auth`、`bad_request`、`payload_too_large`、`context_length` 不重试。取消信号在退避期间生效。
- **错误事件由后端发出并落盘**：最终失败时 Rust emit 扩展后的 `AgentEvent::Error { message, code, retryable, detail, hint }`（新增字段均 optional，向后兼容）并正常返回，不再依赖前端 catch 合成；同时追加一行 JSONL 到 `<app_data_dir>/logs/provider-errors.jsonl`（含 session_id、turn_id、model、kind、status、provider_code、message、detail、attempt），单文件超 1MB 轮转一份。
- **HTTP 超时**：所有 OpenAI 兼容 client `connect_timeout: 15s`；流式读取增加 90s 无数据空闲超时 → `kind: timeout`；非流式辅助请求（`complete_chat`）总超时 120s。
- **UI**：错误不再拼进正文，改为消息下方独立错误卡片（标题 = 按 `kind` 的中文文案、`hint`、可折叠 `detail`、「复制详情」按钮）；重试中显示「网络波动，正在重试（1/2）…」状态行。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-loop`：新增 provider 错误分类、有限重试、错误事件落盘、HTTP 超时四项 requirement。
- `workspace-ui`：新增 provider 错误卡片与重试状态行两项 requirement；`error` 事件类型契约扩展字段。

## Impact

- Rust：`agent/provider/mod.rs`（`ProviderError`、新 `ProviderFailure`）、新增 `agent/provider/failure.rs`（分类与响应体解析）、`openai_compat.rs`（非 2xx、client builder）、`openai_stream.rs`（流内 error 解析）、`sse.rs`（空闲超时）、`gemini/mod.rs`（映射到新结构）、`agent/loop_runner.rs`（重试包装 + emit Error + 落盘）、新增 `agent/provider_retry.rs`、`core/error_log.rs`、`state.rs`（暴露 `data_dir`）、`agent/types.rs`（`AgentEvent::Error` 扩展字段、新增 `ProviderRetry`）。
- 前端：`src/types.ts`、`src/lib/agentEvents.ts`（`error` 不再拼正文，改写 `turnError` 状态；新增 `provider_retry`）、`src/components/ChatPanel.tsx`（渲染错误卡片；如超 250 行抽 `TurnErrorCard.tsx`）、`src/hooks/useWorkspace.ts`（catch 分支仅处理 invoke 级失败）。
- 依赖：**无新增 crate**（`tokio::time::timeout`、`reqwest::ClientBuilder` 已可用）。
- 测试：`agent/provider/tests.rs` 用现有 mock SSE 覆盖分类；`agent/loop` Mock Provider 覆盖重试次数、不重试类别、已输出 token 不重试；前端 `agentEvents.test.ts` 与 ChatPanel 用例。
- 关联 backlog：BL-004「关键错误仅 console.error」由本 change 的 provider 部分覆盖；`spec.md` 条目更新为部分完成。
