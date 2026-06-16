## 1. 模型 Flyout 宽度自适应

- [x] 1.1 `useAnchorPosition`：宽度改为 `Math.max(rect.width, 240)`，移除 320px 上限
- [x] 1.2 `useAnchorPosition`：对 trigger 挂载 `ResizeObserver`，与 window resize 共用 `update()`
- [x] 1.3 手动验证：拖宽/拖窄侧栏时 Flyout 宽度与 trigger 对齐

## 2. 输入弹层字号统一

- [x] 2.1 `SlashMenuFlyout.tsx`：`text-[11px]` → `text-xs`（分类 tab、id、description）
- [x] 2.2 `SlashCommandPopup.tsx`：同上，保持与 Menu 一致
- [x] 2.3 `FileMentionPopup.tsx`：同上（面包屑、提示行、分组标题、文件行）
- [x] 2.4 目视对比 `@` / `/` fuzzy / `/` 图形菜单三处字号一致

## 3. 文件管理器按钮文案

- [x] 3.1 `ProjectFileExplorer.tsx`：`aria-label` / `title` 改为「在文件管理器中打开项目根目录」

## 4. Updater 临时文件启动清理（Rust）

- [x] 4.1 新建 `src-tauri/src/core/updater_cleanup.rs`：`is_stale_updater_entry`、`cleanup_stale_updater_artifacts`、`spawn_stale_cleanup`
- [x] 4.2 pattern：`DocAgent-*-updater*` 目录 + 含 `-updater` 的 `.exe`/`.msi`；mtime < 24h 跳过；顶层最多 512 条目
- [x] 4.3 `lib.rs` setup 末尾调用 `spawn_stale_cleanup()`（detached thread，不 join）
- [x] 4.4 单元测试：pattern 匹配、mtime 阈值、24h 内外删/留行为（tempdir）

## 5. 更新进度遮罩 UX

- [x] 5.1 `UpdateProgressOverlay.tsx`：去掉所有 `…`；主/副文案分行（正在下载 | v{version} | {n}%）
- [x] 5.2 卡片加 `min-w-[17rem] text-center`；百分比 `tabular-nums`
- [x] 5.3 安装阶段：「正在安装」+「即将重启」两行
- [x] 5.4 更新 `updateProgress.test.ts` 或 overlay 相关测试（若有文案断言）

## 6. 验证

- [x] 6.1 `npm run typecheck && npm test && npm run build`
- [x] 6.2 `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`
