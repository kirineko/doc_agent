## MODIFIED Requirements

### Requirement: 左侧项目/会话/模型配置
系统 SHALL 在左侧栏展示项目与会话列表；模型详细配置 MUST 位于侧栏左下锚定的「模型」Flyout，侧栏仅展示模型摘要 trigger。模型配置在已选项目且（草稿态或空会话）时可编辑；会话已有 user/assistant 消息后 Flyout 为只读。侧栏 MUST 保留「新建」会话按钮，新建时不自动触发 starter。LLM API Key 配置 MUST NOT 出现在侧栏或模型 Flyout 内。

#### Scenario: 侧栏精简
- **WHEN** 用户打开侧栏
- **THEN** 项目与会话列表可见，完整模型选择与 Key 表单不在侧栏主区域展开

#### Scenario: 未选项目时无模型入口
- **WHEN** 用户尚未选择 activeProject
- **THEN** 侧栏不显示模型 Flyout trigger；Header 密钥入口仍可用

#### Scenario: 在 Flyout 切换会话与配置
- **WHEN** 用户在已选项目的空会话通过 Flyout 切换模型
- **THEN** 配置持久化且侧栏摘要更新

#### Scenario: 有消息会话模型只读
- **WHEN** 当前会话已有 user 或 assistant 消息
- **THEN** Flyout 以只读形式展示当前模型与思考配置，不可修改

### Requirement: 侧栏 Web 搜索配置区块
系统 SHALL 在侧栏提供「Web 搜索 (Tavily)」**状态摘要**（已启用 / 未启用），与会话、项目选择无关；侧栏 MUST NOT 包含 Tavily Key 输入表单。未启用时 MUST 提供跳转或文案引导用户至 Header 密钥 Drawer 配置 Tavily Key。

#### Scenario: 侧栏仅展示状态
- **WHEN** 用户打开侧栏
- **THEN** Web 搜索区块显示启用状态，不包含 Key 输入框

#### Scenario: 未配置时引导至密钥 Drawer
- **WHEN** Tavily Key 未配置且用户查看侧栏 Web 搜索区块
- **THEN** 显示「未启用」及可点击引导打开 Header 密钥 Drawer

#### Scenario: 已配置时低干扰展示
- **WHEN** Tavily Key 已保存
- **THEN** 侧栏以折叠摘要「已启用」展示，不展开 Key 表单

## REMOVED Requirements

### Requirement: 模型与密钥 Drawer
**Reason**: 模型配置与 API Key 配置拆分为模型 Flyout 与 Header 密钥 Drawer，不再使用合并的右侧「模型与密钥」Drawer。
**Migration**: 模型相关操作改用侧栏模型 Flyout；Key 相关操作改用 Header 密钥 Drawer。

## ADDED Requirements

### Requirement: Header 密钥与设置双入口
系统 SHALL 在应用顶栏右侧提供两个并列 icon 入口：**密钥**（打开「密钥与服务」Drawer）与 **设置**（打开现有设置 Drawer）。两入口 MUST 使用一致的按钮尺寸与边框样式（与现有设置按钮一致）。

#### Scenario: 密钥入口始终可见
- **WHEN** 用户打开应用
- **THEN** Header 显示密钥按钮，不依赖是否已选项目

#### Scenario: 设置入口保持独立
- **WHEN** 用户点击设置按钮
- **THEN** 打开设置 Drawer（版本、布局、余额），不包含 API Key 表单

### Requirement: 密钥与服务 Drawer
系统 SHALL 提供从 Header 密钥按钮打开的右侧 Drawer（标题「密钥与服务」），包含：（1）LLM 分区 — DeepSeek、Kimi、MiMo API Key；（2）搜索服务分区 — Tavily API Key。各 Key MUST 支持保存、更换、清空；已保存 Key 默认折叠摘要展示。

#### Scenario: 打开密钥 Drawer
- **WHEN** 用户点击 Header 密钥按钮
- **THEN** 右侧 Drawer 展示 LLM 与 Tavily Key 配置，不展示模型选择

#### Scenario: 高亮缺 Key 的 Provider
- **WHEN** 系统因发送拦截打开密钥 Drawer 并指定 provider
- **THEN** 对应 Provider 的 Key 行展开并视觉高亮

### Requirement: 模型 Flyout 锚定侧栏
系统 SHALL 在用户点击侧栏模型摘要 trigger 时，于 trigger **附近**（优先向上展开）显示固定定位 Flyout，而非屏幕右侧全高 Drawer。Flyout MUST 包含：当前模型摘要、Provider segmented control、可滚动模型列表、底部 sticky 思考配置。切换 Provider Tab 时 MUST 预选该 Provider 的第一个可用模型（与现有 `configForProviderFirstModel` 行为一致）。

#### Scenario: Flyout 靠近 trigger
- **WHEN** 用户点击侧栏左下模型 trigger
- **THEN** Flyout 在侧栏内、trigger 上方或下方展开，水平对齐 trigger，不要求用户视线移至屏幕最右侧

#### Scenario: Flyout 不含 Key 配置
- **WHEN** 用户打开模型 Flyout
- **THEN** 界面不包含任何 API Key 输入控件

#### Scenario: Provider Tab 预选首模型
- **WHEN** 用户在 Flyout 切换至 Kimi Provider Tab 且会话模型未锁定
- **THEN**  pending/空会话模型切换为该 Provider 列表中的第一个模型

#### Scenario: 侧栏摘要含 vision 标识
- **WHEN** 当前选中 Kimi K2.6
- **THEN** 侧栏模型 trigger 摘要显示模型名与视觉能力标识

### Requirement: 零 LLM Key 启动弱提醒
当尚未配置**任一** LLM Provider（DeepSeek/Kimi/MiMo）API Key 时，系统 SHALL 在每次应用启动后于 Header 区域展示非阻塞弱提醒条（单行 muted 样式），文案说明发送前需配置 Key，并提供「去配置」打开密钥 Drawer。密钥 Header 按钮 MUST 显示 amber 状态 dot。弱提醒 MUST NOT 使用 modal；用户关闭当次提醒条后，下次启动仍 MUST 再次显示（不持久 dismiss）。

#### Scenario: 每次启动显示弱提醒
- **WHEN** 用户启动应用且 `API_PROVIDERS` 中无任何 provider 已配置 Key
- **THEN** Header 显示弱提醒条与密钥按钮 amber dot

#### Scenario: 配置任一 LLM Key 后隐藏
- **WHEN** 用户已配置至少一个 LLM Provider Key
- **THEN** 弱提醒条与 amber dot 不再显示

#### Scenario: 仅缺 Tavily 不触发
- **WHEN** 用户已配置 DeepSeek Key 但未配置 Tavily Key
- **THEN** 不显示 LLM Key 弱提醒条（Tavily 不阻断发送）

#### Scenario: 关闭提醒不影响下次启动
- **WHEN** 用户在当次会话中关闭弱提醒条但未配置 Key，随后重启应用
- **THEN** 弱提醒条再次显示
