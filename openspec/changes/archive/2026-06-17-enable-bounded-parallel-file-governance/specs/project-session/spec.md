## MODIFIED Requirements

### Requirement: 同 project 单 turn 互斥

系统 MUST NOT 再以 project 为单位强制单 turn 互斥。系统 SHALL 允许同一 `project_id` 下多个 session 同时处于 running，前提是：

1. 全局 running turn 数未超过 3
2. 每个 session 仍最多一个 active/reserved turn
3. 各 running turn 的文件锁不冲突

若同 project 内其他 session running 但文件资源不冲突，新 `send_message` 或 `resume_turn` MUST 正常启动。若文件资源冲突，后发工具调用 MUST 失败并返回「当前 xxx 已被会话占用，请稍后重试」类错误。clarify `turn_awaiting_user` 期间不算 running，不占全局并行名额。

#### Scenario: 同 project 不同文件并行

- **WHEN** project P 中 session A 正在生成 `a.docx`
- **AND** session B 发送任务生成 `b.docx`
- **THEN** B 的 turn 可以启动并执行

#### Scenario: 同 project 同文件写冲突

- **WHEN** project P 中 session A 正在写 `report.docx`
- **AND** session B 也尝试写 `report.docx`
- **THEN** B 的对应工具调用失败，错误说明该文件已被 A 占用

#### Scenario: 同 project 三个会话并行

- **WHEN** project P 中 session A、B、C 分别操作不同文件
- **THEN** 三个 session 可同时 running

#### Scenario: 第四个会话被全局上限拒绝

- **WHEN** 应用内任意 project 合计已有 3 个 running turns
- **AND** project P 中 session D 发送消息
- **THEN** D 被全局上限拒绝，且不写入 user message

### Requirement: 会话 running 状态可查询（供 UI）

系统 SHALL 通过现有 `agent-event`（`turn_*` 与 token 事件）驱动前端 running 指示；无需新增 DB 字段。侧栏展示 running 时 MUST 能同时识别多个 running sessions，并能展示全局 running 数接近或达到 3 的状态。

#### Scenario: 多个 running session 在侧栏可识别

- **WHEN** session A、B、C 同时 running
- **THEN** 侧栏对应会话项均显示 running 指示

- **WHEN** 任一 session 结束
- **THEN** 该 session 指示变为 idle，其他 running session 不受影响
