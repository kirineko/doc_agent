## ADDED Requirements

### Requirement: 澄清问题交互卡片

系统 SHALL 在会话区展示 `ClarifyQuestionCard`。活跃卡片（pending 未答）数据源为 `clarify_question` 事件或 session 加载时 bundle 中 `status=awaiting_user` 的 clarify_ask 工具记录；卡片 MUST 渲染在消息列表底部（输入框上方）。卡片 MUST 支持四种 `kind`：

- `single`：选项按钮/卡片，选中高亮；`allow_custom` 时展示「其他」+ 输入框
- `multi`：多选 chip，校验 `min_selections`/`max_selections`；支持自定义追加
- `text`：textarea；可选快捷 chip 辅助填入
- `confirm_brief`：创作简报字段预览 +「确认继续」/「需要修改」；修改时展示 textarea

提交时前端仅发送 `selected` 与 `custom`（`display_text` 由后端组装）。

#### Scenario: 单选含自定义

- **WHEN** pending 问题 `kind=single` 且 `allow_custom=true`
- **THEN** 用户可选择预设选项或填写自定义文本后提交

#### Scenario: 创作简报确认

- **WHEN** pending 问题 `kind=confirm_brief`
- **THEN** 用户可确认（`selected=["confirm"]`）或提交修改意见（`custom`），随后 loop 恢复

#### Scenario: multi 校验

- **WHEN** `kind=multi` 且用户选择数不满足 `min_selections`
- **THEN** 提交按钮禁用或提交时给出前端校验提示，不发起 IPC

#### Scenario: 刷新后恢复活跃卡片

- **WHEN** 用户刷新应用且 bundle 中存在 `status=awaiting_user` 的 clarify_ask 记录
- **THEN** 活跃卡片按 `args_json` 还原渲染，可正常提交

---

### Requirement: 已答澄清卡片展示

已答澄清题 SHALL 以只读卡片形式嵌入消息流（数据源：`list_messages` bundle 中 `status=done` 的 clarify_ask `ToolCallRecord`，`args_json`=问题、`result_json`=答案），用户可回看自己的选择。系统 MUST NOT 为澄清答案生成 user 消息气泡。

#### Scenario: 提交后卡片转为已答态

- **WHEN** 用户成功提交 clarify 答案
- **THEN** 活跃卡片立即转为只读已答态（显示所选项/自定义内容），`busy` 转 true 等待后续流式输出

#### Scenario: 历史会话回看澄清记录

- **WHEN** 用户重新打开包含已完成澄清的会话
- **THEN** 消息流中按位置展示各已答澄清卡片，内容与当时提交一致

---

### Requirement: 澄清进行中输入约束

当 session 存在 clarify pending 时，系统 SHALL suppress 推荐问（`SuggestionCards`），前端 MUST 阻断普通消息发送并提示先完成上方澄清（后端同步强制校验，见 agent-loop spec）。输入框 placeholder MAY 提示「请先回答上方澄清问题」。

#### Scenario: pending 时不可直接发送

- **WHEN** 存在 active clarify pending 且用户尝试发送普通消息
- **THEN** 发送被阻断并展示一次性提示，输入内容保留

#### Scenario: 澄清期间不展示 followup 推荐

- **WHEN** 收到 `turn_awaiting_user`
- **THEN** 不展示 followup/starter 推荐问胶囊

---

### Requirement: clarify 事件与类型契约

前端 `AgentEvent` 与 Rust 序列化 MUST 对齐，新增：

- `clarify_question`：`session_id`、`turn_id`、`tool_call_id`、`question`（ClarifyQuestion）
- `turn_awaiting_user`：`session_id`、`turn_id`

`ToolCall` 事件 status 取值扩展 `awaiting_user`，工具链面板 MUST 为该状态展示「等待回答」而非持续转圈。`submit_clarify_answer` 请求类型（`session_id`、`question_id`、`selected`、`custom`）MUST 在 `types.ts` 定义并与 IPC 一致。

#### Scenario: 事件驱动展示卡片

- **WHEN** 收到 `clarify_question`
- **THEN** 会话区展示对应 ClarifyQuestionCard，且收到随后的 `turn_awaiting_user` 后 `busy` 为 false

#### Scenario: 工具链面板等待态

- **WHEN** clarify_ask 进入 `awaiting_user`
- **THEN** 右侧工具链卡片显示等待回答状态；用户提交后随 `ToolResult` 转为完成
