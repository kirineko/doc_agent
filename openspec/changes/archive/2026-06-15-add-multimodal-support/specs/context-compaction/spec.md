## MODIFIED Requirements

### Requirement: token 用量采集与 pending 估算

系统 SHALL 以 API 回报的精确 token 用量为权威计数，并对「自上次用量回报后新增、尚未发送给 API 的消息」用字符启发式（字符数 / 4）做 pending 估算。压缩触发判定 MUST 使用 `token_count + pending_estimate`。

- 每次 API 流式响应返回后，`token_count` MUST 更新为本次 `usage.total_tokens`，`pending_estimate` MUST 归零。
- 循环内每追加一条工具结果或新消息，`pending_estimate` MUST 累加该消息的**文本专用**估算值。
- pending 与压缩后估算路径 MUST 仅统计文本字段（`content`、`reasoning_content`、tool call 名/参数），**MUST NOT** 读取 `attachments_json` 对应文件或展开 base64。图片 token 以主 Agent loop 下一次 API `usage` 为权威值（对齐 kimi-cli `estimate_text_tokens` 忽略 `ImageURLPart`）。

#### Scenario: API 返回后刷新精确计数

- **WHEN** 一次 LLM 流式请求返回 `usage.total_tokens = 120_000`
- **THEN** `token_count` 更新为 120_000，`pending_estimate` 归零

#### Scenario: 大工具结果计入 pending 防撑爆

- **WHEN** API 上次回报 `token_count = 200_000`（模型上限 256_000），随后追加一条约 60_000 token 的工具结果尚未发出
- **THEN** 触发判定使用 200_000 + 估算 60_000 = 260_000，判定为需压缩（仅看 200_000 会漏判）

#### Scenario: 含附件 user 消息 pending 不含图片

- **WHEN** 用户发送含 1 张图片附件的 user 消息（`attachments_json` 非空）且 API 尚未回报
- **THEN** `pending_estimate` 仅累加该消息文本部分，不因附件文件或 base64 增大

### Requirement: 三段式压缩与工具调用配对完整性

系统 SHALL 以「摘要旧消息 + 保留最近若干轮原样」的三段式策略压缩上下文：

1. 从尾部保留最近 `max_preserved_messages` 条 user/assistant 消息（默认 2）原样（含 `attachments_json`）。
2. 切分保留起点时 MUST 保证 `tool_calls` 与其对应 `tool` 结果（以 `tool_call_id` 关联）不被拆分到压缩段与保留段两侧；必要时将保留起点前移以纳入完整配对。
3. 对压缩段发起一次不含工具的 LLM 摘要请求，使用结构化压缩 prompt；压缩 prompt 输入 MUST 为**纯文本**（不得含 base64 或 `image_url`）。
4. 用「摘要消息 + 保留消息」作为新的工作上下文。

被压缩段中的图片附件 MUST NOT 送入摘要 LLM；被摘要区的视觉信息不保留（仅文本进入摘要，与 kimi-cli `prepare()` 仅保留 `TextPart` 一致）。保留 tail 中的 `attachments_json` MUST 原样持久化并在后续 API 请求时重新编码。

当可压缩消息为空（如全部需保留）时，系统 MUST NOT 发起摘要请求。

#### Scenario: 保留最近两轮并摘要更早历史

- **WHEN** 上下文含 10 条消息且 `max_preserved_messages=2`
- **THEN** 最近 2 条 user/assistant 原样保留，更早消息被摘要为单条摘要消息

#### Scenario: 不拆散工具调用配对

- **WHEN** 保留起点恰好落在某 `tool` 结果与其上游 `tool_calls` 之间
- **THEN** 系统将保留起点前移，使该 `tool_calls` 与对应 `tool` 结果同处一侧

#### Scenario: 无可压缩消息不调用 LLM

- **WHEN** 消息总数不足以在保留最近若干轮后留下可压缩内容
- **THEN** 系统不发起摘要请求，上下文保持不变

#### Scenario: 压缩输入剥离图片附件

- **WHEN** 被压缩段含带 `attachments_json` 的 user 消息
- **THEN** 送入摘要 LLM 的输入仅含该消息文本与 tool 文本，不含 base64；保留 tail 中同类消息仍含 `attachments_json`

#### Scenario: 保留 tail 附件可再次发送

- **WHEN** 压缩完成且保留 tail 含图片附件
- **THEN** 下一次主 Agent 请求将该附件编码为 `image_url` 发往 vision 模型

### Requirement: 压缩与上下文用量事件

系统 SHALL 在以下时机 emit 事件供前端展示：

- `context_usage`：字段 `session_id`、`used_tokens`、`max_tokens`、`ratio`（`= used_tokens / max_tokens`，0~1）。在每次 API token 用量刷新后以及压缩完成后 emit。
- `context_compacted`：字段 `session_id`、`before_tokens`、`after_tokens`。在压缩成功后 emit。

压缩完成后的 `after_tokens` MUST 按「摘要 LLM `usage.output_tokens` + 保留消息文本估算」计算，**不得**将保留 tail 中的图片 base64 计入；下一次主 loop API usage 可校正。

#### Scenario: 用量刷新后通知前端比例

- **WHEN** 一次 LLM 响应返回并刷新精确 token 用量
- **THEN** 系统 emit `context_usage`，`ratio` 反映当前占用比例

#### Scenario: 压缩成功通知前端

- **WHEN** 一次自动压缩成功完成
- **THEN** 系统 emit `context_compacted`（含压缩前后 token 数），随后 emit 反映新占用的 `context_usage`
