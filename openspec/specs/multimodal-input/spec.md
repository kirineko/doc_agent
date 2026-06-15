# multimodal-input Specification

## Purpose
TBD - created by archiving change add-multimodal-support. Update Purpose after archive.
## Requirements
### Requirement: 用户图片粘贴与附件

系统 SHALL 支持用户在聊天输入框通过剪贴板粘贴图片（`image/png`、`image/jpeg`、`image/webp`、`image/gif`）。粘贴成功后图片 MUST 写入当前项目沙箱 `.uploads/` 目录，并在输入区上方以可删除的缩略图 chip 展示；发送时随 `send_message` 一并提交附件元数据（相对路径、MIME）。

#### Scenario: vision 模型粘贴成功

- **WHEN** 当前会话模型 `supports_vision=true` 且用户粘贴一张 PNG
- **THEN** 系统保存至 `.uploads/`、展示缩略图 chip，发送后 user 消息持久化文本与 `attachments_json`

#### Scenario: 非 vision 模型粘贴跳过并提示

- **WHEN** 当前会话模型 `supports_vision=false` 且用户粘贴图片
- **THEN** 系统不保存附件、不插入 chip，并展示 toast 提示切换至 Kimi K2.6 或 MiMo v2.5

#### Scenario: 非 vision 模型发送含附件消息被拒绝

- **WHEN** 客户端绕过 UI 向 `send_message` 提交含 `attachments` 且会话模型 `supports_vision=false`
- **THEN** 系统返回明确错误且不持久化该消息（对齐 kimi-cli `check_message`）

#### Scenario: 发送后 API 多模态组装

- **WHEN** vision 模型用户消息含 1 个附件且文本非空
- **THEN** 发往 Provider 的 user `content` 为数组，含 `text` 与 `image_url`（`data:{mime};base64,...`）

#### Scenario: 仅粘贴图片无文字仍可发送

- **WHEN** 当前会话模型 `supports_vision=true`、用户粘贴图片且未输入文字并点击发送
- **THEN** 发送按钮可用、user 消息持久化空文本与 `attachments_json`、聊天区展示仅含缩略图的气泡
- **AND** 发往 Provider 的 user `content` 数组 MUST 含非空 `text` part（API 占位提示）与 `image_url` part

### Requirement: 附件持久化与历史展示

系统 SHALL 在 `messages` 表以 `attachments_json` 列存储附件列表（路径、MIME），MUST NOT 在数据库保存 base64。聊天历史 MUST 在 user 消息气泡中展示附件缩略图（从项目路径读取）。

#### Scenario: 重载会话后附件可见

- **WHEN** 用户重新打开含图片附件的会话
- **THEN** 消息列表展示历史缩略图，且 Agent 重建上下文时能再次编码为 `image_url`

### Requirement: 附件限制

系统 SHALL 限制单条 user 消息最多 4 个图片附件，单文件最大 50MB；超限 MUST 返回明确错误且不发送。

#### Scenario: 超过 4 张拒绝

- **WHEN** 用户尝试在第 5 张粘贴或发送
- **THEN** 系统提示超出上限并阻止发送
