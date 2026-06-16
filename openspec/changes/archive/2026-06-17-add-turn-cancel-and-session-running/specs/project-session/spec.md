## ADDED Requirements

### Requirement: 同 project 单 turn 互斥

系统 SHALL 保证同一 `project_id` 下最多一个 session 处于 TurnRegistry active（running）状态。当用户在某 session 发起 `send_message` 或 `resume_turn` 时，若同 project 已有其他 session 的 active turn，MUST 拒绝并返回可读错误（含 running session 的标题或 id）。clarify `turn_awaiting_user` 期间不算 active，不触发此互斥。

#### Scenario: 第二会话发送被拒

- **WHEN** session A 正在 running，用户在 session B 发送消息
- **THEN** `send_message` 返回错误，提示先停止 A 或等待完成，B 不写入 user message

#### Scenario: A clarify 等待时 B 可发送

- **WHEN** session A 处于 clarify pending（无 active turn），session B idle
- **THEN** B 的 `send_message` 正常启动

#### Scenario: 互斥不影响会话切换

- **WHEN** session A running，用户切换到 session B 查看历史
- **THEN** 切换成功；B 仍不可发送直至 A 结束或 stop

### Requirement: 会话 running 状态可查询（供 UI）

系统 SHOULD 通过现有 `agent-event`（`turn_*` 与 token 事件）驱动前端 running 指示；无需新增 DB 字段。侧栏展示 running 时 SHOULD 能识别 session id 与 project 内互斥错误中的 session 标题一致。

#### Scenario: running session 在侧栏可识别

- **WHEN** session A 正在 running 且用户查看侧栏
- **THEN** session A 项显示 running 指示，非 running 会话无该指示
