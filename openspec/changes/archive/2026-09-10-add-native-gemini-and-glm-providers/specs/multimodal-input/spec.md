## MODIFIED Requirements

### Requirement: 用户图片粘贴与附件

系统 SHALL 支持用户在聊天输入框通过剪贴板粘贴图片（`image/png`、`image/jpeg`、`image/webp`、`image/gif`）。粘贴成功后图片 MUST 写入当前项目沙箱 `.cache/attachments/` 目录，并在输入区上方以可删除的缩略图 chip 展示；发送时随 `send_message` 一并提交附件元数据（相对路径、MIME）。

#### Scenario: vision 模型粘贴成功

- **WHEN** 当前会话模型 `supports_vision=true` 且用户粘贴一张 PNG
- **THEN** 系统保存至 `.cache/attachments/`、展示缩略图 chip，发送后 user 消息持久化文本与 `attachments_json`

#### Scenario: 非 vision 模型粘贴跳过并提示

- **WHEN** 当前会话模型 `supports_vision=false` 且用户粘贴图片
- **THEN** 系统不保存附件、不插入 chip，并展示 toast 提示切换至当前可新建的视觉模型（DeepSeek Flash、MiMo v2.5、Kimi K3、Gemini 3.8 Flash、GLM-5.3-Flash）

#### Scenario: 非 vision 模型发送含附件消息被拒绝

- **WHEN** 客户端绕过 UI 向 `send_message` 提交含 `attachments` 且会话模型 `supports_vision=false`
- **THEN** 系统返回明确错误且不持久化该 message（对齐 kimi-cli `check_message`）

#### Scenario: 发送后 API 多模态组装

- **WHEN** vision 模型用户消息含 1 个附件且文本非空
- **THEN** Chat Completions 的 user content 含 text 与 image_url Data URL；Google 同样复用 text 与 image_url Data URL

#### Scenario: 仅粘贴图片无文字仍可发送

- **WHEN** 当前会话模型 `supports_vision=true`、用户粘贴图片且未输入文字并点击发送
- **THEN** 发送按钮可用、user 消息持久化空文本与 `attachments_json`、聊天区展示仅含缩略图的气泡
- **AND** 发往 Provider 的用户输入 MUST 含非空文本占位与对应协议的图片内容块

### Requirement: 附件持久化与历史展示

系统 SHALL 在 `messages` 表以 `attachments_json` 列存储附件列表（路径、MIME），MUST NOT 在数据库保存 base64。聊天历史 MUST 在 user 消息气泡中展示附件缩略图（从项目路径读取）。当附件文件不存在时，UI MUST 展示「无法加载」占位且不中断会话；Agent 重建上下文时 MUST 静默跳过缺失文件（见 `project-cache-layout`）。

#### Scenario: 重载会话后附件可见

- **WHEN** 用户重新打开含图片附件的会话且 `.cache/attachments/` 下文件仍存在
- **THEN** 消息列表展示历史缩略图，且 Agent 重建上下文时能再次按对应 provider 编码图片内容

#### Scenario: 附件文件缺失时历史降级

- **WHEN** 用户重新打开含图片附件的会话但磁盘文件已被删除
- **THEN** 消息列表展示「无法加载」占位，消息文本仍可见
- **AND** Agent 重建上下文时不因缺失附件失败

### Requirement: 附件限制

系统 SHALL 限制单条 user 消息最多 4 个图片附件，单文件最大 50MB；超限 MUST 返回明确错误且不发送。

#### Scenario: 超过 4 张拒绝

- **WHEN** 用户尝试在第 5 张粘贴或发送
- **THEN** 系统提示超出上限并阻止发送

Google 内联模式 SHALL 额外限制完整序列化请求为不超过20,000,000字节，包含文本、历史图片、工具定义和协议元数据。超限 MUST 在发送 HTTP 前拒绝并给出缩小/移除图片或压缩历史提示；不能静默丢图片或自动上传 Files API。

#### Scenario: 单图合法但总请求超限
- **WHEN** Google 单张附件符合50MB本地上限，但完整请求超过20,000,000字节
- **THEN** 不发 HTTP 请求，向用户解释总请求大小限制

#### Scenario: 历史图片也计入
- **WHEN** 新图片较小但历史图片使 Google 请求超限
- **THEN** 后端完整请求检查仍阻止发送，不只检查最新附件

## ADDED Requirements

### Requirement: 图片 UI 与历史能力一致

图片粘贴、选择按钮、发送前检查及视觉子调用 SHALL 使用一致的模型能力信息。历史 Kimi K2.6 仍可使用其视觉能力，新建目录 MUST 不再推荐已移除模型。

#### Scenario: 新 GLM、Gemini 与 DeepSeek Flash 图片入口
- **WHEN** 用户选择 GLM、Gemini 或 DeepSeek Flash
- **THEN** 图片粘贴和按钮均可用，消息气泡与现有视觉模型一致

#### Scenario: 历史 K2.6 保留能力
- **WHEN** 用户打开旧 K2.6 会话
- **THEN** 图片能力仍为 true，不因其从新建目录移除而禁用
