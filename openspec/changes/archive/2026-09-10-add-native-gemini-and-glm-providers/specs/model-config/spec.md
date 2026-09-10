## MODIFIED Requirements

### Requirement: 多模型选择
系统 SHALL 允许用户为**尚无 chat 消息的会话**（含草稿态 pending 配置）选择模型，新建选择器仅支持 DeepSeek Flash、MiMo v2.5、MiMo v2.5 Pro、Kimi K3、Gemini 3.8 Flash、GLM-5.3-Flash；所有 provider 使用各自 Chat Completions，Google 使用官方 OpenAI 兼容接口，通过统一模型调用契约接入。已有 chat 消息的会话 MUST NOT 允许切换模型。DeepSeek Flash 的产品 id 与请求模型名均为 `deepseek-flash`。

#### Scenario: 空会话切换模型
- **WHEN** 用户在空会话的模型选择中选择 MiMo v2.5
- **THEN** 该会话后续首条请求使用 MiMo 的 base_url 与模型标识，且选择被持久化

#### Scenario: 有消息会话不可切换
- **WHEN** 会话已有一条 user 消息
- **THEN** 模型下拉不可编辑，展示当前模型只读信息

### Requirement: 思考模式开关
系统 SHALL 允许用户在**尚无 chat 消息的会话**（含草稿态 pending 配置）按模型能力开启 / 关闭思考模式。DeepSeek、MiMo 及历史 Kimi K2.6 支持开关；Kimi K3、Gemini 3.8 Flash 和 GLM-5.3-Flash 始终思考，MUST 不显示关闭开关且请求校验拒绝显式关闭。已有 chat 消息的会话 MUST NOT 允许变更。

#### Scenario: 空会话关闭思考
- **WHEN** 用户在支持开关的模型空会话关闭思考开关
- **THEN** 请求携带 `thinking.type = disabled`，模型不再返回 `reasoning_content`

#### Scenario: 有消息会话不可变更思考
- **WHEN** 会话已有 chat 消息
- **THEN** 思考开关不可编辑

#### Scenario: 始终思考模型
- **WHEN** 空会话选择 Kimi K3、Gemini 或 GLM
- **THEN** UI 展示始终思考说明而非关闭开关，绕过 UI 的 disabled 请求被拒绝

### Requirement: 思考强度（按模型差异化）
系统 SHALL 为支持强度的模型在**尚无 chat 消息的会话**提供模型合法的思考强度选择：DeepSeek 和 Kimi K3/GLM 为 low/high/max，Gemini 为 low/medium/high，并对不支持强度的模型（Kimi K2.6、MiMo v2.5、MiMo v2.5 Pro、MiMo v2.5 Pro Ultraspeed）隐藏该选项。已有 chat 消息的会话 MUST NOT 允许变更强度。

#### Scenario: DeepSeek 空会话显示强度
- **WHEN** 当前为空会话且模型为 DeepSeek 且思考开启
- **THEN** UI 显示 low / high / max 强度选择，并映射为 `reasoning_effort`

#### Scenario: Kimi 无强度
- **WHEN** 当前模型为 Kimi K2.6
- **THEN** UI 不显示思考强度选项，请求中不包含强度参数

#### Scenario: MiMo 无强度
- **WHEN** 当前模型为 MiMo v2.5、MiMo v2.5 Pro 或 MiMo v2.5 Pro Ultraspeed
- **THEN** UI 不显示思考强度选项，请求中不包含 `reasoning_effort`

#### Scenario: K3 与 Gemini 档位不同
- **WHEN** 用户从 Kimi K3 切换到 Gemini
- **THEN** 档位变为 low/medium/high，默认 medium，不把 K3 max 原样带入

#### Scenario: GLM 默认强度
- **WHEN** 用户选择 GLM-5.3-Flash 且未指定档位
- **THEN** 默认使用 max，允许 low/high/max

### Requirement: API Key 全局配置入口
系统 SHALL 在应用 Header 提供与会话、项目均无关的「密钥」入口，打开「密钥与服务」Drawer；覆盖 DeepSeek、Kimi、MiMo、Google Gemini 与智谱 GLM（provider key 分别为 deepseek/kimi/mimo/google/zhipu）。已保存的 Key MUST 默认以折叠/摘要形式展示以降低视觉干扰，未配置时展开输入。Key 配置 MUST NOT 依赖 activeProject 或 activeSession 存在才可访问。

#### Scenario: 启动即可配置 Key
- **WHEN** 用户打开应用且尚未选择项目
- **THEN** 仍可通过 Header 密钥入口配置并保存 DeepSeek/Kimi/MiMo/Google/智谱 API Key

#### Scenario: 无会话时可配置 Key
- **WHEN** 用户已选项目但处于草稿态（无 activeSession）
- **THEN** 仍可在密钥 Drawer 配置并保存 DeepSeek/Kimi/MiMo/Google/智谱 API Key

#### Scenario: 已保存 Key 低干扰展示
- **WHEN** 某 provider 的 API Key 已保存
- **THEN** 密钥 Drawer 内以折叠摘要（如「已保存」）展示，不默认展开密码输入框

### Requirement: 模型上下文上限

系统 SHALL 为每个模型暴露上下文长度上限 `max_context_size`：DeepSeek 系列 = 1_000_000，Kimi K2.6 = 256_000，MiMo v2.5 / MiMo v2.5 Pro / MiMo v2.5 Pro Ultraspeed = 1_000_000，Kimi K3 与 GLM-5.3-Flash = 1_000_000，Gemini 3.8 Flash = 1_048_576，Mock = 100_000。该上限供压缩触发判定使用。

#### Scenario: DeepSeek 上限为 1M

- **WHEN** 当前会话模型为 DeepSeek Flash 或 Pro
- **THEN** `max_context_size` 为 1_000_000

#### Scenario: Kimi 上限为 256K

- **WHEN** 当前会话模型为 Kimi K2.6
- **THEN** `max_context_size` 为 256_000

#### Scenario: MiMo 上限为 1M

- **WHEN** 当前会话模型为 MiMo v2.5 或 MiMo v2.5 Pro
- **THEN** `max_context_size` 为 1_000_000

#### Scenario: 新模型上下文预算
- **WHEN** 当前模型为 Kimi K3、GLM 或 Gemini
- **THEN** K3/GLM 暴露 1_000_000 的保守上下文预算，Gemini 暴露 1_048_576，历史 K2.6 仍为 256_000

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

Google 兼容响应 SHALL 读取 prompt_tokens、completion_tokens、total_tokens，输出统计采用 max(completion_tokens,total_tokens-prompt_tokens)；总量以服务端值为准，输出统计包含思考，不重复累计增量事件中的同一用量。

#### Scenario: Google 独立思考用量
- **WHEN** Google 回报 prompt_tokens=43、completion_tokens=120、total_tokens=668
- **THEN** 记录输入43、输出625、总量668

#### Scenario: GLM 用量回报
- **WHEN** GLM 在末尾返回 usage
- **THEN** 用量进入主循环上下文计数，空推理文本不影响解析

### Requirement: 模型目录与 vision 能力

系统 SHALL 向前端提供全部可识别真实模型目录，包括 id、label、provider、api_model、supports_vision、supports_effort、上下文预算、selectable、availability、supports_thinking_toggle、thinking_efforts、默认思考配置及可选输出上限。supports_effort MUST 与档位列表非空一致。新建 UI MUST 仅列出 selectable 模型；历史查询 MUST 仍识别非 selectable 条目。

| id | provider | vision | selectable | availability |
|---|---|---|---|---|
| deepseek-flash | deepseek | true | true | available |
| mimo-v2.5 | mimo | true | true | available |
| mimo-v2.5-pro | mimo | false | true | available |
| kimi-k3 | kimi | true | true | available |
| gemini-3.8-flash | google | true | true | available |
| glm-5.3-flash | zhipu | true | true | available |
| deepseek-v4-pro | deepseek | false | false | available |
| kimi-k2.6 | kimi | true | false | available |
| mimo-v2.5-pro-ultraspeed | mimo | false | false | retired |

#### Scenario: list_models 返回 vision 标记
- **WHEN** 前端加载模型目录
- **THEN** DeepSeek Flash、Kimi K3、历史 K2.6、MiMo v2.5、Gemini、GLM 的 vision 为 true；DeepSeek Pro 与 MiMo Pro 系列为 false

#### Scenario: DeepSeek Flash 请求名
- **WHEN** 新建或写入会话使用 DeepSeek Flash
- **THEN** 会话 id 与发往 DeepSeek 的 `model` 字段均为 `deepseek-flash`，展示名为 DeepSeek Flash

#### Scenario: 历史 v4 Flash 别名
- **WHEN** 已有会话或草稿的模型 id 为 `deepseek-v4-flash`
- **THEN** 仍解析为 DeepSeek Flash，不批量改写数据库中的旧 id

#### Scenario: 历史 Flash 配置控件
- **WHEN** 打开模型 id 为 `deepseek-v4-flash` 的空会话配置
- **THEN** 模型选择器选中 DeepSeek Flash，按已有配置显示思考开关和合法强度
- **AND** 重复选择当前模型或 provider 不重置配置，已有消息的会话正确展示只读思考标签

#### Scenario: 历史目录不污染新建选择
- **WHEN** 用户新建会话，同时应用中存在旧 K2.6 会话
- **THEN** 新建下拉只有六个 selectable 模型，旧会话仍解析为 kimi provider 且支持图片


### Requirement: MiMo Provider

系统 SHALL 接入小米 MiMo OpenAI 兼容 API：`base_url=https://api.xiaomimimo.com`，鉴权为 `Authorization: Bearer`，`api_model` 分别为 `mimo-v2.5`、`mimo-v2.5-pro` ；`mimo-v2.5-pro-ultraspeed` 仅保留历史识别，标记 retired，不再发送远端请求。secrets MUST 支持 provider key `mimo`。

#### Scenario: MiMo 会话使用正确端点

- **WHEN** 会话模型为 `mimo-v2.5`
- **THEN** 请求发往 `https://api.xiaomimimo.com/v1/chat/completions` 且 model 字段为 `mimo-v2.5`

## ADDED Requirements

### Requirement: 历史名称与退役模型

系统 SHALL 保留历史会话中的原始模型名称和思考设置，不批量替换为新模型。DeepSeek Pro 与 Kimi K2.6 的旧会话可按原协议继续调用；MiMo Ultraspeed 会话可阅读，但发送、恢复、手动压缩 MUST 在远端请求前被明确拒绝。未知模型 MUST 不回退 Mock。

#### Scenario: 旧 K2.6 关闭思考会话
- **WHEN** 打开旧 K2.6 且 thinking_enabled=false 的会话
- **THEN** 显示原模型及关闭思考，后续请求不使用 K3 参数或模型 ID

#### Scenario: 退役 Ultraspeed
- **WHEN** 用户尝试续聊旧 Ultraspeed
- **THEN** 保留历史，提示新建会话使用 MiMo v2.5 Pro，不发送请求、不自动换模型

#### Scenario: 未知历史模型
- **WHEN** 历史 model 字符串未被目录识别
- **THEN** 可显示其原字符串和历史文本，但运行时明确报错，不产生 Mock 回答

### Requirement: 新建配置按模型规范化

系统 SHALL 在 UI 模型切换时应用目标模型默认值，在恢复同模型草稿时保留合法配置。新建/更新请求 MUST 校验模型可选性和思考能力；遗漏字段使用目标模型默认，显式非法值返回错误。已非空会话继续锁定。上次新建配置引用已移除模型时 SHALL 回退默认 DeepSeek Flash enabled/high，不改写已有会话。

#### Scenario: 非法档位被后端拒绝
- **WHEN** 创建 Gemini 会话显式提交 max，或创建 K3 会话显式关闭思考
- **THEN** 返回配置错误，不静默映射

#### Scenario: 合法 low 与 medium 可持久化
- **WHEN** 创建支持 low 的模型或 medium 的 Gemini 会话并重载
- **THEN** 档位原样恢复，不回退 high

#### Scenario: 旧草稿独立回退
- **WHEN** localStorage 上次新建配置为旧 Pro/Ultraspeed/K2.6
- **THEN** 新草稿回到默认模型，已有 session 数据不变

### Requirement: 新 provider 密钥与一致标签

系统 SHALL 在全局密钥入口支持 google/zhipu，缺 Key 时高亮对应行；密钥遵循现有安全存储契约。模型选择、发送前校验、图片能力和只读标签 MUST 使用一致的完整目录。新 provider MUST 不自动加入余额查询或改变智能建议的 DeepSeek 选择策略。

#### Scenario: 只配置 Google Key
- **WHEN** 用户仅配置 google 并选择 Gemini
- **THEN** 主聊天可发送，智能建议遵循原 DeepSeek 缺 Key 行为，余额查询不请求 Google

#### Scenario: 只配置智谱 Key
- **WHEN** 用户仅配置 zhipu 并选择 GLM
- **THEN** 主聊天可发送，Key 不写入 SQLite 或日志
