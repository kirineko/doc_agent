# 模型配置能力

## ADDED Requirements

### Requirement: 多模型选择
系统 SHALL 允许用户为会话选择模型，至少支持 DeepSeek V4 Flash、DeepSeek V4 Pro、Kimi K2.6，并通过统一的 OpenAI 兼容 Provider 抽象接入。

#### Scenario: 切换模型
- **WHEN** 用户在某会话的模型下拉中选择 Kimi K2.6
- **THEN** 该会话后续请求使用 Kimi 的 base_url 与模型标识，且选择被持久化

### Requirement: 思考模式开关
系统 SHALL 允许用户开启 / 关闭思考模式，并将其映射为各模型的 `thinking` 参数。

#### Scenario: 关闭思考
- **WHEN** 用户关闭思考开关
- **THEN** 请求携带 `thinking.type = disabled`，模型不再返回 `reasoning_content`

### Requirement: 思考强度（按模型差异化）
系统 SHALL 为支持强度的模型提供思考强度选择（high / max），并对不支持强度的模型隐藏该选项。

#### Scenario: DeepSeek 显示强度
- **WHEN** 当前模型为 DeepSeek 且思考开启
- **THEN** UI 显示 high / max 强度选择，并映射为 `reasoning_effort`

#### Scenario: Kimi 无强度
- **WHEN** 当前模型为 Kimi K2.6
- **THEN** UI 不显示思考强度选项，请求中不包含强度参数

### Requirement: API Key 安全存储
系统 SHALL 将各模型 API Key 存储于操作系统密钥链，不以明文写入数据库或日志。

#### Scenario: 配置并使用密钥
- **WHEN** 用户输入某模型的 API Key 并保存
- **THEN** 密钥存入 OS keychain，发起请求时从 keychain 读取，界面与日志不回显明文
