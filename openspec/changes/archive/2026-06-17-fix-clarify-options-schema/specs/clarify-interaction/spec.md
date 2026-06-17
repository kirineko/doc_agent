## MODIFIED Requirements

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
