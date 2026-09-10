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

Google 用量 SHALL 以其服务端 total_tokens 为本次基线，包含其报告的思考消耗；pending 和摘要估算 MUST 不重复计算协议元数据中的正文副本，不按签名字节数估算模型 token。

#### Scenario: Google 签名不造成重复估算
- **WHEN** assistant 正文和摘要同时存在于显示字段与协议元数据
- **THEN** pending 只统计一份可读内容，不把签名当文本累计

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
- **THEN** 下一次主 Agent 请求将该附件按 Chat Completions image_url 格式编码并发往 vision 模型

Google 的活动多步工具链 SHALL 从最近的真实 user 输入边界整体保留，包含所有 assistant/tool 消息及签名元数据；不得只保护最后一对工具消息。摘要请求 MUST 不包含协议元数据、签名或附件二进制。

#### Scenario: 多步工具链不被拆散
- **WHEN** Google 当前 user turn 已完成两次工具调用，第三次调用前触发压缩
- **THEN** 保留该 user 输入及整条活动链；只摘要更早已完成的会话段

#### Scenario: clarify pending 的签名保护
- **WHEN** 保留边界涉及尚待用户回答的 clarify 调用
- **THEN** 其签名元数据及整个活动链均不进入摘要段

## ADDED Requirements

### Requirement: 协议元数据随消息归档与重建

系统 SHALL 将协议元数据随对应消息归档，保留 tail 中的签名元数据原样参与后续请求。摘要 SHALL 成为新的普通上下文输入，不复用摘要生成请求的签名来替代原会话状态。原始归档消息不得物理删除。

#### Scenario: 旧 turn 完整摘要
- **WHEN** Google 早期已完成 turn 被摘要且后续 tail 保留
- **THEN** 下一请求包含摘要与完整 tail，不包含被归档旧步骤，也不制造孤立函数结果

#### Scenario: 无可压缩的活动链
- **WHEN** 所有历史均属于必须保留的活动工具链
- **THEN** 不为腾出空间删除签名或拆分调用，按现有无可压缩内容路径明确处理
