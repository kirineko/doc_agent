## Why

聊天输入区已支持键盘 `@` 引用、`/` 斜杠模板与粘贴图片，但能力「藏在输入习惯里」，新用户难以发现；同时用户无法从 UI 将本地文件导入项目根目录并在对话中 `@` 引用。需要在输入区提供显式工具按钮，并补齐「导入 → 索引 → @」闭环；文件浏览区补充「在系统文件管理器中打开项目根」以便用户上传后自行整理子目录。

## What Changes

- 在 Chat 输入区（textarea 左下或 composite 输入框内）增加三个工具按钮：**+**（上传文件）、**图片**（选择图片作为消息附件）、**/**（图形化斜杠命令）
- **+**：Tauri 多选文件对话框 → 复制到**项目根目录**（`./`）；重名时 **覆盖 / 另存为（自动 `文件名 (n).ext` 递增）/ 取消**；成功后刷新 `@` 索引并在**当前光标处**插入 `@相对路径`（多文件按选择顺序空格分隔）
- **图片**：隐藏 file input 或 dialog，`accept` 图片 MIME；逻辑等同粘贴图片（写入 `.cache/attachments/`，不进项目目录、不出现在 `@` 列表）
- **/**：二级分类 flyout（general → word → ppt → excel → pdf → web → 命令列表）；选中后填入 registry `prompt`、选中首个 `{{占位符}}`、不发送；键盘 `/` 触发的 fuzzy 弹层保持不变
- **ProjectFileExplorer** 标题栏增加「在文件管理器中打开项目根」按钮（复用 `tauri_plugin_opener`）
- 新增 Rust IPC：`import_project_file`（或等价单文件命令，前端循环调用）、`open_project_root`

## Capabilities

### New Capabilities

- `project-file-import`：用户主动将本地文件导入项目根、冲突处理、索引刷新与自动 `@` 插入规则

### Modified Capabilities

- `workspace-ui`：Chat 输入工具栏、斜杠二级图形菜单、disabled 规则与 placeholder
- `project-file-browser`：在系统文件管理器中打开项目根目录
- `multimodal-input`：除粘贴外，支持工具按钮选择图片作为附件

## Impact

- **Rust**：`src-tauri/src/ipc/mod.rs`（`import_project_file`、`open_project_root`）、`lib.rs` 注册；sandbox 写入项目根；可选单测
- **前端**：`ChatInputToolbar.tsx`、`SlashMenuFlyout.tsx`、`useProjectImport.ts`；`ChatPanel.tsx` 拆分瘦身；`ProjectFileExplorer.tsx`；`useWorkspace.ts` / `useProjectFiles.ts` 联动
- **测试**：import 冲突命名、`insertSlashPrompt`、mention 拼接 Vitest；Rust import 路径/越界
- **依赖**：无新 npm/Cargo 依赖（沿用 `@tauri-apps/plugin-dialog`、`tauri_plugin_opener`）

## 纳入 / 排除

**纳入**

- 三按钮 + tooltip；多文件上传；任意扩展名；固定根目录；重名三选一（另存为自动递增）；自动 `@`；二级 slash 菜单；Explorer 打开根目录

**排除**

- 上传到 Explorer 当前浏览子目录（不共享 browsePath）
- 拖拽上传到聊天区
- 上传进度条、批量「全部覆盖」快捷选项
- 上传后自动发送消息
- 自定义斜杠命令 / 最近使用
- 图片按钮写入项目目录
