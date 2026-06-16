## MODIFIED Requirements

### Requirement: Tavily API Key 安全存储
系统 SHALL 将 Tavily API Key 以 provider `"tavily"` 存入现有 Secrets 存储（与模型 Key 相同机制），不以明文写入 SQLite 或 Agent 事件日志；界面与日志 MUST NOT 回显明文 Key。Tavily Key 的配置 UI MUST 位于 Header「密钥与服务」Drawer 的搜索服务分区。

#### Scenario: 保存 Tavily Key
- **WHEN** 用户在 Header 密钥 Drawer 的 Tavily 分区输入 API Key 并保存
- **THEN** 系统调用 `set_api_key("tavily", ...)` 持久化，侧栏 Web 搜索摘要变为「已启用」

#### Scenario: 清空 Tavily Key
- **WHEN** 用户在密钥 Drawer 清空已保存的 Tavily Key
- **THEN** Key 从 Secrets 移除，Web 搜索能力对该用户不可用，侧栏摘要变为「未启用」

#### Scenario: 启动即可配置 Tavily
- **WHEN** 用户打开应用且尚未选择项目
- **THEN** 仍可通过 Header 密钥 Drawer 配置 Tavily Key
