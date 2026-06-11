## ADDED Requirements

### Requirement: 两档主题与默认深色

系统 SHALL 支持 `dark` 与 `light` 两档应用主题；首次启动或 `localStorage` 无有效记录时 MUST 使用 `dark` 主题。`light` 主题 SHALL 呈现 Notion 感浅色视觉（暖白背景、柔和灰边框、深色正文），`dark` 主题 SHALL 保持与变更前等效的深色工作区观感。

#### Scenario: 首次启动默认深色

- **WHEN** 用户首次打开应用且本地无已保存主题偏好
- **THEN** 界面以深色主题渲染，与工作区布局及三栏结构正常展示

#### Scenario: 切换到浅色 Notion 风格

- **WHEN** 用户将主题切换为 `light`
- **THEN** 应用背景为暖白系、面板为白底浅灰边框、正文为深灰可读色，整体风格接近 Notion 浅色模式

### Requirement: 主题偏好持久化

系统 SHALL 将用户选择的主题（`dark` 或 `light`）写入浏览器 `localStorage`（键名 `doc-agent-theme`）；应用重启后 MUST 自动恢复上次有效选择。

#### Scenario: 重启后保持浅色

- **WHEN** 用户已切换为 `light` 并关闭后重新打开应用
- **THEN** 应用以 `light` 主题启动，无需用户重新切换

#### Scenario: 非法存储值回退

- **WHEN** `localStorage` 中 `doc-agent-theme` 值不是 `dark` 或 `light`
- **THEN** 系统回退为 `dark` 主题并正常展示

### Requirement: 语义主题 token 全覆盖

系统 SHALL 通过 `html` 元素上的 `data-theme` 与 CSS 语义变量驱动全局背景、面板、边框、主/次文字、消息气泡、工具卡片、Markdown 正文与代码块区域；上述区域在 `light` 与 `dark` 下 MUST 均使用主题 token 而非仅覆盖顶栏或单栏。

#### Scenario: 工具链卡片随主题变化

- **WHEN** 用户在 `dark` 与 `light` 间切换且右侧存在工具调用卡片
- **THEN** 工具卡片背景、边框与文字随主题同步更新，不出现固定深色块嵌在浅色背景中

#### Scenario: Markdown 代码高亮随主题变化

- **WHEN** 会话区展示含代码块的 Markdown 且用户切换主题
- **THEN** 代码块背景与高亮样式与当前主题一致（深色主题使用深色高亮方案，浅色主题使用浅色高亮方案）
