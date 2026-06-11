## Why

Agent 或用户在项目内新建/修改文件后，右侧文件浏览与输入框 `@` 引用仍显示旧快照（仅在切换项目时加载一次），导致无法发现新文件。MVP 设计已标注「工具写文件后可选刷新」但未实现。同时 `ooxml_unpack` 会在工作目录下产生大量 XML 部件，若全量索引会撑爆 `@` 候选并逼近 2000 项上限，需一并治理。

## What Changes

- **事件驱动文件索引同步**：Agent 写文件成功后通知前端；`turn_complete` 时 debounce 全量刷新 `@` 清单；资源管理器按 revision 刷新当前目录（单层 `readdir`，低成本）
- **`ToolResult` 携带 `changed_paths`**：后端在文件变更类工具执行成功后 emit 相对路径列表；`skill_run` 经 `__doc_write` 追踪写入路径
- **忽略 OOXML 解压工作目录**：`list_project_files`（及 `@` 索引）MUST 跳过名为 `unpacked` 或以 `_unpacked` 结尾的目录及其全部子树；资源管理器仍可在根目录看到该文件夹并手动进入
- **手动刷新**：文件浏览区标题旁提供刷新按钮，仅重载当前目录
- **不引入** fs watcher 轮询、定时 poll、每工具一次全量 walkdir

## Capabilities

### New Capabilities

（无 — 行为增量合并入现有能力 spec）

### Modified Capabilities

- `workspace-ui`：`@` 文件清单在项目文件变更后更新；索引忽略 OOXML 解压工作目录
- `project-file-browser`：Agent 写文件后当前目录自动刷新；支持手动刷新当前目录
- `agent-loop`：`tool_result` 事件可选携带 `changed_paths` 供前端增量同步

## Impact

- **Rust**：`core/project_files.rs`（忽略规则）、`agent/loop_runner.rs`（emit `changed_paths`）、`agent/types.rs`、`tools/runtime/ops.rs`（追踪 `__doc_write`）、`tools/mod.rs` 或 registry（路径提取 helper）
- **前端**：`hooks/useWorkspace.ts` 或新 `useProjectFiles.ts`、`ProjectFileExplorer.tsx`、`types.ts`（`ToolResult` 类型）
- **测试**：Rust 忽略规则单测；TS debounce/增量 merge 单测
- **依赖**：无新增 crate
