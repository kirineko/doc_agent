# agent-loop 能力增量

## ADDED Requirements

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
