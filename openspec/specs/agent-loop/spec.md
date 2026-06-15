# agent-loop Specification

## Purpose
TBD - created by archiving change bootstrap-doc-agent-mvp. Update Purpose after archive.
## Requirements
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

---

### Requirement: 思考与正文的流式分离输出
系统 SHALL 在流式响应中分别累积 `reasoning_content`（思考）与 `content`（正文），并以独立事件推送给前端。

#### Scenario: 思考与正文分区展示
- **WHEN** 模型处于思考模式并流式返回 `reasoning_content` 与 `content`
- **THEN** 系统先以「思考」事件推送思考增量、再以「正文」事件推送回答增量，二者不混淆

### Requirement: 工具调用轮的 reasoning_content 回填
系统 SHALL 在持久化 assistant 消息时一并存储其 `reasoning_content`；在构造后续请求时，对「包含工具调用的 assistant 消息」必须回传其 `reasoning_content`。

#### Scenario: 含工具调用的轮次正确回填
- **WHEN** 某轮 assistant 消息包含 `tool_calls` 与 `reasoning_content`，且需要继续请求模型
- **THEN** 系统在后续请求中携带该 `reasoning_content`，使模型不返回 400 错误

### Requirement: 工具在目录沙箱内执行
系统 SHALL 通过统一的工具分发器执行所有工具调用；**文件系统类工具**对文件系统的访问被限定在当前项目根目录内。已启用的 Web 搜索类工具（`web_search` / `web_extract`）作为例外 MAY 访问外部 HTTP 服务：其 URL 与查询参数 MUST NOT 被校验为项目相对路径，且成功执行时 `tool_result.changed_paths` MUST 为空。

#### Scenario: 越界路径被拒绝
- **WHEN** 模型请求的工具参数包含指向项目根目录之外的路径
- **THEN** 系统拒绝执行并返回错误结果，循环继续而不中断

#### Scenario: web_search 无 changed_paths
- **WHEN** `web_search` 成功返回
- **THEN** 对应 `tool_result` 的 `changed_paths` 为空或省略

### Requirement: 无密钥可运行的 Mock Provider

系统 SHALL 提供一个 Mock Provider，在未配置真实 API Key 时仍可驱动循环、工具调用与前端事件，用于开发与测试。Mock Provider MUST 支持 clarify 场景：用户文本命中约定关键词（如「澄清」）时返回一个 `clarify_ask` tool call，以便无 Key 验证暂停/恢复全链路。

#### Scenario: Mock 模式跑通端到端

- **WHEN** 未配置任何模型 API Key 且选择 Mock Provider
- **THEN** 系统可产生模拟的思考 / 正文 / 工具调用事件，完整跑通 UI 与工具执行

#### Scenario: Mock clarify 场景

- **WHEN** Mock Provider 收到含「澄清」关键词的用户消息
- **THEN** 返回一个合法 `clarify_ask` tool call，触发暂停流程；收到对应 tool result 后返回最终回答

### Requirement: Assistant 逐步持久化事件
系统 SHALL 在 Agent 循环中每次成功 `persist_assistant` 写入 assistant 消息后，向客户端 emit `assistant_step_done` 事件；payload MUST 包含 `session_id`、`turn_id` 与刚持久化的完整 assistant 消息（含 `id`、`content`、`reasoning_content` 等字段，与 `list_messages` 单条结构一致）。该事件 MUST 在工具执行之前发出（含工具调用轮与最终回答轮）。

#### Scenario: 含工具调用的轮次逐步通知
- **WHEN** 模型返回带 `tool_calls` 的 assistant 回答并已持久化
- **THEN** 系统在执行任何工具之前 emit `assistant_step_done`，且消息内容与 DB 一致

#### Scenario: 最终回答轮逐步通知
- **WHEN** 模型返回不含工具调用的最终 assistant 回答并已持久化
- **THEN** 系统在 emit `turn_complete` 之前 emit `assistant_step_done`

#### Scenario: Mock Provider 同样逐步通知
- **WHEN** 使用 Mock Provider 跑通多步工具循环
- **THEN** 每一步持久化的 assistant 均 emit `assistant_step_done`，行为与真实 Provider 一致

### Requirement: 多步 loop 间流式状态边界
系统 SHALL 将每次 LLM 流式请求（`reasoning_token` / `content_token`）的累积范围限定为「当前步」；当前步 assistant 持久化并完成 `assistant_step_done` 后，后续步的 token 事件 MUST NOT 与前一步已推送的正文/思考混在同一逻辑步内（由前端在收到 `assistant_step_done` 时清空 streaming 缓冲实现）。

#### Scenario: 第二步 LLM 不合并第一步流式内容
- **WHEN** 第一轮 LLM 已持久化并 emit `assistant_step_done`，随后开始第二轮 LLM 流式输出
- **THEN** 前端 streaming 缓冲为空，第二轮 token 仅构成新的流式预览

### Requirement: 工具结果携带变更路径
系统 SHALL 在 Agent 循环执行文件变更类工具成功后，于 `tool_result` 事件中携带 `changed_paths` 字段（相对项目根的路径字符串数组，POSIX 分隔符）；前端据此增量更新文件索引。失败或只读工具 MUST NOT 携带有效变更路径。

#### Scenario: 写文件工具返回路径
- **WHEN** `fs_write` 成功写入 `notes/todo.md`
- **THEN** 对应 `tool_result` 的 `changed_paths` 包含 `notes/todo.md`

#### Scenario: 解压工具返回目录而非内部 XML
- **WHEN** `ooxml_unpack` 成功解压到 `unpacked/`
- **THEN** `changed_paths` 包含 `unpacked/`（或等效目录路径），MUST NOT 枚举 `unpacked/word/*.xml` 等内部部件

#### Scenario: skill_run 追踪 doc_write
- **WHEN** `skill_run` 内脚本通过 `doc_write` / `__doc_write` 写入 `out.xlsx`
- **THEN** `tool_result.changed_paths` 包含 `out.xlsx`

#### Scenario: 工具失败无变更路径
- **WHEN** 某写文件工具执行失败
- **THEN** `tool_result` 的 `changed_paths` 为空或省略，且 `ok` 为 false

### Requirement: 网络工具条件注册
系统 SHALL 支持按运行时条件过滤注册到 LLM 的工具子集；当 Tavily Key 未配置时，`web_search` 与 `web_extract` MUST 从 tool definitions 中排除。

#### Scenario: 过滤在每回合生效
- **WHEN** Agent loop 开始处理用户消息并构造 LLM 请求
- **THEN** 系统根据当前 `has_api_key("tavily")` 决定是否包含 web 工具，而非启动时静态固定

### Requirement: 异步工具执行
系统 SHALL 支持异步工具 handler；`loop_runner` 在沙箱/Secrets 上下文内 await 工具执行结果，网络 I/O MUST NOT 阻塞 tokio worker 以外的同步阻塞调用（禁止在 async 上下文中对 Tavily 使用 `block_on`）。

#### Scenario: web 工具异步完成
- **WHEN** 模型调用 `web_search` 且 Tavily API 需要网络等待
- **THEN** loop 异步等待 handler 完成后再 persist tool 结果并继续下一轮 LLM

### Requirement: Web 能力 system prompt 注入
系统 SHALL 在 Tavily Key 已配置时，于 system prompt 追加简短说明：可用 `web_search` 获取外部信息、可用 `web_extract` 读取已知 URL 正文。

#### Scenario: 有 Key 时 prompt 含 Web 说明
- **WHEN** 用户已配置 Tavily Key 且 Agent 构造 system 消息
- **THEN** system 内容包含 Web 搜索工具的使用提示

#### Scenario: 无 Key 时不注入
- **WHEN** 用户未配置 Tavily Key
- **THEN** system 消息不包含 Web 搜索相关说明

### Requirement: clarify 人机协作断点

当 Agent loop 某轮 tool_calls 中包含 `clarify_ask` 且参数校验通过时，系统 SHALL：

1. 持久化 assistant 消息与全部 tool_calls（与现有行为一致），emit `assistant_step_done`
2. **先按序执行所有非 clarify 工具**，正常写入 result 与 tool message
3. 对第一个 `clarify_ask`：更新 tool_call status 为 `awaiting_user`、写入 `clarify_pending`、emit `ToolCall` 事件（status=`awaiting_user`）、emit `clarify_question` 事件（payload 含完整 ClarifyQuestion + `tool_call_id`）
4. 对其余多出的 `clarify_ask`：立即写入结构化错误 result
5. emit `turn_awaiting_user`（含 session_id、turn_id）
6. `run_turn` 正常 return；MUST NOT emit `turn_complete`，MUST NOT 执行下一轮 LLM，MUST NOT 执行 turn 结束清理（`.skill-run/` 清理留待 turn 真正结束）

#### Scenario: clarify 后 loop 暂停

- **WHEN** 模型返回 `clarify_ask` 且校验通过
- **THEN** 前端收到 `turn_awaiting_user` 而非 `turn_complete`，工具链卡片显示等待状态

#### Scenario: clarify 与普通工具混合

- **WHEN** 模型同轮返回 `[fs_read, clarify_ask]`
- **THEN** `fs_read` 先正常执行并写入 result，随后才进入 clarify 暂停；暂停时刻该 assistant 轮仅 clarify 调用缺 result

#### Scenario: 参数非法不暂停

- **WHEN** `clarify_ask` 参数校验失败
- **THEN** 写入错误 result 后 loop 继续（视同普通工具失败），不创建 pending

#### Scenario: 非 clarify 工具不受影响

- **WHEN** 模型返回 `fs_read` 等普通工具
- **THEN** 行为与变更前一致，同步 execute 并继续 loop

---

### Requirement: resume_turn 恢复循环

系统 SHALL 提供 `resume_turn(session_id, turn_id)`：从 DB 全量重建 `working_messages`（经 `messages_from_store` + system prompt 注入，MUST NOT 追加新 user message），在同一 `turn_id` 下继续工具循环，直至无 tool_calls（emit `turn_complete`）或再次 clarify 暂停。每次 resume 步数预算重置为 `MAX_TOOL_STEPS`。最终回答轮的 autotitle 判定 MUST 以 history 中最后一条 `role='user'` 消息为 user_text 依据。

#### Scenario: 提交答案后继续澄清

- **WHEN** 用户提交 clarify 答案且 Agent 再次调用 `clarify_ask`
- **THEN** 再次进入暂停，直至澄清完成

#### Scenario: 澄清完成后进入生成

- **WHEN** 用户确认创作简报且 Agent 不再调用 `clarify_ask`
- **THEN** loop 正常执行 skill_read / skill_run 等，最终 emit `turn_complete`

#### Scenario: resume 重建含 reasoning_content

- **WHEN** resume 重建 working_messages 且历史含带 tool_calls 的 assistant 消息
- **THEN** 该消息的 `reasoning_content` 正确回填（沿用现有「工具调用轮的 reasoning_content 回填」要求）

---

### Requirement: turn_awaiting_user 事件

系统 SHALL 新增 AgentEvent `turn_awaiting_user`，字段含 `session_id`、`turn_id`。语义：本轮用户请求尚未完成，等待 clarify 交互。

#### Scenario: 前端 busy 状态

- **WHEN** 收到 `turn_awaiting_user`
- **THEN** 前端 MUST 将 `busy` 置为 false，以便用户操作澄清卡片（与 `turn_complete` 区分）

---

### Requirement: pending 期间拒绝新消息

当 session 存在 clarify pending 时，`send_message` command MUST 返回错误（提示先完成澄清），后端入口强制校验，不依赖前端拦截。

#### Scenario: pending 时发送被后端拒绝

- **WHEN** session 存在 clarify pending 且 `send_message` 被调用
- **THEN** command 返回错误，不写入 user message、不启动新 turn

### Requirement: 循环内自动压缩触发

系统 SHALL 在 Agent 工具循环的**每一步开头**（构造 LLM 请求之前）检查上下文是否需要压缩：以 `token_count + pending_estimate` 与当前模型 `max_context_size` 调用压缩触发判定，命中则先执行压缩并以未归档消息重建 `working_messages`，再发起本步请求。触发点 MUST 覆盖单个 turn 内连续工具结果累加的场景，而非仅在 turn 开始时检查一次。

#### Scenario: turn 内大工具输出触发压缩

- **WHEN** 单个 turn 内连续多次工具调用使累计上下文接近模型上限
- **THEN** 在下一步构造请求前触发压缩，压缩后再发起请求，请求上下文不超过上限

#### Scenario: 未超阈值不压缩

- **WHEN** 当前 `token_count + pending_estimate` 低于触发阈值
- **THEN** 不执行压缩，直接发起本步请求

### Requirement: 工作上下文基于未归档消息重建

系统 SHALL 使 `build_working_messages` 仅纳入未归档（archived = 0）的消息构造工作上下文，从而让压缩产生的摘要消息与保留消息自然成为后续轮次的上下文基础。

#### Scenario: 压缩后续 turn 复用摘要

- **WHEN** 某 turn 已压缩历史并写入摘要，用户在后续 turn 继续对话
- **THEN** 新 turn 重建的工作上下文包含该摘要而非已归档的原始旧消息

### Requirement: 提高单 turn 工具步数上限

系统 SHALL 将单个 turn 的最大工具调用步数上限从 32 提高（目标 64），以支撑文档生成类多步流程（澄清 → 多次 skill_read/skill_run → 校验）。该上限仍作为防失控循环的保护，达到上限时行为与原「达到最大轮次保护」一致。

#### Scenario: 多步文档流程不易触顶

- **WHEN** 一个文档生成 turn 需要超过 32 步工具调用但少于新上限
- **THEN** loop 正常完成而不再因步数上限提前中断

#### Scenario: 达到新上限仍有保护

- **WHEN** 工具调用步数达到提高后的上限
- **THEN** 系统终止循环并返回「已达最大步数」提示

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

### Requirement: vision 能力发送前校验

Agent loop 在持久化或发往 Provider 之前 MUST 校验 user 消息：若含 `attachments_json`（或等价附件元数据）且会话模型 `supports_vision=false`，MUST 拒绝并返回明确错误，不得仅依赖前端 toast。

#### Scenario: non-vision 会话拒绝多模态 user 消息

- **WHEN** 会话模型为 DeepSeek V4 Flash 且 `send_message` 含图片附件
- **THEN** loop 返回错误，消息不进入 store，不发起 Provider 请求

### Requirement: PDF vision 工具注册

系统 SHALL 在默认工具列表中注册 `pdf_render_pages` 与 `pdf_read`（所有模型可见）。`pdf_read` 无 `mode` 参数：非 vision 会话走 PDFium 文本分支，vision 会话经硬规则与代表页 Judge 决定是否全量 vision 子调用。

`pdf_read` 全量 vision 路径分批理解时 MAY 内部调用共享 vision helper 或已注册的 `image_read` 逻辑，每批图片数 MUST NOT 超过 4。

#### Scenario: 非 vision 会话可见 pdf_read

- **WHEN** 会话模型为 DeepSeek V4 Flash
- **THEN** 工具列表含 `pdf_read` 与 `pdf_render_pages`，不含 `image_read`

#### Scenario: vision 会话全套 PDF 工具

- **WHEN** 会话模型为 MiMo v2.5
- **THEN** 工具列表含 `pdf_read`、`pdf_render_pages` 与 `image_read`

