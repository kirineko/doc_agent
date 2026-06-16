## Context

- 侧栏左下有「模型」摘要按钮，点击后 `ModelSettingsDrawer` 从屏幕**右侧**全高滑出（`justify-end`），内含 Provider Tab、API Key、模型 radio、思考配置。
- Tavily Key 在侧栏底部 `WebSearchSection` 独立配置；LLM Key 在模型 Drawer 内按 Provider 展示。
- `ApiKeySection.tsx`（三 Provider 集中列表）已存在但未引用。
- 缺 LLM Key 发送时 `showSendBlocker` 打开 `ModelSettingsOpen` 并高亮 Key。
- 现有 spec 要求 Key 在「模型与密钥 Drawer」与侧栏 Web 搜索区块。

## Goals / Non-Goals

**Goals:**

- Header **密钥** + **设置** 双入口，视觉统一（28×28 icon button）
- 密钥 Drawer（右侧）：LLM Keys + Tavily Key，启动即可访问，不依赖项目/会话
- 模型 Flyout（侧栏左下锚定）：仅模型 + 思考；Provider Tab 切换时仍预选该 Provider 第一个模型
- 零 LLM Key 时每次启动显示 Header 弱提醒条 + 密钥按钮 amber dot
- 发送缺 Key 时打开密钥 Drawer（非模型 Flyout）
- 未选项目时隐藏模型 Flyout 入口

**Non-Goals:**

- 后端 Secrets / IPC 变更
- 首次启动 Modal 强制 onboarding
- 模型 Flyout 改为 Header 入口
- 弱提醒 dismiss 后永久隐藏（用户要求每次启动都显示）

## Decisions

### 1. Header 双入口组件

| 按钮 | 组件 | 行为 |
|------|------|------|
| 密钥 | `CredentialsButton` | 打开 `CredentialsDrawer`；无 LLM Key 时显示 6px amber dot |
| 设置 | `SettingsButton`（现有） | 打开 `SettingsDrawer`（版本/布局/余额） |

两按钮相邻排列于 Header 右侧（主题切换左侧），共用 `SettingsButton` 的 border/hover 样式。密钥 icon 使用 SVG 钥匙轮廓（非 emoji）。

### 2. CredentialsDrawer（右侧 w-80）

- 复用 `ApiKeySection` 渲染 DeepSeek / Kimi / MiMo（折叠摘要 + `ProviderKeyRow`）
- 将 `WebSearchSection` 的 Key 表单逻辑提取为 `TavilyKeyPanel`，嵌入 Drawer「搜索服务」分区；侧栏仅保留 `WebSearchStatus`（已启用/未启用 + 链接「配置密钥」打开 Drawer）
- 标题：「密钥与服务」；底部小字说明 OS keychain 存储
- `highlightApiKeyProvider` 打开时自动展开对应 Provider 并 scrollIntoView（保留现有高亮逻辑）

**备选**：合并进 SettingsDrawer Tab — 拒绝，用户明确要求两个入口。

### 3. ModelFlyout（锚定 Popover，替代 ModelSettingsDrawer）

**定位：**

```tsx
// useAnchorPosition(triggerRef) → { top, left, width, maxHeight, placement: 'above' | 'below' }
const rect = trigger.getBoundingClientRect();
const width = Math.min(320, sidebarWidth - 16);
const left = rect.left;
// 优先向上展开：bottom = window.innerHeight - rect.top + 8
// 空间不足则向下：top = rect.bottom + 8
```

- `position: fixed`，z-index 与现有 Drawer 同级（z-50）
- 轻遮罩 `bg-black/20`，点击关闭
- 侧栏 resize 时监听 layout 或 window resize 重新计算位置

**内容结构（自上而下）：**

1. **Sticky 摘要条**：当前模型 label + 思考状态；locked 时副文案「对话开始后不可更改」
2. **Provider segmented control**：切换 Tab 时调用现有 `configForProviderFirstModel`（保留预选）
3. **模型列表**：`max-h-[240px] overflow-y-auto`；card row + 选中 accent 左边框；vision tag
4. **Sticky 底部思考区**：checkbox + effort select（逻辑同 `ProviderModelPanel`）

**移除：** Drawer 内一切 API Key UI；locked 态 Flyout 只读展示摘要，不显示 Provider Tab 改模型。

**触发：** 侧栏 `#sidebar-model-trigger`，仅 `activeProjectId` 存在时渲染。

### 4. 零 Key 弱提醒

```ts
const anyLlmKey = API_PROVIDERS.some((p) => apiKeyStatus[p]);
```

- **Layer A**：`CredentialsButton` amber dot（`!anyLlmKey`）
- **Layer B**：Header 内弱提醒条 — `!anyLlmKey` 时**每次应用启动**显示（不写入 dismiss localStorage）；文案：「尚未配置模型 API Key，发送前需先配置。」+ 链接「去配置」打开 CredentialsDrawer
- **Layer C**：`SendHintBanner` + `showSendBlocker` → `setCredentialsOpen(true)` + `highlightApiKeyProvider`

Tavily 未配置 **不** 触发 Layer A/B（不阻断发送）。

### 5. 状态命名 refactor

| 旧 | 新 |
|----|-----|
| `modelSettingsOpen` | `modelFlyoutOpen` |
| — | `credentialsOpen` |
| `setModelSettingsOpen` | `setModelFlyoutOpen` / `openCredentialsDrawer` |

`useWorkspace` 导出上述 state；`App.tsx` 挂载两个 overlay 组件。

### 6. 文件变更计划

| 操作 | 文件 |
|------|------|
| 新增 | `CredentialsButton.tsx`, `CredentialsDrawer.tsx`, `ModelFlyout.tsx`, `WebSearchStatus.tsx`, `useAnchorPosition.ts`（或内联 hook） |
| 重构 | `Sidebar.tsx`, `App.tsx`, `useWorkspace.ts`, `WebSearchSection.tsx` → 拆分 TavilyKeyPanel |
| 删除/替换 | `ModelSettingsDrawer.tsx`（逻辑迁入 ModelFlyout） |
| 保留 | `ProviderModelPanel.tsx` 可拆为 Flyout 子组件或内联简化 |
| 测试 | `sendReadiness` 不变；可选 Flyout 渲染/anchor 单测 |

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 侧栏拖拽宽度导致 Flyout 错位 | resize 时重算 anchor；Flyout width 跟随 trigger 容器 |
| Flyout 向上展开被 Header 遮挡 | 检测空间，不足时改向下展开 |
| 每次启动弱提醒可能烦扰 | 用户明确要求；样式保持 muted，单行，无 modal |
| spec 与旧 UI 文档多处引用「模型与密钥 Drawer」 | 本 change 的 spec delta 全量 MODIFIED 相关 Requirement |

## Migration Plan

1. 实现 CredentialsDrawer + Header 按钮，迁移 Key 表单，侧栏 Tavily 改 status-only
2. 实现 ModelFlyout，删除 ModelSettingsDrawer，更新 Sidebar trigger
3. 更新 send blocker / 弱提醒 wiring
4. 删除未使用 import；跑 `npm run typecheck && npm test && npm run build`
5. Archive change 后合并 spec delta 到 `openspec/specs/`

Rollback：revert 前端 PR；Secrets 数据无迁移。

## Open Questions

（无 — 用户已确认：弱提醒每次启动显示；未选项目隐藏模型 Flyout。）
