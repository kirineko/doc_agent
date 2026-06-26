## MODIFIED Requirements

### Requirement: 工作区分栏布局持久化
系统 SHALL 将 **主三栏水平比例** 持久化于前端 `localStorage`，应用重启后恢复用户上次选择。首次访问或无有效缓存时 MUST 使用默认水平比例。**Inspector 当前 Tab MUST NOT 持久化**；每次应用启动 MUST 默认选中「项目文件」。布局持久化 MUST NOT 与 `doc-agent-last-session-config` 混用同一存储键。系统 MUST NOT 再持久化右侧 **上下垂直** 分栏比例。

#### Scenario: Inspector Tab 启动默认
- **WHEN** 用户重启应用
- **THEN** 右侧 Inspector 默认展示「项目文件」，MUST NOT 读取上次退出时的 Tab 偏好
