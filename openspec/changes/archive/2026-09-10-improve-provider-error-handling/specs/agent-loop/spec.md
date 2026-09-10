## ADDED Requirements

### Requirement: Provider 错误结构化分类

Provider 请求失败 MUST 被归类为以下之一：`auth`（401/403）、`rate_limit`（429）、`payload_too_large`（413）、`context_length`（错误文本命中上下文超限特征）、`bad_request`（其余 4xx）、`server`（5xx）、`network`（连接失败）、`timeout`（连接/空闲超时）、`stream_error`（流内 `error` 对象）、`stream_incomplete`（流结束但无 `finish_reason`）。分类结果 MUST 携带 HTTP 状态码（若有）、provider 错误码（若响应体含 `error.code`）、面向用户的一句话消息、以及不超过 2KB 的原始详情。对 OpenAI 兼容响应体与流内 `error` 对象 MUST 尽力解析 `error.message` / `error.code` / `error.type`，MUST NOT 以固定文案替换 provider 消息。Gemini 的详情 MUST 为空（维持现有脱敏策略）。

#### Scenario: 429 带 provider 消息

- **WHEN** provider 返回 `429` 且响应体为 `{"error":{"message":"Rate limit reached","type":"rate_limit_error","code":"rate_limit_exceeded"}}`
- **THEN** 错误分类为 `rate_limit`，状态码 429，provider 错误码 `rate_limit_exceeded`，消息包含 `Rate limit reached`

#### Scenario: 流内 error 保留 provider 消息

- **WHEN** SSE 帧 `data: {"error":{"code":"1210","message":"参数非法：messages 过长"}}`
- **THEN** 错误分类为 `stream_error`，provider 错误码 `1210`，消息包含 `messages 过长`

#### Scenario: 上下文超限识别

- **WHEN** provider 返回 400 且 `error.message` 含 `maximum context length`
- **THEN** 错误分类为 `context_length` 而非 `bad_request`

#### Scenario: 连接失败

- **WHEN** 请求因 DNS 失败或连接被拒而未收到响应
- **THEN** 错误分类为 `network`，详情包含传输层错误文本

#### Scenario: Gemini 详情脱敏

- **WHEN** Gemini 返回 400 且响应体含请求元数据
- **THEN** 错误分类为 `bad_request`，状态码 400，详情为空

#### Scenario: 详情截断

- **WHEN** 非 2xx 响应体超过 2KB
- **THEN** 详情截断到 2KB 并以 `…(truncated)` 结尾

### Requirement: 瞬时错误有限重试

Agent 循环对分类为 `rate_limit`、`server`、`network`、`timeout`、`stream_incomplete` 的失败，且本次请求尚未向 UI 输出任何 `content_token` / `reasoning_token` / `tool_call_stream` 事件时，MUST 用相同请求自动重试，最多 2 次；退避为 1s、3s，`429` 携带整数秒 `Retry-After` 时优先使用该值，任何退避 MUST NOT 超过 10s。每次重试前 MUST emit `provider_retry` 事件（含 `attempt`、`max`、`kind`、`delay_ms`）。`auth`、`bad_request`、`payload_too_large`、`context_length`、`stream_error` MUST NOT 重试。退避期间收到取消信号 MUST 立即以 `turn_cancelled` 结束。重试次数 MAY 通过环境变量 `DOC_AGENT_PROVIDER_RETRIES` 覆盖（`0` 关闭）。

#### Scenario: 5xx 两次后成功

- **WHEN** Mock Provider 前两次返回 `server` 失败、第三次成功
- **THEN** 循环收到成功的 turn，期间 emit 两次 `provider_retry`（attempt 1、2），无 `error` 事件

#### Scenario: 超过重试上限

- **WHEN** Mock Provider 连续三次返回 `network` 失败
- **THEN** 循环在第二次重试失败后 emit `error` 事件，`code` 为 `network`，`retryable` 为 true

#### Scenario: 已输出 token 不重试

- **WHEN** Mock Provider 先 emit 一个 `content_token` 再以 `stream_incomplete` 失败
- **THEN** 循环 MUST NOT 重试，直接 emit `error` 事件，已输出的 token 不重复

#### Scenario: 4xx 不重试

- **WHEN** Mock Provider 返回 `bad_request`
- **THEN** 循环立即 emit `error` 事件，无 `provider_retry`

#### Scenario: 退避期间取消

- **WHEN** 第一次重试退避期间用户 `cancel_turn`
- **THEN** 循环 emit `turn_cancelled`，不再发起请求

### Requirement: Provider 错误事件与落盘

Provider 最终失败时，后端 MUST emit `error` 事件（`message` 为「分类标题：provider 消息」，附 `code`、`retryable`、`detail`、`hint`）并以正常返回结束 `send_message` / `resume_turn`，MUST NOT 以 invoke 拒绝的方式把错误抛给前端。同时 MUST 追加一行 JSON 到 `<app_data_dir>/logs/provider-errors.jsonl`，字段至少含 `ts`、`session_id`、`turn_id`、`model`、`kind`、`status`、`provider_code`、`message`、`detail`、`attempts`；文件超过 1MB 时 MUST 轮转为 `provider-errors.1.jsonl`（仅保留一份）。落盘失败 MUST NOT 阻止事件发出。

#### Scenario: 错误事件字段

- **WHEN** provider 返回 401
- **THEN** 前端收到 `error` 事件，`code` 为 `auth`，`retryable` 为 false，`hint` 提示检查 API Key，`send_message` invoke 正常 resolve

#### Scenario: 日志追加

- **WHEN** 任意 provider 失败结束一个 turn
- **THEN** `logs/provider-errors.jsonl` 新增一行，可被 `serde_json` 解析且 `session_id` 与该 turn 一致

#### Scenario: 日志轮转

- **WHEN** `provider-errors.jsonl` 已超过 1MB 时再次追加
- **THEN** 旧文件重命名为 `provider-errors.1.jsonl`，新文件仅含本次一行

### Requirement: Provider HTTP 超时

所有 provider HTTP 客户端 MUST 设置 15s 连接超时；流式响应 MUST 设置 90s 空闲超时（自上次收到任意字节起计，含 SSE 注释/keep-alive 行），触发时分类为 `timeout`；非流式辅助请求 MUST 设置 120s 总超时。

#### Scenario: 流式空闲超时

- **WHEN** Mock SSE 服务器发送首帧后停止输出超过 90s
- **THEN** 请求以 `timeout` 失败，且因已输出 token 不重试，emit `error`

#### Scenario: keep-alive 重置计时

- **WHEN** Mock SSE 服务器每 30s 发送一行 `: keep-alive` 注释持续 3 分钟后再发数据帧
- **THEN** 请求不触发空闲超时并正常完成

#### Scenario: 连接超时

- **WHEN** 目标地址不可达（黑洞 IP）
- **THEN** 请求在约 15s 内以 `timeout` 失败并进入重试
