## Why

侧栏会话列表目前固定按 `updated_at` 降序排列，用户无法将常用会话固定在顶部或按个人习惯组织。需要支持拖动排序，并在前端持久化；同时保持未手动排序时的现有「最近活跃在上」行为，避免对大多数用户造成干扰。

## What Changes

- 侧栏会话列表支持拖动重排（独立 drag handle，不影响点击选中与删除）
- **懒激活**：项目从未被用户拖动过时，列表仍按后端 `updated_at DESC` 自动排序（与现网一致）
- 用户首次在某项目下完成拖动后，该项目进入**手动序模式**，顺序写入 `localStorage`（按 `projectId` 隔离）
- 手动序模式下：对话完成、标题更新等刷新**不改变**列表顺序；新建会话仍插入列表顶部
- 切换项目时 activeSession 仍按 `updated_at` 最新选取，与展示顺序解耦
- 新增 `@dnd-kit/core`、`@dnd-kit/sortable` 依赖；拆分 `SessionList` 组件

## Capabilities

### New Capabilities

（无独立新 capability；行为归入既有 workspace-ui / project-session）

### Modified Capabilities

- `workspace-ui`：侧栏会话列表增加拖动排序与前端持久化相关 UI/交互要求
- `project-session`：明确列表展示顺序（懒激活自动序 vs 手动序）与切换项目选中逻辑（仍按 `updated_at`）

## Impact

- **前端**：`Sidebar.tsx`（拆出 `SessionList`）、`useWorkspace.ts`、`projectSession.ts`；新增 `sessionOrder.ts`
- **依赖**：`@dnd-kit/core`、`@dnd-kit/sortable`（及 `@dnd-kit/utilities`）
- **后端**：无变更（排序持久化纯前端）
- **测试**：`sessionOrder` 单元测试；更新 `mostRecentSessionId` 相关测试
