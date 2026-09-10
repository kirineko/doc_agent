## ADDED Requirements

### Requirement: Provider 错误事件类型契约

前端 `AgentEvent` 类型与 Rust 序列化 MUST 对齐，扩展与新增：

- `error`：在原有 `session_id`、`turn_id`、`message` 之外新增可选 `code`（`auth` | `rate_limit` | `bad_request` | `payload_too_large` | `context_length` | `server` | `network` | `timeout` | `stream_error` | `stream_incomplete`）、`retryable`、`detail`、`hint`
- `provider_retry`：`session_id`、`turn_id`、`attempt`、`max`、`kind`、`delay_ms`

`AgentStreamState` MUST 维护 `turnError`（最近一次 `error` 事件的结构化内容）与 `retryNotice`；`error` 事件 MUST NOT 再把 `message` 拼接进 `streamingContent`。任何 `content_token` / `reasoning_token` / `turn_complete` / `turn_cancelled` MUST 清空 `retryNotice`；用户发送新消息 MUST 清空 `turnError`。

#### Scenario: error 事件更新状态

- **WHEN** 收到 `error` 事件 `{code:"rate_limit", retryable:true, hint:"…"}`
- **THEN** `turnError` 为该结构，`busy` 为 false，`streamingContent` 与事件前一致

#### Scenario: 旧格式 error 兼容

- **WHEN** 收到仅含 `message` 的 `error` 事件（无 `code`）
- **THEN** `turnError.message` 为该文本，其余字段为 undefined，UI 仍可渲染

#### Scenario: retryNotice 生命周期

- **WHEN** 收到 `provider_retry` 后再收到 `content_token`
- **THEN** `retryNotice` 先被设置、后被清空

#### Scenario: 退避期间取消清除提示

- **WHEN** 收到 `provider_retry` 后用户取消，随后收到 `turn_cancelled`
- **THEN** 会话回到 idle，`busy` 为 false，`retryNotice` 被清空，重试状态行消失

### Requirement: Provider 错误卡片

会话区 MUST 在流式内容之后以独立卡片展示 `turnError`：标题为 `message`；`hint` 存在时以次级文字展示；`detail` 存在时提供默认收起的折叠区与「复制详情」按钮；卡片 MUST 使用错误语义样式且与正文 Markdown 区分。`retryNotice` 存在时 MUST 在流式区域展示一行状态文案「网络波动，正在重试（{attempt}/{max}）…」。

#### Scenario: 鉴权失败卡片

- **WHEN** `turnError` 为 `{message:"模型服务鉴权失败：Invalid API key", code:"auth", hint:"请在「密钥与服务」中检查该模型的 API Key", detail:"{\"error\":…}"}`
- **THEN** 卡片显示标题与 hint，折叠区默认收起，点击展开后可见 `detail`，点击「复制详情」将 `detail` 写入剪贴板

#### Scenario: 重试状态行

- **WHEN** `retryNotice` 为 `{attempt:1, max:2}`
- **THEN** 流式区域显示「网络波动，正在重试（1/2）…」，`busy` 保持 true

#### Scenario: 新消息清除卡片

- **WHEN** 用户在错误卡片存在时发送新消息
- **THEN** 卡片消失
