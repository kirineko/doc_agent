# 提案：浅色 / 深色主题切换（add-theme-toggle）

## Why

Doc Agent 当前 UI 为写死的深色配色，长时间使用或明亮环境下可读性不足；用户需要浅色模式以获得更舒适的阅读体验，同时保留现有深色作为默认风格。顶栏右上角提供显式切换，可避免依赖系统 `prefers-color-scheme` 且行为可预期。

## What Changes

- 引入**两档主题**：`dark`（默认，保持现有视觉）与 `light`（Notion 感：暖白底、柔和灰边、深色正文）。
- 在顶栏**右上角**增加 **toggle** 控件，点击即可在深 / 浅之间切换。
- 使用 **CSS 语义变量**（`data-theme` on `<html>`）统一驱动全局背景、面板、边框、文字与 Markdown 区域，替换散落的硬编码 hex / `slate-*` 默认值。
- 主题偏好通过前端 **`localStorage`** 持久化，应用重启后恢复用户上次选择。
- Markdown 代码高亮随主题切换（深色 `github-dark` / 浅色 `github`）；消息区 `prose` 变体随主题调整。
- **排除**：「跟随系统」第三档、自定义 accent 色、Rust/Tauri 侧持久化、主题切换动画、独立设置页。

## Capabilities

### New Capabilities

- `app-theme`: 主题 token 体系、深 / 浅两档语义、默认深色、`localStorage` 持久化与切换行为契约。

### Modified Capabilities

- `workspace-ui`: 顶栏右上角主题 toggle 控件及与品牌区（左侧 Logo + 标题）的布局要求。

## Impact

- **前端**：`index.css`（CSS 变量 + `@theme` 映射）、新增 `useTheme` hook 与 `ThemeToggle` 组件；`App.tsx` 顶栏布局；~13 个现有组件及 `.tool-card` / `.panel` / `.markdown-body` 样式迁移至语义 token；`MarkdownView` 高亮主题切换；`public/logo.svg` 适配浅色背景（`currentColor` 或等效）。
- **Rust / IPC**：无变更。
- **依赖**：无新增 npm / Cargo 依赖。
- **风险**：大面积 class 迁移可能漏改边角组件；浅色模式须逐区校验 WCAG 对比度。
