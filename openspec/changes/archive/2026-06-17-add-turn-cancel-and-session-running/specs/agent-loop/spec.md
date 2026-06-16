## ADDED Requirements

### Requirement: TurnRegistry 与 active turn 生命周期

系统 SHALL 在内存中维护 `TurnRegistry`，按 `session_id` 注册 in-flight turn，记录 `turn_id` 与 `project_id`。`run_turn` 与 `resume_turn` 进入 `continue_loop` 前 MUST register；turn 以 `turn_complete`、`turn_cancelled`、`turn_awaiting_user`（clarify 暂停）或不可恢复错误结束时 MUST unregister。`turn_awaiting_user` 时 MUST unregister（clarify 等待不算 running）。

#### Scenario: turn 开始时注册

- **WHEN** `send_message` 成功启动 `run_turn`
- **THEN** 该 `session_id` 在 TurnRegistry 中存在 active 记录直至 turn 结束事件

#### Scenario: clarify 暂停后不算 running

- **WHEN** loop emit `turn_awaiting_user`
- **THEN** TurnRegistry 中该 session 无 active 记录，允许同 project 其他 session 发起新 turn

### Requirement: cancel_turn 协作式停止

系统 SHALL 提供 `cancel_turn` command（参数 `session_id`）。当该 session 存在 active turn 时，MUST 触发 cancel 信号；loop 在步间或工具 handler **返回后** 退出，并对尚未写入 result 的 tool_calls 补写 `{ "cancelled": true }` 的 tool result 与 tool message。Cancel 完成后 MUST emit `turn_cancelled`（含 `session_id`、`turn_id`），MUST NOT emit `turn_complete`，MUST NOT 触发会话自动标题。若无 active turn，MUST 返回错误。

#### Scenario: 用户停止进行中的 turn

- **WHEN** 用户在前端点击停止且该 session 有 active turn
- **THEN** 系统在合理时间内 emit `turn_cancelled`，已 persist 的 assistant 步骤保留，未完成 tool_calls 均有 cancelled result

#### Scenario: 无 active turn 时拒绝

- **WHEN** 用户对 idle session 调用 `cancel_turn`
- **THEN** command 返回错误，不修改 DB

#### Scenario: cancel 不触发 autotitle

- **WHEN** 用户在第 1 或第 2 轮 user 消息 turn 中途 cancel
- **THEN** 会话标题不因该次 cancel 自动更新

#### Scenario: 工具执行中 cancel 等待 handler

- **WHEN** cancel 请求到达时某 tool handler（含 `skill_run`）仍在执行
- **THEN** 系统等待该 handler 返回或超时后再补 cancelled results 并 emit `turn_cancelled`

### Requirement: turn_cancelled 事件

系统 SHALL 新增 AgentEvent `turn_cancelled`，字段含 `session_id`、`turn_id`。语义：用户主动停止，本轮 user 请求未完成，不可 `resume_turn` 续跑（与 `turn_awaiting_user` 区分）。

#### Scenario: 前端结束 running 态

- **WHEN** 前端收到 `turn_cancelled`
- **THEN** 该 session 的运行态（busy/running）MUST 置为 idle，streaming 缓冲清空

### Requirement: SSE 与压缩摘要可被取消

主 Agent loop 的流式 LLM 请求与上下文压缩的摘要 LLM 请求 MUST 监听 cancel 信号；收到 cancel 后 MUST 停止读取 SSE 并进入 cancel 收尾路径，不得继续追加 token 事件。

#### Scenario: 流式生成中被 stop

- **WHEN** 模型正在流式返回 content 且用户 stop
- **THEN** SSE 读取终止，不再 emit 新的 `content_token` / `reasoning_token`，随后 emit `turn_cancelled`

### Requirement: running 期间拒绝同 session 新消息

当 TurnRegistry 中某 `session_id` 存在 active turn 时，`send_message` MUST 返回错误（提示等待完成或先停止），MUST NOT 写入新 user message。

#### Scenario: 同 session 重复发送被拒

- **WHEN** session A 的 turn 仍在 running 且用户对 A 再次 `send_message`
- **THEN** command 返回错误，不追加 user message

### Requirement: resume_turn 受 project 互斥约束

`resume_turn` 启动前 MUST 与 `send_message` 相同地检查 TurnRegistry：同 session 不得已有 active turn；同 project 不得有其他 session 的 active turn。

#### Scenario: 其他 session running 时 clarify resume 被拒

- **WHEN** 项目内 session B 正在 running，session A 存在 clarify pending 且用户 submit 触发 `resume_turn`
- **THEN** resume 被拒绝并返回含 B 会话标识的错误

## MODIFIED Requirements

### Requirement: turn_awaiting_user 事件

系统 SHALL 新增 AgentEvent `turn_awaiting_user`，字段含 `session_id`、`turn_id`。语义：本轮用户请求尚未完成，等待 clarify 交互。收到该事件时 TurnRegistry MUST 已 unregister 该 session。

#### Scenario: 前端 busy 状态

- **WHEN** 收到 `turn_awaiting_user`
- **THEN** 前端 MUST 将该 session 的运行态置为 idle（非 running），以便用户操作澄清卡片（与 `turn_complete` 区分）
