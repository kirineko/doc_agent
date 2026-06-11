# agent-loop Specification

## Purpose
TBD - created by archiving change bootstrap-doc-agent-mvp. Update Purpose after archive.
## Requirements
### Requirement: Agent 多轮工具调用循环
系统 SHALL 实现一个 Agent 执行循环：构造会话上下文后向模型发起流式请求，若返回包含工具调用则在沙箱内执行并将结果回填，再次请求模型，如此重复，直到模型返回不含工具调用的最终回答为止。

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
系统 SHALL 提供一个 Mock Provider，在未配置真实 API Key 时仍可驱动循环、工具调用与前端事件，用于开发与测试。

#### Scenario: Mock 模式跑通端到端
- **WHEN** 未配置任何模型 API Key 且选择 Mock Provider
- **THEN** 系统可产生模拟的思考 / 正文 / 工具调用事件，完整跑通 UI 与工具执行

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

