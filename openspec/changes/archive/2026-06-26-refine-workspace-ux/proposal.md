## Why

当前工作区存在侧栏信息架构陈旧（项目/会话平级分区、全宽 primary「选择目录」按钮）、顶栏与中间区空态利用率低、右侧上下分栏在 idle 时大量占位等问题。用户期望借鉴 Cursor 的导航效率，同时保持并强化 Notion 风格视觉。需在不大改后端的前提下，统一重构侧栏、空态 Composer、右侧 Inspector 与全局命令入口。

## What Changes

- **侧栏**：改为 Cursor 式「项目折叠组 → 会话」树形导航；同时仅展开 active 项目；顶部「新建会话」「搜索」动作区；「添加项目目录」改为 ghost 入口
- **项目菜单**：项目行 `···` 上下文菜单，含「在 Finder/文件管理器中打开」、从列表移除等
- **新建会话**：侧栏顶部与项目行 `[+]` 均 MUST 在对应项目上下文中创建会话（非 active 项目时先切换再创建）
- **顶栏**：瘦身，项目名/上下文占用迁至 Composer 上下文条
- **空态 Composer**：无消息时居中展示输入区 + 问候/引导；有消息后过渡回底部 dock
- **Composer 上下文条**：项目切换、**模型选择与 Flyout**、AGENTS.md、上下文占用 %（**模型 MUST NOT 再出现于侧栏**）
- **右侧 Inspector**：合并为单一右栏三 Tab（**项目文件** | **工具调用链** | **构建产物**），默认「项目文件」；移除上下垂直分栏
- **智能 Tab 切换**：Agent 开始 tool call 时自动切到工具链（用户手动切换后本 turn 内暂停 auto-switch）
- **命令面板**：`⌘K` / `Ctrl+K` 搜索项目、会话、斜杠命令与快捷操作；侧栏搜索入口打开同一面板
- **Notion 视觉**：侧栏无卡片边框、flat 分栏、composer 大圆角浅阴影、减少 uppercase 分区标签

## Capabilities

### New Capabilities

（无独立新 capability；能力均归入 `workspace-ui` delta）

### Modified Capabilities

- `workspace-ui`：侧栏树形导航、项目菜单、空态居中 Composer、Composer 上下文条（**模型 Flyout 迁入**）、Inspector 三 Tab、智能 Tab 切换、命令面板、Notion 视觉、顶栏瘦身；移除右侧上下分栏与侧栏模型区

## Impact

- **前端**：`Sidebar.tsx`（或拆为 `ProjectSessionTree`）、`RightPanel.tsx`、`ChatPanel.tsx`、`ComposerContextBar.tsx`（**含 ModelFlyout 锚定**）、`ModelFlyout.tsx`（trigger 迁出侧栏）、`App.tsx`、新建 `CommandPalette.tsx`、`InspectorTabs.tsx`；`index.css` 设计 token；`workspaceLayout.ts`
- **Rust / IPC**：复用现有 `open_project_root`、`create_session`；命令面板无新 IPC
- **测试**：侧栏树交互、Inspector Tab、命令面板 fuzzy、空态布局切换、智能 Tab 切换逻辑
- **Spec**：`workspace-ui` 大量 MODIFIED/ADDED/REMOVED delta

## 纳入 / 排除

**纳入**

- 项目折叠组（仅展开 active）
- 项目内新建会话交互 + 项目 `[+]` + 顶栏「新建会话 ⌘N」
- 项目 `···` 菜单（打开根目录、移除）
- 空态居中 Composer + 过渡动画
- Composer 上下文条（含 Model Flyout，侧栏移除模型区）
- Inspector 三 Tab，默认项目文件
- 智能 Tab 切换（含 user pin）
- ⌘K / Ctrl+K 命令面板
- Notion 风视觉 token 与侧栏/分栏 flat 化
- 顶栏瘦身

**排除**

- 后端项目/会话数据模型变更
- 右栏完全隐藏或改为 overlay drawer（仍保持三栏 spec）
- dark 主题重设计（仅同步 flat 化，不推翻 palette）
- 全局 type-to-focus
- 多项目同时展开
