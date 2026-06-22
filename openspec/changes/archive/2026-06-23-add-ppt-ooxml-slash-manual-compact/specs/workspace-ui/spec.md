## MODIFIED Requirements

### Requirement: 自动压缩一次性提示

系统 SHALL 在收到 `context_compacted` 事件时展示一次性、非阻断的轻提示（toast 或会话区一行系统提示）。文案 MUST 根据 `trigger` 区分：

- `trigger: "auto"`：说明已**自动**压缩较早历史以节省上下文
- `trigger: "manual"`：说明已**手动**压缩对话历史

该提示 MUST NOT 阻断输入框或弹出模态，且 MUST NOT 常驻。

当 `compact_session` 返回 `compacted: false`（无可压缩段）时，前端 SHALL 展示一次性轻提示说明当前上下文较短、无需压缩；MUST NOT 展示「已压缩」类文案。

在调用摘要 LLM 之前，系统 SHALL 发出 `compaction_started` 事件；前端 MUST 展示「正在压缩上下文，请稍候…」类进行中轻提示，直至收到 `context_compacted`、错误或取消事件。

#### Scenario: 自动压缩进行中提示

- **WHEN** 循环内自动压缩开始调用摘要 LLM 且前端收到 `compaction_started`（`trigger: auto`）
- **THEN** 展示「正在压缩上下文，请稍候…」进行中轻提示
- **AND** 收到 `context_compacted` 后替换为「已自动压缩…」完成提示

#### Scenario: 自动压缩后轻提示

- **WHEN** 前端收到 `context_compacted` 且 `trigger` 为 `auto`
- **THEN** 展示「已自动压缩…」类一次性轻提示，输入框不被阻断

#### Scenario: 手动压缩后轻提示

- **WHEN** 用户通过 `/compact` 成功触发压缩且收到 `context_compacted`（`trigger: manual`）
- **THEN** 展示「已手动压缩…」类一次性轻提示，输入框不被阻断

#### Scenario: 手动压缩无需压缩时提示

- **WHEN** `compact_session` 返回 `compacted: false`
- **THEN** 展示「当前上下文较短，无需压缩」类一次性轻提示
- **AND** 不展示「已压缩」文案

### Requirement: 上下文事件类型契约

前端 `AgentEvent` 类型与 Rust 序列化 MUST 对齐，包含：

- `context_usage`：`session_id`、`used_tokens`、`max_tokens`、`ratio`
- `context_compacted`：`session_id`、`before_tokens`、`after_tokens`、`trigger`（`"auto"` | `"manual"`）
- `compaction_started`：`session_id`、`trigger`（`"auto"` | `"manual"`）

`AgentStreamState` MUST 维护当前会话的上下文比例（如 `contextRatio`），由 `context_usage` 更新、会话切换时重置。

#### Scenario: context_compacted 含 trigger

- **WHEN** 前端解析 `context_compacted` 事件 payload
- **THEN** `trigger` 字段为 `auto` 或 `manual`

#### Scenario: 事件驱动更新比例状态

- **WHEN** 收到 `context_usage` 且 `session_id` 为当前会话
- **THEN** 更新 `contextRatio` 用于标题栏百分比展示

## ADDED Requirements

### Requirement: Compact slash command UI blocking

The workspace chat UI SHALL block `/compact` execution under the same conditions as `/init` for clarify pending, and SHALL additionally block when the active session turn is `running` or `stopping`.

#### Scenario: Compact blocked while clarify pending

- **WHEN** the active session has a pending clarify question
- **THEN** picking `compact` from the slash menu SHALL show the same user-visible error as `/init`

#### Scenario: Compact blocked while turn running

- **WHEN** the session turn is running or stopping
- **THEN** submitting `/compact` SHALL be prevented with user-visible error
