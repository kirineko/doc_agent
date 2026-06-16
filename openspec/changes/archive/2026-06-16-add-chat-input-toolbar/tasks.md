## 1. Rust IPC

- [x] 1.1 新增 `import_project_file`：sandbox 写入项目根、100MB 上限、basename 校验、`overwrite` / `rename` / `skip` 冲突策略（rename 自动 `(n)` 递增）
- [x] 1.2 新增 `open_project_root`：`tauri_plugin_opener::open_path` 打开 sandbox 根目录
- [x] 1.3 在 `lib.rs` 注册命令；补充 sandbox/import 单元测试（越界路径、rename 递增、overwrite）

## 2. 前端逻辑

- [x] 2.1 新增 `src/lib/projectImport.ts`：`buildMentionInsert`、冲突 ask 三选一、逐文件 import 队列
- [x] 2.2 新增 `useProjectImport.ts`：串联 dialog、`invoke`、索引 merge、`fileRevision` bump、输入框插入
- [x] 2.3 新增 `insertSlashPrompt`（`src/lib/slash.ts` 或同级）：键盘 `/` 与按钮共用；Vitest 覆盖插入与占位符选中
- [x] 2.4 `projectImport.ts` / `slash` 单测

## 3. UI 组件

- [x] 3.1 新增 `ChatInputToolbar.tsx`：+ / 图片 / / 三按钮、disabled、tooltip、隐藏 file input
- [x] 3.2 新增 `SlashMenuFlyout.tsx`：二级分类菜单，数据来自 `SLASH_COMMANDS` + `CATEGORY_ORDER`
- [x] 3.3 重构 `ChatPanel.tsx`：composite 输入布局、集成 toolbar、refactor `pickSlash` → `insertSlashPrompt`、popup 锚点调整（目标 <400 行）
- [x] 3.4 `ProjectFileExplorer.tsx`：标题栏增加「在文件管理器中打开」按钮

## 4. 集成与验证

- [x] 4.1 `useWorkspace` 暴露 `importProjectFiles`、`addPastedImage` 已有；`App.tsx` 传 props
- [x] 4.2 更新 textarea placeholder
- [x] 4.3 `npm run typecheck && npm test && npm run build`；`cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [x] 4.4 手动冒烟：多文件导入 + 重名三选一 + 自动 `@`；图片按钮；二级 slash 菜单；Finder 打开根目录；clarify/busy 禁用
