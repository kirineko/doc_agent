## 1. 依赖与常量

- [x] 1.1 添加 `react-resizable-panels` 到 `package.json` 并安装
- [x] 1.2 新建 `src/lib/workspaceLayout.ts`：导出 `MAIN_LAYOUT_GROUP_ID`、`RIGHT_LAYOUT_GROUP_ID` 与默认 layout 常量
- [x] 1.3 编写 `src/lib/workspaceLayout.test.ts` 覆盖 groupId 与默认比例常量

## 2. 水平三栏布局

- [x] 2.1 新建 `WorkspaceLayout.tsx`：horizontal `Group` + `useDefaultLayout(main)`，包裹 Sidebar / ChatPanel / RightPanel
- [x] 2.2 新建或内联 `PanelSeparator` 水平样式（hover、`col-resize`、主题色）
- [x] 2.3 `App.tsx` 的 `<main>` 改用 `WorkspaceLayout`，移除子组件固定宽度假设
- [x] 2.4 `Sidebar.tsx` 移除 `w-72 shrink-0`，宽度由 Panel 控制

## 3. 右侧垂直分栏与折叠

- [x] 3.1 重构 `RightPanel.tsx`：vertical `Group` + `useDefaultLayout(right)`，默认 60/40
- [x] 3.2 `ToolChainPanel` 标题行增加折叠 chevron，接入 `panelRef.collapse()` / `expand()`
- [x] 3.3 `ProjectFileExplorer` 移除 `flex-[0.38]`，标题行增加折叠 chevron
- [x] 3.4 任一侧折叠时隐藏 vertical `Separator`；`collapsedSize` 保留标题行高度
- [x] 3.5 新建或内联 vertical `PanelSeparator` 样式（`row-resize`）

## 4. 验证

- [x] 4.1 运行 `npm run typecheck && npm test && npm run build` 通过
- [x] 4.2 手动验证：水平拖三栏、垂直拖 60/40、折叠/展开、重启后布局恢复、与会话模型配置 localStorage 互不影响
