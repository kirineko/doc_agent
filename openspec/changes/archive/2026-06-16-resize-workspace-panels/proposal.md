## Why

用户反馈右侧项目文件资源管理器区域过小、无法按需调整；当前三栏与右侧上下分栏均为固定尺寸（Sidebar `w-72`、RightPanel `w-64`、文件区 `flex-[0.38]`），在 Agent 工具链较长时进一步挤压文件浏览空间。需要提供可拖拽调整的分栏与折叠能力，并将布局偏好持久化，以适配不同屏幕与用户工作流。

## What Changes

- 引入 `react-resizable-panels`，替换主工作区固定宽度三栏布局为可水平拖拽的 Sidebar ↔ 会话 ↔ 右侧栏
- 右侧栏内工具调用链与项目文件改为可垂直拖拽分栏，默认比例 **60% / 40%**（工具链 / 文件）
- 工具调用链与项目文件两个区域支持标题栏折叠 / 展开
- 主布局与右侧内部分栏尺寸通过 `localStorage` 持久化（`useDefaultLayout`）
- 为拖拽分割条与折叠控件补充样式，与现有明暗主题一致

## Capabilities

### New Capabilities

（无独立新 capability；行为归入既有 `workspace-ui`）

### Modified Capabilities

- `workspace-ui`：三栏布局与右侧上下分栏由固定尺寸改为可拖拽、可折叠、可持久化

## Impact

- **前端**：`App.tsx`（主 `<main>` 布局）、`RightPanel.tsx`、`Sidebar.tsx`（移除固定 `w-*`）、新增布局相关组件或 hook；`ToolChainPanel` / `ProjectFileExplorer` 标题行增加折叠入口
- **依赖**：新增 npm 包 `react-resizable-panels`（需在 design.md 说明理由）
- **持久化**：新增 layout groupId（与现有 `doc-agent-theme`、`doc-agent-last-session-config` 并列，不混用 sessionConfig）
- **测试**：布局默认值与持久化 helper 单元测试；关键折叠 / 渲染 smoke 测试（可选）
- **Rust / IPC**：无变更
