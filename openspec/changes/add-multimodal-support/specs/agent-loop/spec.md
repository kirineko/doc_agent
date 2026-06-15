## ADDED Requirements

### Requirement: 多模态消息序列化

系统 SHALL 支持将 user 消息的 `content` 序列化为 OpenAI 兼容多模态数组（`text` + `image_url`）。`ChatMessage` 与 store 层 MUST 能承载文本与附件元数据；发往 Provider 前由 `messages_from_store`（或等价模块）将附件文件编码为 `data:{mime};base64,...`。

tool / assistant / system 消息在 MVP 中 MUST 保持字符串 content（`image_read` 子调用结果以纯文本 tool 消息回填）。

#### Scenario: 重建含附件的 user 消息

- **WHEN** store 中 user 消息含 `attachments_json` 与文本
- **THEN** 发往 Kimi 的 messages 数组中该条 user content 为含 `image_url` 的数组

### Requirement: Provider 输出 token 字段映射

当且仅当调用方显式指定输出 token 上限时，系统 SHALL 按 Provider 写入正确字段：DeepSeek → `max_tokens`；Kimi 与 MiMo → `max_completion_tokens`。主 Agent 循环的常规 chat 请求 MUST 省略二者（使用厂商默认，约 32K）。

#### Scenario: 主循环不传输出上限

- **WHEN** Agent loop 发起常规模型请求且未设置内部输出上限
- **THEN** 请求 body 不包含 `max_tokens` 也不包含 `max_completion_tokens`

#### Scenario: 压缩摘要使用正确字段

- **WHEN** 对 DeepSeek 发起压缩摘要且内部上限为 8192
- **THEN** body 含 `max_tokens: 8192` 且不含 `max_completion_tokens`

#### Scenario: 压缩摘要 Kimi 字段

- **WHEN** 对 Kimi 发起压缩摘要且内部上限为 8192
- **THEN** body 含 `max_completion_tokens: 8192`

### Requirement: 按模型动态工具列表

Agent loop 组装 `tools` 时 MUST 依据会话模型的 `supports_vision` 过滤工具；`supports_vision=false` 时不得包含 `image_read`。

#### Scenario: 工具列表随模型变化

- **WHEN** 同一会话锁定为 `mimo-v2.5-pro` 并开始新 turn（首条消息前已选模型）
- **THEN** 该会话全程工具定义不含 `image_read`

## MODIFIED Requirements

### Requirement: Agent 多轮工具调用循环

系统 SHALL 实现一个 Agent 执行循环：构造会话上下文后向模型发起流式请求，若返回包含工具调用则在沙箱内执行并将结果回填，再次请求模型，如此重复，直到模型返回不含工具调用的最终回答为止。**例外**：`clarify_ask` 工具调用进入 `awaiting_user` 状态时，loop 暂停等待 `submit_clarify_answer`，通过 `resume_turn` 继续，暂停期间不视为 turn 完成。循环 MUST 支持 user 多模态输入与 `image_read` 工具（vision 子调用）。

#### Scenario: 单轮工具调用后给出答案

- **WHEN** 用户提问需要读取一个文档，模型返回一个 `read_to_markdown` 工具调用
- **THEN** 系统在沙箱内执行该工具，将结果作为 `tool` 消息回填，并再次请求模型
- **AND** 模型基于工具结果返回最终回答时，循环结束

#### Scenario: 多轮连续工具调用

- **WHEN** 模型在一次回答中先后需要「列目录」再「读取某文件」
- **THEN** 系统按序执行每个工具调用、逐次回填，并持续循环直到模型不再请求工具

#### Scenario: 达到最大轮次保护

- **WHEN** 工具调用轮次达到配置的上限
- **THEN** 系统终止循环并向用户返回已产出的内容与「已达最大步数」提示

#### Scenario: clarify 暂停后继续

- **WHEN** 模型返回 `clarify_ask` 且用户提交答案
- **THEN** 系统通过 `resume_turn` 继续循环，且不计为新的用户 turn

#### Scenario: image_read 后继续推理

- **WHEN** vision 模型调用 `image_read` 并获得文本描述
- **THEN** 系统将文本作为 tool 结果回填并继续 loop，直至最终回答

### Requirement: vision 能力发送前校验

Agent loop 在持久化或发往 Provider 之前 MUST 校验 user 消息：若含 `attachments_json`（或等价附件元数据）且会话模型 `supports_vision=false`，MUST 拒绝并返回明确错误，不得仅依赖前端 toast。

#### Scenario: non-vision 会话拒绝多模态 user 消息

- **WHEN** 会话模型为 DeepSeek V4 Flash 且 `send_message` 含图片附件
- **THEN** loop 返回错误，消息不进入 store，不发起 Provider 请求
