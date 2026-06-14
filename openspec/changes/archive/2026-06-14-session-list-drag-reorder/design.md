## Context

- 会话列表由 `Sidebar.tsx` 渲染，`useWorkspace` 通过 `list_sessions` IPC 获取数据；后端固定 `ORDER BY updated_at DESC`。
- `turn_complete` 后会再次 `list_sessions` 刷新标题等元数据。
- `mostRecentSessionId` 当前取 `sessions[0]`，依赖后端排序。
- 主题已用 `localStorage`（`doc-agent-theme`）作为前端持久化先例。

## Goals / Non-Goals

**Goals:**

- 支持侧栏会话拖动排序，顺序按项目隔离并持久化于 `localStorage`
- **懒激活**：未拖动过的项目保持 `updated_at DESC` 自动序；首次拖动后进入手动序
- 手动序下新建会话仍置顶；对话完成不自动改变顺序
- 切换项目时 activeSession 仍按 `updated_at` 最大选取
- 使用 @dnd-kit，独立 drag handle，组件拆分符合体量规范

**Non-Goals:**

- 后端 SQLite 增加 `sort_index` 或 IPC 变更
- 项目列表拖动
- 跨设备/跨实例同步顺序
- 提供 UI 切换回自动序（清除 localStorage 即可，无显式按钮）

## Decisions

### 1. 持久化：localStorage，按 projectId 映射 sessionId[]

- **选择**：`doc-agent-session-order` → `Record<projectId, string[]>`
- **理由**：与 `theme.ts` 一致；用户明确要求前端持久化；零后端改动
- **懒激活语义**：某 `projectId` 在 storage 中**无条目** = 自动序；**有条目** = 手动序（即使数组为空也应视为已激活——实现上首次 drag 才写入）

### 2. 排序库：@dnd-kit/core + @dnd-kit/sortable

- **理由**：React 19 兼容、可访问性/键盘传感器、Tauri WebView 稳定
- **替代**：原生 HTML5 DnD（touch/体验差）；react-beautiful-dnd（停更）

### 3. 排序应用点：所有 `setSessions` 来自 backend 的路径统一 apply

集中 helper：

```text
displaySessions = hasManualOrder(projectId)
  ? applySessionOrder(backendList, readOrder(projectId))
  : backendList   // 已是 updated_at DESC
```

**必须覆盖**：

- `selectProject` → `list_sessions`
- `turn_complete` → `list_sessions`
- 本地 prepend（createSession / ensureSession）在手动序下同步 `prependSessionToOrder`

### 4. `mostRecentSessionId` 改为按 `updated_at` 取 max

- **理由**：展示序与选中逻辑解耦，满足既有 spec「切换项目选中最近会话」
- **影响**：`projectSession.ts` + 测试更新；`selectProject` 在 apply order **之前**或**之后**选取均可（只要按 updated_at）

### 5. UI：拆 `SessionList.tsx`，左侧 ⋮⋮ drag handle

- `listeners` 仅绑 handle，避免与选中/删除冲突
- `Sidebar.tsx` 委托会话区给 `SessionList`，控制 150 行组件上限

### 6. 手动序下新建会话

- 列表：`[newSession, ...sessions]`
- order：`[newId, ...existingOrder]`
- 与现网「新建置顶」一致

### 7. 未知 session 合并策略

`applySessionOrder` 中，backend 返回但不在 order 里的 session（如懒创建）→ **插入手动序顶部**（与新建一致）

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| `turn_complete` 直接 `setSessions` 冲掉手动序 | 所有 backend 刷新路径走 `applySessionOrder` |
| 清 localStorage 丢失手动序 | 可接受；回退自动序 |
| @dnd-kit 增加 bundle | 桌面应用可接受 (~15–30KB gzip) |
| 拖动与删除按钮误触 | 独立 drag handle + 删除保持 hover 显示 |

## Migration Plan

- 纯前端增量发布，无数据库 migration
- 现有用户无 storage 条目 → 行为与现网完全一致
- 回滚：移除 DnD 组件与 storage 读写即可

## Open Questions

（无——懒激活 + 手动序新建置顶已由产品确认）
