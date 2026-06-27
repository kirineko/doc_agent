# 提案：界面缩放配置（add-ui-scale）

## Why

Doc Agent 当前界面固定为 100% 渲染，在高分辨率屏幕或长时间使用时，部分用户需要放大整体 UI 以提升可读性。应在设置中提供可预期的缩放档位，并与键盘快捷键统一，重启后保持用户选择。

## What Changes

- 引入主窗口 **界面缩放**：默认 **100%**，范围 **100%–200%**，步进 **20%**（100 / 120 / 140 / 160 / 180 / 200）。
- 在 **设置抽屉** 增加 `config-surface` 区块：带 snap 的 **滑块** + 当前百分比标签；附带 ⌘/Ctrl ± 快捷键提示。
- 使用 Tauri WebView **`setZoom`** 整页等比缩放（文字、布局、弹层、Markdown 一并放大）。
- 缩放偏好写入前端 **`localStorage`**（`doc-agent-ui-scale`），应用启动时恢复。
- 在 **`App.tsx` 全局快捷键** 中处理 ⌘/Ctrl `+`/`=`（放大）、`-`（缩小）、`0`（重置 100%），与设置 UI 共用同一 snap 逻辑并持久化。
- **`tauri.conf.json`** 为主窗口设置 `zoomHotkeysEnabled: false`，避免 WebView 内置热键与应用层双重缩放；**不排除** ⌘/Ctrl ± 功能（由应用实现）。
- **Tauri capability** 增加 `core:webview:allow-set-webview-zoom`。
- **排除**：低于 100% 的缩小、任意连续百分比（非 20% 步进）、按面板单独缩放、Rust/SQLite 持久化、系统 DPI 自动检测。

## Capabilities

### New Capabilities

- `app-ui-scale`: WebView 缩放因子、20% 步进与范围约束、`localStorage` 持久化、启动恢复、全局 ⌘/Ctrl ±/0 快捷键契约。

### Modified Capabilities

- `workspace-ui`: 设置抽屉内界面缩放配置区块及与版本/布局区块并列的展示要求。

## Impact

- **前端**：新增 `src/lib/uiScale.ts`、`src/hooks/useUiScale.tsx`（或等价 hook）；`SettingsDrawer` 滑块区块；`App.tsx` 全局 zoom 快捷键；`main.tsx` 启动时 apply；Vitest 单测。
- **Tauri**：`capabilities/default.json` 权限；`tauri.conf.json` 主窗口 `zoomHotkeysEnabled: false`；若 `@tauri-apps/api` 无 `setZoom` 类型/导出，使用 `invoke('plugin:webview|set_webview_zoom', …)` 或薄 Rust command 包装 `Webview::set_zoom`（实现阶段 spike）。
- **Rust 业务逻辑**：无 IPC 契约变更（除非 spike 决定加 `set_ui_scale` command）。
- **依赖**：无新增 npm/Cargo 依赖（优先使用现有 Tauri 2 API）。
- **风险**：200% 下有效视口变小，三栏布局更挤；需 muted 提示用户可适当拉宽窗口。
