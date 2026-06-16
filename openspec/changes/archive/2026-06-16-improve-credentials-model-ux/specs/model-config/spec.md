## MODIFIED Requirements

### Requirement: API Key 安全存储
系统 SHALL 将各模型 API Key 存储于操作系统密钥链，不以明文写入数据库或日志。Key 在 Header「密钥与服务」Drawer 中配置，供所有会话按 provider 复用。

#### Scenario: 配置并使用密钥
- **WHEN** 用户在 Header 密钥 Drawer 输入某 provider 的 API Key 并保存
- **THEN** 密钥存入 OS keychain，该 provider 下所有会话发起请求时从 keychain 读取，界面与日志不回显明文

### Requirement: API Key 全局配置入口
系统 SHALL 在应用 Header 提供与会话、项目均无关的「密钥」入口，打开「密钥与服务」Drawer；至少覆盖 DeepSeek、Kimi 与 MiMo。已保存的 Key MUST 默认以折叠/摘要形式展示以降低视觉干扰，未配置时展开输入。Key 配置 MUST NOT 依赖 activeProject 或 activeSession 存在才可访问。

#### Scenario: 启动即可配置 Key
- **WHEN** 用户打开应用且尚未选择项目
- **THEN** 仍可通过 Header 密钥入口配置并保存 DeepSeek/Kimi/MiMo API Key

#### Scenario: 无会话时可配置 Key
- **WHEN** 用户已选项目但处于草稿态（无 activeSession）
- **THEN** 仍可在密钥 Drawer 配置并保存 DeepSeek/Kimi/MiMo API Key

#### Scenario: 已保存 Key 低干扰展示
- **WHEN** 某 provider 的 API Key 已保存
- **THEN** 密钥 Drawer 内以折叠摘要（如「已保存」）展示，不默认展开密码输入框

## ADDED Requirements

### Requirement: 发送缺 Key 时打开密钥 Drawer
当用户因缺少 LLM Provider API Key 而无法发送时，系统 SHALL 打开 Header 密钥 Drawer（而非模型 Flyout），并高亮对应 Provider 的 Key 配置行。

#### Scenario: 缺 DeepSeek Key 发送
- **WHEN** 用户尝试发送消息且当前模型 provider 为 deepseek 但未配置 Key
- **THEN** 系统展示 send hint，并打开密钥 Drawer 且高亮 DeepSeek Key 行
