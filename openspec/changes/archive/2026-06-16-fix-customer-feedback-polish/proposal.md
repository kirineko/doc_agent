## Why

客户反馈集中暴露若干 UI 细节与 Windows 更新残留问题：模型 Flyout 宽度未随侧栏自适应、斜杠/`@` 弹层字号偏小、文件管理器按钮误标 Finder、更新后临时安装包残留在系统 temp 目录、更新进度遮罩文案含省略号易被误解为截断。需在不大改架构的前提下快速修复体验与磁盘卫生。

## What Changes

- **模型 Flyout**：宽度与侧栏 trigger 对齐；侧栏拖拽 resize 时实时重算位置（`ResizeObserver`）
- **输入弹层字号**：`SlashMenuFlyout`、`SlashCommandPopup`、`FileMentionPopup` 统一抬升字号（11px → 12px 基线，主 label 可选 14px），三者视觉一致
- **文件管理器按钮**：`aria-label` / `title` 统一为「在文件管理器中打开项目根目录」（跨平台中性文案）
- **Updater 临时文件清理**：应用启动时在**后台线程**扫描系统 temp 目录，按 pattern 删除 **24 小时前**的 updater 临时文件/目录；失败静默、不阻塞 UI、快速退出
- **更新进度遮罩 UX**：去掉文案中的 `…`；版本号与百分比分行或分区展示；卡片最小宽度与居中排版，避免误读为截断

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `workspace-ui`：模型 Flyout 宽度自适应；`@`/斜杠弹层字号；更新进度遮罩文案与布局
- `project-file-browser`：打开项目根按钮无障碍文案改为「文件管理器」
- `app-updater`：启动时清理 stale updater 临时产物（24h、pattern、非阻塞）

## Impact

- **前端**：`useAnchorPosition.ts`、`ModelFlyout.tsx`（可选）、`SlashMenuFlyout.tsx`、`SlashCommandPopup.tsx`、`FileMentionPopup.tsx`、`UpdateProgressOverlay.tsx`
- **Rust**：新建 `core/updater_cleanup.rs`（或 `ipc/` 下模块），`lib.rs` setup 中 fire-and-forget 调用；单元测试覆盖 pattern 与 mtime 逻辑
- **Spec delta**：`workspace-ui`、`project-file-browser`、`app-updater`
- **依赖**：无新增 npm/Cargo crate
