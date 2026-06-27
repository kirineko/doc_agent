# 设计：界面缩放配置

## Context

- Doc Agent 为 Tauri 2 桌面应用（React 19 + Tailwind v4），主窗口默认 1400×900，无 UI 缩放能力。
- 主题偏好已通过 `localStorage` + `useTheme` 模式持久化（`doc-agent-theme`），设置抽屉已有 `config-surface` 区块（版本、工作区布局、余额）。
- 全局快捷键已占用 ⌘/Ctrl+K（命令面板）、N（新建会话）、O（添加项目）；**⌘/Ctrl ± / 0 未被占用**。
- Tauri WebView 原生热键（`zoomHotkeysEnabled: true`）在 macOS/Linux 上以 **20% 步进** 缩放，但当前 `@tauri-apps/api@2.11.0` 的 `webview.d.ts` **未导出 `setZoom`**；Rust `Webview::set_zoom` 可用。实现时需 spike JS `invoke` 或薄 command。
- 用户决策：**100%–200%**、**20% 步进**、设置滑块 + **保留 ⌘/Ctrl ±**（应用层统一实现）、其余沿用 explore 结论。

## Goals / Non-Goals

**Goals：**

- 主窗口整页 UI 在 100%–200% 间以 20% 步进缩放，默认 100%。
- 设置抽屉滑块与 ⌘/Ctrl ±/0 读写 **同一缩放状态**，重启后恢复。
- 缩放作用于主窗口全部 WebView 内容（三栏、Drawer、Overlay、Markdown），与 theme 同级全局偏好。
- 遵循现有 `localStorage` + hook 模式，设置 UI 风格与 `SettingsDrawer` 一致。

**Non-Goals：**

- 缩放低于 100%（80% 等）。
- 非 20% 步进的连续缩放（如 125%、175%）。
- 按面板/组件单独缩放。
- Rust `config.toml` 或 SQLite 持久化。
- 隐藏 PDF 导出 `WebviewWindow` 的缩放联动（保持独立默认 100%）。
- 自动跟随系统 display scaling。

## Decisions

### D1：Tauri `setZoom` 作为唯一缩放机制

调用 WebView `setZoom(scaleFactor)`（`1.0` = 100%），整页等比放大，覆盖 Tailwind `px`、固定 `text-[11px]`、Drawer `w-80` 等，无需改组件字号。

**备选**：CSS `zoom` / root `font-size` — 拒绝，前者非标准且 dev/prod 不一致；后者无法覆盖 px 字面量。

### D2：缩放常量与 snap

```text
UI_SCALE_STORAGE_KEY = "doc-agent-ui-scale"
UI_SCALE_STEP        = 0.2
UI_SCALE_MIN         = 1.0   // 100%
UI_SCALE_MAX         = 2.0   // 200%
UI_SCALE_DEFAULT     = 1.0
```

合法档位：`1.0, 1.2, 1.4, 1.6, 1.8, 2.0`

```text
snapUiScale(value):
  round(value / STEP) * STEP
  clamp to [MIN, MAX]
  fix float (e.g. round to 1 decimal)
```

`parseUiScale(stored)`：非法或非有限数 → `DEFAULT`；超出范围 → clamp 后 snap。

### D3：`src/lib/uiScale.ts` + `useUiScale` hook

对齐 `theme.ts` / `useTheme.tsx`：

```text
readStoredUiScale() → number
writeStoredUiScale(scale) → void
applyUiScale(scale) → Promise<void>  // snap + setZoom + optional storage
```

`applyUiScale` 在 Tauri 环境调用 zoom API；纯 Vite dev（无 Tauri）可 no-op 或 CSS `zoom` fallback，避免 `npm run dev` 报错。

Provider 挂载于 `main.tsx`（与 `ThemeProvider` 并列），供 `SettingsDrawer` 与 `App` 消费。

### D4：设置抽屉 UI

在 `SettingsDrawer`「工作区布局」区块 **之上** 新增 `config-surface`：

```
界面缩放
[========●====]  140%
100%                              200%
⌘ + / ⌘ − 调整；⌘ 0 恢复 100%     （Windows 文案用 Ctrl）
```

- `<input type="range" min="1.0" max="2.0" step="0.2">`
- 右侧或下方显示整数百分比（`Math.round(scale * 100)` + `%`）
- `aria-label` / `aria-valuetext` 可访问性
- 可选一行 muted：「放大后可视区域变小，可适当拉宽窗口。」

### D5：全局快捷键（应用层，与设置同源）

在 `App.tsx` 现有 `keydown` 监听中 **追加**（不干扰 K/N/O）：

| 快捷键 | 行为 |
|--------|------|
| ⌘/Ctrl + `=` 或 `+` | `snap(current + 0.2)`，已达 MAX 则 no-op |
| ⌘/Ctrl + `-` | `snap(current - 0.2)`，已达 MIN 则 no-op |
| ⌘/Ctrl + `0` | 设为 `1.0` |

条件：`event.preventDefault()`；composer 内 IME composing（`keyCode 229`）时不处理；与现有弹层 Esc 处理不冲突。

**实现 `setZoom` 后** 更新 React state + `writeStoredUiScale`。

### D6：`zoomHotkeysEnabled: false`

在 `tauri.conf.json` → `app.windows[0]` 增加 `"zoomHotkeysEnabled": false`，防止 WebView polyfill 与应用层 **双重 zoom**。

对用户而言 ⌘/Ctrl ± **仍可用**（由 D5 提供），与设置滑块步进一致。

**备选**：保留 `zoomHotkeysEnabled: true` 且不拦截 — 拒绝，无 `getZoom` 无法持久化热键调整，且步进虽同为 20% 但与 `localStorage` 状态易分叉。

### D7：Tauri 权限与 zoom API 接入

`src-tauri/capabilities/default.json`：

```json
"core:webview:allow-set-webview-zoom"
```

实现 spike 顺序：

1. 尝试 `import { getCurrentWebview } from '@tauri-apps/api/webview'` + `.setZoom()`（若运行时存在）
2. 否则 `invoke('plugin:webview|set_webview_zoom', { label, value })`
3. 若仍不可用，新增最小 Rust command 调用 `app.get_webview_window("main")?.set_zoom()`

不在 MVP 引入新 Cargo 依赖。

### D8：启动恢复

`main.tsx` 在 React render 前（对齐 `applyTheme(readStoredTheme())`）：

```text
applyUiScale(readStoredUiScale())  // fire-and-forget Promise
```

确保首屏即为目标缩放，避免闪 100% 再跳变。

### D9：测试

- `uiScale.test.ts`：`snapUiScale`、`parseUiScale`、storage 读写、边界 1.0/2.0
- `keyboardShortcuts` 或新 `uiScaleShortcuts.test.ts`：zoom in/out/reset 识别
- `SettingsDrawer` 或 `useUiScale.test.tsx`：滑块变更调用 apply（mock setZoom）
- `App` 可选：mock 快捷键步进
- 无 Rust 变更时仍跑现有 `cargo test`；若加 command 则补 handler 测试

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 200% 下三栏挤压、minWidth 更早触顶 | muted 提示；不自动改 window 物理尺寸 |
| `@tauri-apps/api` 无 `setZoom` 类型 | spike invoke / Rust command；封装于 `applyUiScale` |
| 纯 Vite dev 无 Tauri | `applyUiScale` 检测环境后 no-op 或 CSS fallback |
| 与 ⌘= 浏览器默认行为 | Tauri WebView 内由应用 preventDefault |
| workspace-ui spec 字号「12px 基线」 | Requirement 明确缩放作用于整窗，视觉字号随 zoom 放大 |

## Migration Plan

- 纯增量；无 DB / 配置迁移。
- 无 `localStorage` 记录时默认 100%，与现网一致。
- 回滚：移除 hook、设置区块、快捷键与 capability；git revert。

## Open Questions

- （已决）范围 100%–200%、步进 20%、滑块 + ⌘/Ctrl ±/0、应用层统一热键、`zoomHotkeysEnabled: false`
- （实现 spike）`setZoom` 前端调用路径以仓库内 Tauri 2.11 运行时为准
