## Context

- 主工作区 `App.tsx` 使用 flex 三栏：`Sidebar` 固定 `w-72`（288px）、`ChatPanel` `flex-1`、`RightPanel` 固定 `w-64`（256px）。
- `RightPanel` 内 `ToolChainPanel` 为 `flex-1`，`ProjectFileExplorer` 为 `flex-[0.38]`，比例不可调。
- 用户反馈文件资源管理器区域过小；Agent 工具链较长时垂直空间进一步被挤压。
- 项目已有前端 `localStorage` 先例：`doc-agent-theme`、`doc-agent-last-session-config`（模型配置）、`doc-agent-session-order`。
- 会话列表拖动已引入 `@dnd-kit`；面板 resize 场景更适合专用库。

## Goals / Non-Goals

**Goals:**

- 主布局水平可拖拽：Sidebar ↔ Chat ↔ RightPanel
- 右侧栏垂直可拖拽：ToolChain ↔ ProjectFileExplorer，默认 **60% / 40%**
- 工具调用链与项目文件支持标题栏折叠 / 展开
- 布局比例通过 `localStorage` 持久化，重启恢复
- 分割条与折叠控件样式与明暗主题一致

**Non-Goals:**

- 左侧栏折叠（仅宽度可调）
- 双击分割条重置默认比例（可后续加）
- 窗口过窄时的响应式自动折叠
- 后端 / IPC 变更
- 与 `sessionConfig` 共用存储键

## Decisions

### D1：依赖 `react-resizable-panels`（v4）

- **选择**：`Group` / `Panel` / `Separator` + `useDefaultLayout`
- **理由**：成熟、维护活跃（Brian Vaughn）、支持水平/垂直嵌套、`collapsible`、内置 localStorage 持久化；自研 pointer resize + 折叠联动成本高且易出 edge case
- **替代**：自研 splitter（零依赖但 a11y/嵌套/持久化需全包）；`react-split-pane`（维护弱于前者）

### D2：嵌套 Group 结构

```text
App main
└─ Group horizontal (groupId: doc-agent-layout-main)
   ├─ Panel sidebar   (default ~20%, min ~12%, collapsible: false)
   ├─ Separator
   ├─ Panel chat      (min ~35%, 占剩余)
   ├─ Separator
   └─ Panel right     (default ~20%, min ~12%)
      └─ Group vertical (groupId: doc-agent-layout-right)
         ├─ Panel toolchain  (default 60%, min ~15%, collapsible)
         ├─ Separator        (一侧折叠时隐藏)
         └─ Panel files      (default 40%, min ~15%, collapsible)
```

- **默认水平比例**：sidebar 20%、chat 60%、right 20%
- **默认垂直比例**：toolchain 60%、files 40%（产品确认）

### D3：持久化 — `useDefaultLayout` + 独立 groupId

- **选择**：
  - `doc-agent-layout-main` → 水平三栏比例
  - `doc-agent-layout-right` → 右侧上下比例
- **理由**：与库 v4 API 对齐；折叠后 size→0 一并持久化，重启恢复折叠态，符合 IDE 习惯
- **约束**：不得写入 `doc-agent-last-session-config`；该键仅用于 `{ model, thinking_enabled, thinking_effort }`

### D4：折叠 — Panel `collapsible` + 标题栏 chevron

- **选择**：`ToolChainPanel` / `ProjectFileExplorer` 标题行右侧 chevron，调用 `panelRef.collapse()` / `expand()`
- **折叠尺寸**：`collapsedSize` 保留标题行高度（约 28–32px），非完全隐藏
- **行为**：一侧折叠 → 另一侧占满 vertical group；两侧均折叠 → 右侧栏仅两条标题栏
- **分割条**：任一侧 `isCollapsed()` 时隐藏 vertical `Separator`

### D5：组件拆分

| 文件 | 职责 |
|------|------|
| `WorkspaceLayout.tsx`（新建） | 水平 Group + `useDefaultLayout(main)`，包裹 Sidebar / Chat / RightPanel |
| `RightPanel.tsx` | 垂直 Group + `useDefaultLayout(right)` + 折叠 wiring |
| `PanelSeparator.tsx`（可选） | 统一 Separator 样式（hover、cursor） |
| `lib/workspaceLayout.ts`（可选） | groupId 常量、defaultLayout 默认值，供测试 |

- `Sidebar` / `ChatPanel` 去掉对外层宽度的假设；Sidebar 移除 `w-72`，RightPanel 移除 `w-64`
- `ProjectFileExplorer` 移除 `flex-[0.38]`，高度由 Panel 控制

### D6：最小尺寸

| Panel | minSize（建议） |
|-------|----------------|
| sidebar | 12% |
| chat | 35% |
| right | 12% |
| toolchain | 15% |
| files | 15% |

防止三栏拖拽把会话区挤成不可用缝。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 新增依赖增加 bundle | 桌面 Tauri 应用可接受；库 ~12KB gzip 量级 |
| 百分比 default 导致 SSR/hydration 微移 | 纯客户端 Tauri，无 SSR；可忽略 |
| 极窄右侧栏文件名截断 | 已有 truncate；用户可拖宽 |
| 折叠 + 拖拽交互冲突 | 折叠走 chevron，拖拽走 Separator，职责分离 |
| 库 API v3→v4 命名变更 | 实现时锁定 v4 文档（Group/Separator/useDefaultLayout） |

## Migration Plan

- 纯前端增量，无数据库 migration
- 无 localStorage 缓存的用户首次打开使用 defaultLayout，观感接近现网
- 回滚：移除 Group 包装，恢复固定 `w-72` / `w-64` / `flex-[0.38]`

## Open Questions

（无——默认 60/40、含左侧水平拖拽、持久化、右侧折叠均已确认）
