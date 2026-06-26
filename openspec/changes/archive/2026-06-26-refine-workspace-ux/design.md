## Context

Doc Agent 工作区当前为「顶栏 + 三栏 panel 卡片」：`Sidebar` 分区展示项目/会话/模型；`RightPanel` 用 `react-resizable-panels` 垂直分割工具链 Tab 区与文件浏览；`ChatPanel` 输入 dock 在底部。`workspace-ui` spec 已规定大量交互（模型 Flyout、构建产物、per-session running 等），本次为 **presentation + IA 重构**，后端契约基本不变。

参考：Cursor 侧栏项目树 + 命令面板；Notion 侧栏 page 列表与 centered AI 输入。

## Goals / Non-Goals

**Goals:**

- 侧栏改为项目 → 会话树，仅展开 active 项目
- 新建会话始终绑定正确 `project_id`
- 空态居中 Composer，有消息后回到底部
- 右栏统一 Inspector 三 Tab，默认文件浏览
- tool activity 智能切 Tab，尊重用户手动选择
- ⌘K / Ctrl+K 统一搜索与快捷操作
- Notion 味视觉加强（light 为主，dark 同步 flat 化）

**Non-Goals:**

- 改为两栏或 overlay 右栏
- 修改 session/project 数据库 schema
- 命令面板内执行 Agent turn 或发送消息
- 重写 Markdown/工具链卡片内容

## Decisions

### D1. 侧栏组件：`ProjectSessionTree`

新建 `ProjectSessionTree.tsx`，替换 `Sidebar` 内项目/会话区块。数据结构仍用 `projects[]` + `sessions[]`（按 `project_id` 过滤）。

- **手风琴**：`expandedProjectId === activeProjectId`；切换项目时折叠其它
- **新建会话**：
  - 顶栏按钮 / ⌘N：`createSession()`（已有逻辑，依赖 `activeProjectRef`）
  - 项目行 `[+]`：若 `projectId !== activeProjectId` 则 `await selectProject(projectId)` 再 `createSession()`
- **样式**：Notion sidebar item — 无 border card，`hover:bg-hover`，active 左侧 2px accent

**备选**：保留 `SessionList` 独立组件 — 拒绝，会话必须视觉嵌套在项目下。

### D2. 项目上下文菜单

项目行 `···` 触发 popover/menu：

| 项 | 动作 |
|----|------|
| 在 Finder 中打开 | `invoke('open_project_root', { projectId })`（已有 IPC） |
| 从列表移除 | 现有 `hide_project` |

macOS 文案「在 Finder 中打开」，Windows「在文件资源管理器中打开」，可用平台检测或统一「在文件夹中打开」。

### D3. 右侧 `InspectorTabs`

删除 `RightPanel` 内 vertical `Group`；单容器顶部 segmented control：

```
[ 项目文件 | 工具调用链 | 构建产物 (badge) ]
```

- 默认 Tab：`files`（localStorage key `doc-agent-inspector-tab`，无效时 fallback `files`）
- 内容区全高：`ProjectFileExplorer` / `ToolChainPanel` / `BuildArtifactsPanel`（去各自 duplicate header）
- 整栏折叠：保留 collapsible，折叠时 icon rail（📁🔧📦）点击展开并切 Tab

**备选**：保留上下分栏仅改比例 — 拒绝，用户明确要求三 Tab 统一切换。

### D4. 智能 Tab 切换

状态（per `session_id` 或 global inspector state）：

```ts
type InspectorTab = 'files' | 'toolchain' | 'artifacts';
// userPinnedTab: InspectorTab | null
// lastManualTabSwitchAt: number
```

规则：

1. 收到首个 `ToolCall { status: running }`（或 streaming 占位）且 `userPinnedTab === null` → 切 `toolchain`
2. 用户点击 Tab → 设 `userPinnedTab = tab`，记录 timestamp
3. 新 turn 开始（用户 send）→ 清除 `userPinnedTab`
4. 构建产物累积 → 仅更新 badge，**不** auto-switch

`PIN_GRACE_MS = 8000`（可选）：手动切换后 8s 内不 auto-switch，即使 pin 被清除 — 实现时可简化为「本 turn 内 pin 直至 send」。

### D5. 空态居中 Composer

`ChatPanel` 派生 `layoutMode: 'empty' | 'chat'`：

- `empty`：`visibleMessages.length === 0` 且无 streaming
- 空态：flex 居中，`max-w-[720px]` composer + `ComposerContextBar` + 问候 + Init 胶囊
- `chat`：现有底部 dock 布局
- 切换：`layoutMode` 变化时 CSS transition（`transition-all duration-200`），**不**清空 input/attachments

`ComposerContextBar` 从 ChatPanel 标题行迁出：项目 popover、**模型 trigger + ModelFlyout（自侧栏迁入，为模型选择唯一入口）**、`AgentsMdIndicator`、`ContextUsageIndicator`。

**ModelFlyout 锚定变更**：

- trigger 从 `#sidebar-model-trigger` 迁至上下文条内 `#composer-model-trigger`
- Flyout 定位逻辑复用 `ModelFlyout.tsx`，`triggerRef` 改绑 composer trigger；优先向上展开以免遮挡 textarea
- `App.tsx` 中 `modelFlyoutOpen` / `composerFocusBlockers` 仍生效，但状态由 `ComposerContextBar` 或 `ChatPanel` 持有，**Sidebar 不再传递 `onModelFlyoutOpenChange`**
- 侧栏删除「模型」区块与 Web 搜索之上的 model 区域

**备选**：侧栏保留只读模型摘要 — 拒绝，用户明确要求模型选择仅在上下文条。

### D6. 命令面板 `CommandPalette`

- 组件：`CommandPalette.tsx` + `useCommandPalette.ts`
- 打开：⌘K / Ctrl+K（`useEffect` + `keydown` on window）、侧栏搜索按钮
- 数据源：
  - Actions：新建会话、添加项目、打开设置（可选）
  - Projects：`projects` fuzzy
  - Sessions：当前项目或全项目 sessions fuzzy
  - Commands：`SLASH_COMMANDS` + `searchSlashCommands`
- 选择行为：复用 `selectProject`、`onSelectSession`、`pickProject`、`insertSlashPrompt`
- UI：Notion Quick Find 风 — modal overlay、`text-sm`、分组标题 muted

不引入新 npm 依赖；fuzzy 复用 `@`/`/` 已有 util 或轻量子序列匹配。

### D7. 顶栏瘦身

`App.tsx` header 移除项目名副标题；`CredentialsHintBanner` 保留。项目上下文仅出现在 Composer 上下文条与侧栏树。

### D8. Notion 视觉 token

`index.css` 调整：

- `.panel`：`border-radius: 0.5rem` 或侧栏/主区去 panel 改用 `border-r`
- `WorkspaceLayout` `main`：去掉 `p-2.5`，三栏贴边
- 侧栏项：移除 `item-surface` border，改用 hover/active 背景
- Composer：`rounded-xl shadow-sm border border-border-subtle`

Dark 主题同步减 nested panel 感，不改 hue 体系。

### D9. 布局持久化迁移

- 移除 `RIGHT_LAYOUT_GROUP_ID` vertical layout 读写（`rightLayoutStorage`）
- `clearStoredWorkspaceLayouts` / 设置「重置布局」仅清 horizontal main layout + inspector tab preference
- 首次升级：旧 vertical layout key 忽略即可

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 空态↔chat 过渡导致 composer 焦点丢失 | 过渡结束后 `useComposerFocus` 补 focus；遵循现有 overlay 抑制规则 |
| 智能 Tab 切换打扰正在看文件的用户 | user pin + 本 turn  scoped；默认仅 tool **running** 触发 |
| ⌘K 与系统/输入法快捷键冲突 | Tauri 内 `preventDefault`；文档注明 |
| Spec MODIFIED 范围大，archive 合并冲突 | delta 写完整 requirement 块，archive 前 diff 主 spec |
| 组件测试面大 | 优先 `ProjectSessionTree`、`InspectorTabs`、`commandPaletteSearch` 单测 |

## Migration Plan

1. 实现 Inspector 三 Tab（可独立验证）
2. 侧栏树 + 项目菜单
3. Composer 上下文条 + ModelFlyout 迁入 + 顶栏瘦身；侧栏删除模型区
4. 空态居中布局
5. 命令面板 + 快捷键
6. 视觉 token + 布局存储清理
7. 更新/新增 Vitest；`npm run release:check`

无数据迁移；localStorage 旧 vertical layout 键可残留。

## Open Questions

（已关闭）

- 多项目展开 → **仅 active**（用户确认）
- Finder 菜单、智能切换、命令面板 → **全部纳入本 change**
