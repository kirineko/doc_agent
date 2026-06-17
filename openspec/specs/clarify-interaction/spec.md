# clarify-interaction Specification

## Purpose
TBD - created by archiving change add-clarify-interaction. Update Purpose after archive.
## Requirements
### Requirement: clarify_ask 工具

系统 SHALL 提供 `clarify_ask` 工具，供 Agent 在需求澄清流程中发起结构化问题。工具参数 MUST 符合 `ClarifyQuestion` schema（`id`、`kind`、`prompt` 必填；`kind` 为 `single` | `multi` | `text` | `confirm_brief`）。`single`/`multi` MUST 携带 2–12 个 `options`；工具 JSON Schema 的 `options` 数组 MUST 声明 `minItems: 2` 与 `maxItems: 12`。clarify skill 推荐 Agent 使用 2–8 个选项并配合 `allow_custom` 承接「其他」。`confirm_brief` MUST 携带 `brief` 字段；`allow_custom` 默认为 true。

#### Scenario: Agent 发起单选澄清题

- **WHEN** Agent 调用 `clarify_ask` 且 `kind=single`，含 2–12 个 `options`
- **THEN** 系统校验通过并进入 clarify 暂停流程（见 agent-loop spec），向前端 emit `clarify_question`

#### Scenario: 参数非法被拒绝

- **WHEN** Agent 调用 `clarify_ask` 但缺少 `id`、`kind`、`prompt`，或 `single` 无 `options`，或 `options` 少于 2 项或多于 12 项
- **THEN** 返回结构化错误 tool result，loop 正常继续（不暂停、不创建 pending）

#### Scenario: confirm_brief 题型

- **WHEN** Agent 调用 `clarify_ask` 且 `kind=confirm_brief`，`brief` 含创作简报字段
- **THEN** 前端展示简报预览与确认/修改交互

---

### Requirement: clarify_pending 持久化

系统 SHALL 在 SQLite 中维护 `clarify_pending` 表，每 session 最多一条 pending 记录，字段含 `session_id`（PK）、`turn_id`、`tool_call_id`、`question_json`、`created_at`。对应 `tool_calls` 记录 status MUST 为 `awaiting_user` 且 `result_json` 为空。该表仅供后端 resume 与单 pending 约束使用。

#### Scenario: 刷新后恢复 pending 问题

- **WHEN** 用户刷新应用且 session 存在未答的 clarify pending
- **THEN** 前端通过 `list_messages` bundle 中 `status=awaiting_user` 的 clarify_ask 工具记录还原活跃澄清卡片，无需新增查询 IPC

#### Scenario: 同一轮多个 clarify_ask 仅保留一个

- **WHEN** 模型在同一轮返回多个 `clarify_ask` 调用
- **THEN** 仅第一个进入 pending，其余 MUST 立即写入结构化错误 result（提示一次只允许一个澄清问题），不阻塞暂停流程

---

### Requirement: 澄清答案以 tool result 为唯一载体

用户提交澄清答案后，系统 MUST NOT 写入 `role='user'` 消息；答案 MUST 仅持久化为对应 tool_call 的 `result_json` 与 tool role message。消息序列 MUST 保持 `assistant(tool_calls) → tool(result)` 紧邻，确保 OpenAI 兼容协议重建合法。

#### Scenario: 提交后无 user 消息

- **WHEN** 用户提交 clarify 答案
- **THEN** `messages` 表新增 tool role 消息（`tool_call_id` 对应），且 user 消息计数不变

#### Scenario: 统计逻辑不受澄清影响

- **WHEN** 用户在一次会话中回答了 3 道澄清题
- **THEN** autotitle 资格判定、模型锁定（`session_has_chat_messages`）、followup 推荐与前端消息计数所依据的 user 消息数 MUST 不因澄清回答而增加

---

### Requirement: submit_clarify_answer IPC

系统 SHALL 提供 `submit_clarify_answer` command，接收 `session_id`、`question_id`、`selected`（option id 数组，可空）、`custom`（自定义文本，可空）。提交后 MUST 按序：

1. 事务内删除 `clarify_pending`（删除 0 行时返回错误，防双重提交）
2. 按 `question_json` 校验答案（kind 匹配、option id 合法、multi 满足 min/max、无选项时 custom 非空）
3. **后端**组装 `display_text`（选项 label 与 custom 拼接）与 tool result JSON
4. `finish_tool_call`（status=`done`）并写入 tool role message
5. emit `ToolResult` 事件
6. 调用 `resume_turn(session_id, turn_id)` 继续 Agent loop

#### Scenario: 用户选择预设选项

- **WHEN** 用户在前端选择单选选项并提交
- **THEN** tool result 含 `selected` 与后端组装的 `display_text`，Agent loop 在同一 `turn_id` 下继续

#### Scenario: 用户提交自定义文本

- **WHEN** 用户选择「其他」并填写自定义内容后提交
- **THEN** tool result 的 `custom` 字段含用户文本，`display_text` 由后端依据 custom 组装

#### Scenario: 双重提交被拒绝

- **WHEN** 同一 pending 被并发提交两次
- **THEN** 仅第一次成功，第二次返回「澄清已处理或不存在」错误，不重复写消息

#### Scenario: question_id 不匹配

- **WHEN** 提交的 `question_id` 与 pending 的 `question_json.id` 不一致
- **THEN** 返回错误，不修改 pending 与消息历史

#### Scenario: 答案不满足约束

- **WHEN** `kind=multi` 且选择数低于 `min_selections`，或无选项且 `custom` 为空
- **THEN** 返回校验错误，pending 保留

---

### Requirement: cancel_clarify IPC

系统 SHALL 提供 `cancel_clarify` command：流程与 submit 一致，tool result 写 `{ "cancelled": true }`，resume 后由 Agent 决定结束澄清或采用默认值继续。

#### Scenario: 用户取消澄清

- **WHEN** 用户点击取消澄清
- **THEN** pending 清除，Agent 收到 cancelled tool result 并可结束或改用默认参数

