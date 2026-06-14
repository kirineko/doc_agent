## 1. 依赖与基础模块

- [x] 1.1 添加 `@dnd-kit/core`、`@dnd-kit/sortable`、`@dnd-kit/utilities` 到 `package.json`
- [x] 1.2 实现 `src/lib/sessionOrder.ts`：`readProjectOrder`、`writeProjectOrder`、`hasManualOrder`、`applySessionOrder`、`prependSessionToOrder`、`removeSessionFromOrder`
- [x] 1.3 编写 `src/lib/sessionOrder.test.ts` 覆盖懒激活、orphan id、未知 session 置顶合并

## 2. 选中逻辑解耦

- [x] 2.1 修改 `mostRecentSessionId` 为按 `updated_at` 取最大值（`projectSession.ts`）
- [x] 2.2 更新 `projectSession.test.ts` 中「首项即最近」用例为显式 `updated_at` 比较

## 3. 工作区集成

- [x] 3.1 在 `useWorkspace` 封装 `setSessionsFromBackend(list, projectId)`，backend 刷新路径统一 apply order
- [x] 3.2 `selectProject` 与 `turn_complete` 的 `list_sessions` 改用上述 helper
- [x] 3.3 手动序下 `createSession` / `ensureSession` prepend 时同步 `prependSessionToOrder`
- [x] 3.4 删除会话时同步 `removeSessionFromOrder`（手动序项目）
- [x] 3.5 暴露 `reorderSessions(orderedIds)` 供 Sidebar 拖动回调

## 4. UI 组件

- [x] 4.1 新建 `SessionList.tsx`：`SortableContext` + 独立 drag handle + 保留选中/删除交互
- [x] 4.2 `Sidebar.tsx` 会话区委托给 `SessionList`，拖动结束调用 `reorderSessions`
- [x] 4.3 补充 drag handle 的 `aria-label` 等可访问性属性

## 5. 验证

- [x] 5.1 运行 `npm run typecheck && npm test && npm run build` 通过
- [x] 5.2 手动验证：未拖动时 turn_complete 仍会按 updated_at 置顶；拖动后顺序持久且 turn_complete 不改变序；新建置顶；切换项目选中 updated_at 最新
