## MODIFIED Requirements

### Requirement: Agent 多轮工具调用循环

系统 SHALL 实现一个 Agent 执行循环：构造会话上下文后向模型发起流式请求，若返回包含工具调用则在沙箱内执行并将结果回填，再次请求模型，如此重复，直到模型返回成功终态且不含工具调用的最终回答为止；不完整、失败或断流 MUST 不视为成功完成，不执行未完整校验的调用。**例外**：`clarify_ask` 工具调用进入 `awaiting_user` 状态时，loop 暂停等待 `submit_clarify_answer`，通过 `resume_turn` 继续，暂停期间不视为 turn 完成。循环 MUST 支持 user 多模态输入与 `image_read` 工具（vision 子调用）。

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

### Requirement: 思考与正文的流式分离输出
系统 SHALL 在流式响应中分别累积 provider 的可读思考内容（包括 Google thought summary）与正文，并以独立事件推送给前端。

#### Scenario: 思考与正文分区展示
- **WHEN** 模型处于思考模式并流式返回 `reasoning_content` 与 `content`
- **THEN** 系统先以「思考」事件推送思考增量、再以「正文」事件推送回答增量，二者不混淆

#### Scenario: Google 无摘要只有签名
- **WHEN** Google 返回 signature 但无 summary
- **THEN** 仍保留内部协议元数据，不向 UI 输出签名或虚构思考文本

### Requirement: 工具调用轮的 reasoning_content 回填
系统 SHALL 在持久化 assistant 消息时一并存储其 `reasoning_content`；在构造后续请求时，对 DeepSeek/MiMo/Kimi/GLM「包含工具调用的 assistant 消息」必须回传其 `reasoning_content`，Google 回传独立签名元数据。

#### Scenario: 含工具调用的轮次正确回填
- **WHEN** 某轮 assistant 消息包含 `tool_calls` 与 `reasoning_content`，且需要继续请求模型
- **THEN** 系统在后续请求中携带该 `reasoning_content`，使模型不返回 400 错误

对于 DeepSeek/MiMo/Kimi/GLM，已有可用 reasoning_content SHALL 在工具循环历史中保留，包括 DeepSeek 带 tools 请求所需的历史非工具 assistant 内容。Google SHALL 回传兼容工具消息和 extra_content 签名，不发送 reasoning_content，不把可读摘要当作签名替代品。

#### Scenario: Google 恢复完整步骤
- **WHEN** Google 工具调用后继续请求
- **THEN** 使用普通 assistant/tool 消息及签名元数据，不仅回传正文和摘要

### Requirement: Assistant 逐步持久化事件
系统 SHALL 在 Agent 循环中每次成功 `persist_assistant` 写入 assistant 消息后，向客户端 emit `assistant_step_done` 事件；payload MUST 包含 `session_id`、`turn_id` 与刚持久化的用户可见 assistant 消息（含 `id`、`content`、`reasoning_content` 等字段，与 `list_messages` 单条结构一致）。该事件 MUST 在工具执行之前发出（含工具调用轮与最终回答轮）。

#### Scenario: 含工具调用的轮次逐步通知
- **WHEN** 模型返回带 `tool_calls` 的 assistant 回答并已持久化
- **THEN** 系统在执行任何工具之前 emit `assistant_step_done`，且消息内容与 DB 一致

#### Scenario: 最终回答轮逐步通知
- **WHEN** 模型返回不含工具调用的最终 assistant 回答并已持久化
- **THEN** 系统在 emit `turn_complete` 之前 emit `assistant_step_done`

#### Scenario: Mock Provider 同样逐步通知
- **WHEN** 使用 Mock Provider 跑通多步工具循环
- **THEN** 每一步持久化的 assistant 均 emit `assistant_step_done`，行为与真实 Provider 一致

Google 协议元数据 MUST 与消息正文一起持久化，但其不透明签名和内部映射 MUST 不出现在用户可见事件及历史列表 JSON 中。

#### Scenario: 协议元数据先持久化
- **WHEN** Google 返回工具调用并准备通知 UI
- **THEN** 正文、可读摘要和协议元数据已同次写入，之后才执行工具，事件不包含内部签名

### Requirement: 多模态消息序列化

系统 SHALL 支持按 provider 序列化 user 多模态输入：Chat Completions 使用 text + image_url，Google 同样使用 text + image_url。`ChatMessage` 与 store 层 MUST 能承载文本与附件元数据；发往 Provider 前由 `messages_from_store`（或等价模块）将附件文件编码为 `data:{mime};base64,...`。

Chat Completions 的 tool / assistant / system 消息在 MVP 中 MUST 保持字符串 content（`image_read` 子调用结果以纯文本 tool 消息回填）。

#### Scenario: 重建含附件的 user 消息

- **WHEN** store 中 user 消息含 `attachments_json` 与文本
- **THEN** 发往 Kimi 的 messages 数组中该条 user content 为含 `image_url` 的数组

#### Scenario: Google 图片历史重建
- **WHEN** Google 会话 user 消息含合法附件
- **THEN** 兼容请求包含 image_url Data URL，复用其他视觉模型编码

### Requirement: Provider 输出 token 字段映射

当且仅当调用方显式指定输出 token 上限时，系统 SHALL 按 Provider 写入正确字段：DeepSeek 与智谱 GLM → `max_tokens`；Kimi 与 MiMo → `max_completion_tokens`；Google → `max_tokens`。主 Agent 循环的常规 chat 请求 MUST 省略对应输出限制字段（使用各模型的厂商默认，不假定均为 32K）。

#### Scenario: 主循环不传输出上限

- **WHEN** Agent loop 发起常规模型请求且未设置内部输出上限
- **THEN** 请求 body 不包含 `max_tokens` 也不包含 `max_completion_tokens`

#### Scenario: 压缩摘要使用正确字段

- **WHEN** 对 DeepSeek 发起压缩摘要且内部上限为 8192
- **THEN** body 含 `max_tokens: 8192` 且不含 `max_completion_tokens`

#### Scenario: 压缩摘要 Kimi 字段

- **WHEN** 对 Kimi 发起压缩摘要且内部上限为 8192
- **THEN** body 含 `max_completion_tokens: 8192`

#### Scenario: Google 与 GLM 显式预算
- **WHEN** Google 或 GLM 辅助请求显式使用输出预算1024
- **THEN** Google 发送 max_tokens=1024，GLM 发送 max_tokens=1024，不混用其他协议字段

### Requirement: 按模型动态工具列表

Agent loop 组装 `tools` 时 MUST 依据会话模型的 `supports_vision` 过滤工具；`supports_vision=false` 时不得包含 `image_read`。

#### Scenario: 工具列表随模型变化

- **WHEN** 同一会话锁定为 `mimo-v2.5-pro` 并开始新 turn（首条消息前已选模型）
- **THEN** 该会话全程工具定义不含 `image_read`

#### Scenario: DeepSeek Flash 含 image_read

- **WHEN** 会话模型为 DeepSeek Flash
- **THEN** 工具列表含 `pdf_read`、`pdf_render_pages` 与 `image_read`

### Requirement: vision 能力发送前校验

Agent loop 在持久化或发往 Provider 之前 MUST 校验 user 消息：若含 `attachments_json`（或等价附件元数据）且会话模型 `supports_vision=false`，MUST 拒绝并返回明确错误，不得仅依赖前端 toast。

#### Scenario: non-vision 会话拒绝多模态 user 消息

- **WHEN** 会话模型为 MiMo v2.5 Pro 且 `send_message` 含图片附件
- **THEN** loop 返回错误，消息不进入 store，不发起 Provider 请求

#### Scenario: DeepSeek Flash 接受图片附件

- **WHEN** 会话模型为 `deepseek-flash` 且 `send_message` 含图片附件
- **THEN** 允许发送，user content 含 text 与 image_url Data URL

### Requirement: PDF vision 工具注册

系统 SHALL 在默认工具列表中注册 `pdf_render_pages` 与 `pdf_read`（所有模型可见）。`pdf_read` 无 `mode` 参数：非 vision 会话走 PDFium 文本分支，vision 会话经硬规则与代表页 Judge 决定是否全量 vision 子调用。

`pdf_read` 全量 vision 路径分批理解时 MAY 内部调用共享 vision helper 或已注册的 `image_read` 逻辑，每批图片数 MUST NOT 超过 4。

#### Scenario: 非 vision 会话可见 pdf_read

- **WHEN** 会话模型为 MiMo v2.5 Pro
- **THEN** 工具列表含 `pdf_read` 与 `pdf_render_pages`，不含 `image_read`

#### Scenario: vision 会话全套 PDF 工具

- **WHEN** 会话模型为 MiMo v2.5 或 DeepSeek Flash
- **THEN** 工具列表含 `pdf_read`、`pdf_render_pages` 与 `image_read`

## ADDED Requirements

### Requirement: Provider 协议元数据持久化兼容

系统 SHALL 支持可选的轻量协议元数据，以保证 Google 会话工具调用及恢复可重放。旧 provider 消息无协议元数据时 SHALL 保持原请求行为；升级 MUST 不改写历史正文、推理、模型标识和工具结果。可空状态的数据库迁移 MUST 可重复执行，失败 MUST 明确报告。

#### Scenario: 旧库升级继续使用现有 provider
- **WHEN** 升级含 DeepSeek/MiMo/Kimi 历史的数据库
- **THEN** 历史可读取，协议元数据为空，后续请求不包含 Google 字段

#### Scenario: 重复初始化数据库
- **WHEN** 已升级数据库再次启动
- **THEN** 不重复添加列、不丢弃历史、协议元数据可恢复

#### Scenario: 损坏状态隔离
- **WHEN** 一条 Google 协议元数据损坏
- **THEN** 会话历史仍可显示，运行该 Google 会话明确失败，不影响其他 provider 会话

### Requirement: Gemini 人工澄清恢复

系统 SHALL 将 Google clarify 调用的签名元数据和调用对应关系持久化；用户提交答案后，恢复请求 SHALL 包含原步骤和匹配答案，不重复执行已完成工具，也不把恢复视为新 user turn。

#### Scenario: 重载后提交澄清答案
- **WHEN** Google 会话停在 clarify pending，重载后用户提交答案
- **THEN** 原始签名和调用标识可恢复，答案作为对应函数结果回传，loop 继续

### Requirement: 辅助请求遵循模型能力

标题、压缩、视觉子调用 SHALL 使用目标模型可接受的思考配置；可关闭模型关闭思考，始终思考模型使用 low。输出预算 MUST 给正文保留合理空间；截断或失败不得覆盖有效标题/摘要。退休或未知模型的后台辅助调用 SHALL 跳过，主动操作返回明确错误。

#### Scenario: Gemini 标题生成
- **WHEN** Gemini 会话生成标题
- **THEN** 请求使用 low 和适合思考模型的输出预算，不请求 disabled，只有有效标题才更新

#### Scenario: GLM 视觉子调用
- **WHEN** GLM 执行 image_read 或 PDF 视觉子调用
- **THEN** 使用 enabled/low，结果按现有纯文本工具结果返回
