# image-read-tool Specification

## Purpose
TBD - created by archiving change add-multimodal-support. Update Purpose after archive.
## Requirements
### Requirement: image_read 工具

系统 SHALL 提供 `image_read` 工具，读取项目沙箱内一张或多张图片文件并通过 vision 能力生成文本描述。参数 MUST 包含 `paths`（项目相对路径数组，长度 1–4）；可选 `prompt`（理解指令，默认「请详细描述图片内容」）。**不再**接受单张 `path` 参数。

工具执行 MUST 使用当前会话已锁定的 vision 模型发起**独立** `chat/completions` 请求（单轮、无 tools）：user `content` 为多个 `image_url` part 与 text part（顺序：先图后文或先文后图须在实现中固定并文档化）。返回结果 MUST 为纯文本 JSON（含 `text`、`paths`、`count` 字段），供 Agent 后续推理。

#### Scenario: vision 模型读取单张 PNG

- **WHEN** 会话模型为 Kimi K2.6，Agent 调用 `image_read` 且 `paths` 为 `[".uploads/a.png"]`
- **THEN** 工具返回非空文本描述，且主 Agent 循环的 tool 消息仅含该文本

#### Scenario: 一次读取最多 4 张图

- **WHEN** Agent 调用 `image_read` 且 `paths` 含 4 张 `.cache/pdf/.../page_00N.png`
- **THEN** 单次 vision 子调用包含 4 个 `image_url`，返回合并文本

#### Scenario: 超过 4 张拒绝

- **WHEN** Agent 调用 `image_read` 且 `paths` 长度为 5
- **THEN** 返回参数错误，不发起 vision 子调用

#### Scenario: 自定义 prompt

- **WHEN** Agent 调用 `image_read` 且 `prompt` 为「按页序提取图中全部可见文字与公式」
- **THEN** 子调用将该 prompt 作为 text part 与图片一并发送

#### Scenario: 可读 cache 页图

- **WHEN** Agent 对 `.cache/pdf/<key>/page_001.png` 调用 `image_read`
- **THEN** 系统正常编码并理解，不要求路径位于 `.uploads/`

### Requirement: 按模型条件注册

当会话模型 `supports_vision=false` 时，系统 MUST NOT 向 Agent 暴露 `image_read` 工具定义（动态工具列表过滤）。

#### Scenario: DeepSeek 会话无 image_read

- **WHEN** 会话模型为 DeepSeek V4 Flash
- **THEN** `tools` 列表不包含 `image_read`

#### Scenario: MiMo v2.5 有 image_read

- **WHEN** 会话模型为 MiMo v2.5
- **THEN** `tools` 列表包含 `image_read`

### Requirement: 路径与格式校验

`image_read` MUST 仅接受沙箱内存在的图片文件；非图片或越界路径 MUST 返回带上下文的工具错误。

#### Scenario: 非图片文件报错

- **WHEN** Agent 对 `.docx` 调用 `image_read`
- **THEN** 返回说明需使用 office 读取工具的错误

### Requirement: 子调用 token 不计入会话

`image_read` 的 vision 子调用 MUST 为独立 `chat/completions` 请求；其子调用返回的 `usage` MUST NOT 写入会话 `token_count` 或 `pending_estimate`。主 Agent loop 下一次请求的 `usage` 为会话权威计数（tool 结果文本 token 由主 loop 自然计入）。

#### Scenario: 子调用不改变 session token_count

- **WHEN** `image_read` 子调用返回 `usage.total_tokens = 5_000` 且主 loop 尚未继续
- **THEN** 会话 `token_count` 保持子调用前的值

#### Scenario: 主 loop 继续后 usage 含 tool 文本

- **WHEN** `image_read` 完成后主 Agent 基于文本 tool 结果继续推理
- **THEN** 该步 API `usage` 更新会话 `token_count`，包含 tool 结果文本的 input token
