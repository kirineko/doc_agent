## ADDED Requirements

### Requirement: 上下文占用比例展示

系统 SHALL 在会话区标题栏（中间区「会话」标题行右侧）以**最小化**形式展示当前上下文占用比例：仅图标 + 比例百分比值（如 `42%`），MUST NOT 展示 token 绝对值等冗余信息。比例数据来源为 `context_usage` 事件的 `ratio`；切换会话时 MUST 重置。无可用数据（尚未发生任何 LLM 调用）时 MAY 隐藏该指示器。指示器颜色 MAY 随接近上限而变化（如转橙/红）。

#### Scenario: 展示当前占用比例

- **WHEN** 当前会话已发生至少一次 LLM 响应且收到 `context_usage`
- **THEN** 会话区标题栏右侧显示图标 + 百分比（如 `42%`），不显示绝对 token 数

#### Scenario: 切换会话重置比例

- **WHEN** 用户切换到另一个会话
- **THEN** 比例指示器重置，按新会话的 `context_usage` 重新展示（或在无数据时隐藏）

#### Scenario: 接近上限的视觉提示

- **WHEN** 上下文占用比例接近模型上限（高 ratio）
- **THEN** 指示器以更醒目的颜色提示（如橙/红），帮助用户感知即将压缩

### Requirement: 自动压缩一次性提示

系统 SHALL 在收到 `context_compacted` 事件时展示一次性、非阻断的轻提示（toast 或会话区一行系统提示），文案说明已自动压缩较早历史以节省上下文。该提示 MUST NOT 阻断输入框或弹出模态，且 MUST NOT 常驻。

#### Scenario: 压缩后给出轻提示

- **WHEN** 前端收到 `context_compacted` 事件
- **THEN** 展示一次性轻提示（如「已自动压缩较早的对话历史」），输入框不被阻断

### Requirement: 上下文事件类型契约

前端 `AgentEvent` 类型与 Rust 序列化 MUST 对齐，新增：

- `context_usage`：`session_id`、`used_tokens`、`max_tokens`、`ratio`
- `context_compacted`：`session_id`、`before_tokens`、`after_tokens`

`AgentStreamState` MUST 维护当前会话的上下文比例（如 `contextRatio`），由 `context_usage` 更新、会话切换时重置。

#### Scenario: 事件驱动更新比例状态

- **WHEN** 收到归属当前 activeSession 的 `context_usage`
- **THEN** `AgentStreamState.contextRatio` 更新为事件 `ratio`，指示器随之刷新

#### Scenario: 非活跃会话事件被忽略

- **WHEN** 收到 `session_id` 非当前 activeSession 的 `context_usage`
- **THEN** 当前展示的比例不受影响
