## ADDED Requirements

### Requirement: 侧栏 Web 搜索配置区块
系统 SHALL 在侧栏提供独立于模型 API Key 区域的「Web 搜索 (Tavily)」配置入口，与会话无关；已保存 Key 时摘要显示「已启用」，未配置时显示「未启用」。交互 MUST 支持保存、更换、清空，且 MUST NOT 依赖 activeSession。

#### Scenario: 无会话时可配置 Tavily
- **WHEN** 用户已选项目但无 activeSession
- **THEN** 仍可在侧栏 Web 搜索区块配置 Tavily Key

#### Scenario: 与模型 Key 分区展示
- **WHEN** 用户打开侧栏
- **THEN** Web 搜索配置与 DeepSeek/Kimi API Key 区域分离展示，不混入模型 provider 列表

#### Scenario: 已保存 Key 低干扰展示
- **WHEN** Tavily Key 已保存
- **THEN** 区块以折叠摘要「已启用」展示，不默认展开密码输入框

### Requirement: Web 工具中文标签
系统 SHALL 为 `web_search` 与 `web_extract` 提供中文工具链标签，并在工具名注册列表测试中保持同步。

#### Scenario: 工具卡片显示中文名
- **WHEN** Agent 调用 `web_search` 或 `web_extract`
- **THEN** 右侧工具链卡片显示对应中文标签（非原始英文名）
