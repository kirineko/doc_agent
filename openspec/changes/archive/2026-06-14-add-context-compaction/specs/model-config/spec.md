## ADDED Requirements

### Requirement: 模型上下文上限

系统 SHALL 为每个模型暴露上下文长度上限 `max_context_size`：DeepSeek 系列 = 1_000_000，Kimi K2.6 = 256_000，Mock = 100_000（便于测试触发小阈值）。该上限供压缩触发判定使用。

#### Scenario: DeepSeek 上限为 1M

- **WHEN** 当前会话模型为 DeepSeek V4 Flash 或 Pro
- **THEN** `max_context_size` 为 1_000_000

#### Scenario: Kimi 上限为 256K

- **WHEN** 当前会话模型为 Kimi K2.6
- **THEN** `max_context_size` 为 256_000

### Requirement: 流式响应 token 用量采集

系统 SHALL 在 OpenAI 兼容流式请求中携带 `stream_options.include_usage = true`，并在 SSE 解析中读取末尾包含 `usage` 的 chunk（`prompt_tokens`、`completion_tokens`、`total_tokens`），将其填入助手轮结果（`AssistantTurn`）。Mock Provider MUST 返回估算用量以贯通测试链路。

#### Scenario: 真实 Provider 回报用量

- **WHEN** DeepSeek/Kimi 流式响应在末尾返回 usage chunk
- **THEN** 系统解析出 `total_tokens` 并随该轮结果一并返回，供上下文计数刷新

#### Scenario: Mock Provider 提供估算用量

- **WHEN** 使用 Mock Provider 完成一轮响应
- **THEN** 返回非空的估算 usage，使压缩计数逻辑可在无真实 Key 时测试
