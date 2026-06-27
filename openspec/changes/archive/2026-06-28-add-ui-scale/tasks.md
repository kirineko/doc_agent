## 1. Tauri 配置与权限

- [x] 1.1 在 `src-tauri/capabilities/default.json` 添加 `core:webview:allow-set-webview-zoom`
- [x] 1.2 在 `src-tauri/tauri.conf.json` 主窗口设置 `zoomHotkeysEnabled: false`
- [x] 1.3 Spike：确认 `setZoom` 可用路径（`getCurrentWebview().setZoom` / `invoke` / 薄 Rust command），封装为单一 `applyWebviewZoom` helper

## 2. 缩放核心模块

- [x] 2.1 新增 `src/lib/uiScale.ts`：常量、`snapUiScale`、`parseUiScale`、`readStoredUiScale`、`writeStoredUiScale`、`applyUiScale`（含非 Tauri 环境 graceful no-op）
- [x] 2.2 新增 `src/lib/uiScale.test.ts`：snap、parse、边界、storage
- [x] 2.3 新增 `src/hooks/useUiScale.tsx`：Provider + hook，变更时 apply 并持久化
- [x] 2.4 新增 `src/hooks/useUiScale.test.tsx`：变更 scale 触发 apply（mock zoom API）
- [x] 2.5 在 `main.tsx` 启动时 `applyUiScale(readStoredUiScale())`（与 `applyTheme` 并列）

## 3. 设置抽屉 UI

- [x] 3.1 在 `SettingsDrawer` 增加界面缩放 `config-surface`：range 滑块（1.0–2.0 step 0.2）、百分比标签、快捷键提示、可选 muted 说明
- [x] 3.2 确保滑块 `aria-label` / `aria-valuetext` 可访问

## 4. 全局快捷键

- [x] 4.1 在 `src/lib/keyboardShortcuts.ts`（或 `uiScaleShortcuts.ts`）添加 zoom in/out/reset 快捷键识别（⌘/Ctrl + =/+、-、0；排除 composing）
- [x] 4.2 在 `App.tsx` 全局 `keydown` 中接入 zoom 快捷键，调用 `useUiScale` 步进/重置
- [x] 4.3 为快捷键识别添加单元测试

## 5. 验证

- [x] 5.1 `npm run typecheck && npm test && npm run build` 通过
- [x] 5.2 `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test` 通过（若有 Rust command 变更）
- [x] 5.3 手动验证：设置滑块 100%–200%、⌘/Ctrl ±/0、重启恢复、200% 下三栏仍可用
