## Why

当前「模型与密钥」混在同一右侧 Drawer，且从侧栏左下角点击后在屏幕右上角展开，视线与鼠标动线割裂；LLM Key 与 Tavily Key 分散在 Drawer 与侧栏两处，启动后未选项目时难以配置 Key。需要拆分凭证与模型配置、改善空间锚点，并在零 Key 时给出持续弱提醒。

## What Changes

- Header 新增 **密钥** 入口（与 **设置** 并列），打开「密钥与服务」Drawer，集中配置 DeepSeek / Kimi / MiMo / Tavily API Key
- 侧栏模型入口改为 **左下锚定 Flyout**（Popover），仅含 Provider Tab、模型选择与思考配置；移除 Key 表单
- 侧栏 Web 搜索区块 **移除 Key 表单**，仅保留启用状态摘要；未配置时引导至密钥 Drawer
- 未配置任一 LLM Provider Key 时：**每次应用启动** 在 Header 显示弱提醒条；密钥按钮显示状态 dot
- 发送拦截（缺 Key）改为打开 **密钥 Drawer** 并高亮对应 Provider，不再打开模型 Flyout
- 优化模型 Flyout UI：当前模型摘要、segmented Provider、可滚动模型卡片、底部 sticky 思考区
- 未选项目时 **隐藏** 侧栏模型 Flyout 入口；密钥入口始终可用
- 移除或废弃「模型与密钥」右侧全高 Drawer（**BREAKING**：UI 入口与交互变更，后端 IPC 不变）

## Capabilities

### New Capabilities

（无 — 行为变更均落在既有 capability spec delta 中）

### Modified Capabilities

- `model-config`：Key 配置入口从「模型与密钥 Drawer / 侧栏」迁至 Header 密钥 Drawer；模型选择入口改为侧栏 Flyout
- `workspace-ui`：Header 双入口、密钥 Drawer、模型 Flyout 锚点与布局、零 Key 弱提醒、侧栏 Web 搜索区块简化
- `web-search`：Tavily Key 配置 UI 从侧栏迁至密钥 Drawer

## Impact

- 前端：`App.tsx`、`Sidebar.tsx`、新建 `CredentialsDrawer` / `CredentialsButton` / `ModelFlyout`，重构或替换 `ModelSettingsDrawer`、`WebSearchSection`、`ApiKeySection`；`useWorkspace` 中 send blocker 与 open state 分流
- Spec：`openspec/specs/model-config`、`workspace-ui`、`web-search` 若干 Requirement 文案更新
- 后端：无变更（`set_api_key` / `has_api_key` / `list_models` 不变）
