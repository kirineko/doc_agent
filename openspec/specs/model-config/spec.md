# model-config Specification

## Purpose
TBD - created by archiving change bootstrap-doc-agent-mvp. Update Purpose after archive.
## Requirements
### Requirement: 多模型选择
系统 SHALL 允许用户为**尚无 chat 消息的会话**（含草稿态 pending 配置）选择模型，至少支持 DeepSeek V4 Flash、DeepSeek V4 Pro、Kimi K2.6、MiMo v2.5、MiMo v2.5 Pro、MiMo v2.5 Pro Ultraspeed，并通过统一的 OpenAI 兼容 Provider 抽象接入。已有 chat 消息的会话 MUST NOT 允许切换模型。

#### Scenario: 空会话切换模型
- **WHEN** 用户在空会话的模型选择中选择 MiMo v2.5
- **THEN** 该会话后续首条请求使用 MiMo 的 base_url 与模型标识，且选择被持久化

#### Scenario: 有消息会话不可切换
- **WHEN** 会话已有一条 user 消息
- **THEN** 模型下拉不可编辑，展示当前模型只读信息

### Requirement: 思考模式开关
系统 SHALL 允许用户在**尚无 chat 消息的会话**（含草稿态 pending 配置）开启 / 关闭思考模式，并将其映射为各模型的 `thinking` 参数。已有 chat 消息的会话 MUST NOT 允许变更。

#### Scenario: 空会话关闭思考
- **WHEN** 用户在空会话关闭思考开关
- **THEN** 请求携带 `thinking.type = disabled`，模型不再返回 `reasoning_content`

#### Scenario: 有消息会话不可变更思考
- **WHEN** 会话已有 chat 消息
- **THEN** 思考开关不可编辑

### Requirement: 思考强度（按模型差异化）
系统 SHALL 为支持强度的模型在**尚无 chat 消息的会话**提供思考强度选择（high / max），并对不支持强度的模型（Kimi K2.6、MiMo v2.5、MiMo v2.5 Pro、MiMo v2.5 Pro Ultraspeed）隐藏该选项。已有 chat 消息的会话 MUST NOT 允许变更强度。

#### Scenario: DeepSeek 空会话显示强度
- **WHEN** 当前为空会话且模型为 DeepSeek 且思考开启
- **THEN** UI 显示 high / max 强度选择，并映射为 `reasoning_effort`

#### Scenario: Kimi 无强度
- **WHEN** 当前模型为 Kimi K2.6
- **THEN** UI 不显示思考强度选项，请求中不包含强度参数

#### Scenario: MiMo 无强度
- **WHEN** 当前模型为 MiMo v2.5、MiMo v2.5 Pro 或 MiMo v2.5 Pro Ultraspeed
- **THEN** UI 不显示思考强度选项，请求中不包含 `reasoning_effort`

### Requirement: API Key 安全存储
系统 SHALL 将各模型 API Key 存储于操作系统密钥链，不以明文写入数据库或日志。Key 在侧栏全局区域配置，供所有会话按 provider 复用。

#### Scenario: 配置并使用密钥
- **WHEN** 用户在侧栏全局区域输入某 provider 的 API Key 并保存
- **THEN** 密钥存入 OS keychain，该 provider 下所有会话发起请求时从 keychain 读取，界面与日志不回显明文

### Requirement: API Key 全局配置入口
系统 SHALL 在应用内提供与会话无关的 API Key 配置入口，至少覆盖 DeepSeek、Kimi 与 MiMo；已保存的 Key MUST 默认以折叠/摘要形式展示以降低视觉干扰，未配置时展开输入。Key 配置 MUST NOT 依赖 activeSession 存在才可访问。配置入口位于「模型与密钥」Drawer 内。

#### Scenario: 无会话时可配置 Key
- **WHEN** 用户已选项目但处于草稿态（无 activeSession）
- **THEN** 仍可在 Drawer 配置并保存 DeepSeek/Kimi/MiMo API Key

#### Scenario: 已保存 Key 低干扰展示
- **WHEN** 某 provider 的 API Key 已保存
- **THEN** Drawer 内以折叠摘要（如「已保存」）展示，不默认展开密码输入框

### Requirement: 默认会话模型配置
系统 SHALL 在创建新会话（含懒创建与侧栏新建）时，若用户未显式选择其他模型，默认使用 DeepSeek V4 Flash、thinking enabled、thinking effort high。

#### Scenario: 懒创建默认模型
- **WHEN** 用户在草稿态直接发送且未修改 pending 模型配置
- **THEN** 创建的会话 model 为 deepseek-v4-flash，thinking_enabled 为 true，thinking_effort 为 high

### Requirement: 会话模型锁定
系统 SHALL 在会话已存在至少一条 user 或 assistant 消息之后，禁止变更该会话的 model、thinking_enabled 与 thinking_effort；UI MUST 以只读展示，后端 MUST 拒绝非法 update 请求。

#### Scenario: 首条消息后锁定
- **WHEN** 会话已有 user 或 assistant 消息且用户尝试在侧栏切换模型
- **THEN** UI 不提供可编辑控件，若通过 IPC 强制更新则返回错误

#### Scenario: 空会话仍可改模型
- **WHEN** 会话尚无任何 user 或 assistant 消息
- **THEN** 用户可在侧栏修改模型与思考配置并持久化

### Requirement: 模型上下文上限

系统 SHALL 为每个模型暴露上下文长度上限 `max_context_size`：DeepSeek 系列 = 1_000_000，Kimi K2.6 = 256_000，MiMo v2.5 / MiMo v2.5 Pro / MiMo v2.5 Pro Ultraspeed = 1_000_000，Mock = 100_000。该上限供压缩触发判定使用。

#### Scenario: DeepSeek 上限为 1M

- **WHEN** 当前会话模型为 DeepSeek V4 Flash 或 Pro
- **THEN** `max_context_size` 为 1_000_000

#### Scenario: Kimi 上限为 256K

- **WHEN** 当前会话模型为 Kimi K2.6
- **THEN** `max_context_size` 为 256_000

#### Scenario: MiMo 上限为 1M

- **WHEN** 当前会话模型为 MiMo v2.5 或 MiMo v2.5 Pro
- **THEN** `max_context_size` 为 1_000_000

### Requirement: 流式响应 token 用量采集

系统 SHALL 在 OpenAI 兼容流式请求中携带 `stream_options.include_usage = true`，并在 SSE 解析中读取末尾包含 `usage` 的 chunk（`prompt_tokens`、`completion_tokens`、`total_tokens`），将其填入助手轮结果（`AssistantTurn`）。Mock Provider MUST 返回估算用量以贯通测试链路。MiMo 流式 usage chunk（`choices:[]` + `usage`）MUST 被正确解析。

#### Scenario: 真实 Provider 回报用量

- **WHEN** DeepSeek/Kimi 流式响应在末尾返回 usage chunk
- **THEN** 系统解析出 `total_tokens` 并随该轮结果一并返回，供上下文计数刷新

#### Scenario: MiMo 流式回报用量

- **WHEN** MiMo 流式响应在末尾返回 usage chunk
- **THEN** 系统解析出 `total_tokens` 并随该轮结果一并返回

#### Scenario: Mock Provider 提供估算用量

- **WHEN** 使用 Mock Provider 完成一轮响应
- **THEN** 返回非空的估算 usage，使压缩计数逻辑可在无真实 Key 时测试

### Requirement: 模型目录与 vision 能力

系统 SHALL 通过 Rust 侧 `ModelCatalog` 定义全部可用模型，并经 IPC `list_models` 暴露给前端。每个模型 MUST 包含：`id`、`label`、`provider`、`api_model`、`supports_vision`、`supports_effort`、`max_context_size`。

模型集合 MUST 为：

| id | provider | supports_vision | supports_effort | max_context_size |
|----|----------|-----------------|-----------------|------------------|
| deepseek-v4-flash | deepseek | false | true | 1_000_000 |
| deepseek-v4-pro | deepseek | false | true | 1_000_000 |
| kimi-k2.6 | kimi | true | false | 256_000 |
| mimo-v2.5 | mimo | true | false | 1_000_000 |
| mimo-v2.5-pro | mimo | false | false | 1_000_000 |
| mimo-v2.5-pro-ultraspeed | mimo | false | false | 1_000_000 |

#### Scenario: list_models 返回 vision 标记

- **WHEN** 前端调用 `list_models`
- **THEN** 响应中 `kimi-k2.6` 与 `mimo-v2.5` 的 `supports_vision` 为 true，DeepSeek 与 MiMo Pro 系列（`mimo-v2.5-pro`、`mimo-v2.5-pro-ultraspeed`）为 false

### Requirement: MiMo Provider

系统 SHALL 接入小米 MiMo OpenAI 兼容 API：`base_url=https://api.xiaomimimo.com`，鉴权为 `Authorization: Bearer`，`api_model` 分别为 `mimo-v2.5`、`mimo-v2.5-pro` 与 `mimo-v2.5-pro-ultraspeed`。secrets MUST 支持 provider key `mimo`。

#### Scenario: MiMo 会话使用正确端点

- **WHEN** 会话模型为 `mimo-v2.5`
- **THEN** 请求发往 `https://api.xiaomimimo.com/v1/chat/completions` 且 model 字段为 `mimo-v2.5`

### Requirement: MiMo 思考模式

MiMo 模型的思考参数 MUST 仅映射 `thinking.type`（`enabled`/`disabled`），MUST NOT 发送 `reasoning_effort` 或 Kimi 的 `keep: all`。

#### Scenario: MiMo 开启思考

- **WHEN** 会话为 MiMo v2.5 Pro 且 thinking_enabled=true
- **THEN** 请求 body 含 `thinking.type=enabled` 且无 `reasoning_effort`

