## 1. 后端：忽略规则与路径提取

- [x] 1.1 在 `core/project_files.rs` 实现 OOXML 解压目录忽略（段名 `unpacked` 或 `*_unpacked`），walkdir 不 descend
- [x] 1.2 为忽略规则添加 Rust 单测（含 `unpacked/`、`contract_unpacked/` 子树与同级 docx 仍可见）
- [x] 1.3 新增 `extract_changed_paths(tool_name, args, result)` helper，覆盖主要写文件工具与 `ooxml_unpack` 的 `out_dir`
- [x] 1.4 `runtime/ops.rs` 线程内（thread_local）记录 `__doc_write` 写入路径；`skill_run` result JSON 携带 `written_paths`，提取时读取
- [x] 1.5 `AgentEvent::ToolResult` 增加 `changed_paths: Option<Vec<String>>`；`loop_runner` 成功时填充并 emit
- [x] 1.6 同步 `src/types.ts` 中 `AgentEvent` 的 `tool_result` 类型

## 2. 前端：文件索引 hook 与事件同步

- [x] 2.1 抽取 `useProjectFiles(projectId)`（或等效模块）：`filePaths`、`fileRevision`、`refreshAll`、`onAgentEvent`
- [x] 2.2 实现 `mergePaths(prev, changed_paths)` 去重合并，并过滤忽略目录前缀
- [x] 2.3 在 `tool_result`（含 `changed_paths`）时增量 merge 并 bump `fileRevision`（explorer reload 当前目录）
- [x] 2.4 在 `turn_complete` 时 debounce（500ms）调用全量刷新；清单实际变化才 bump `fileRevision`；切换项目/unmount 时 clear timer
- [x] 2.5 将 `useWorkspace` 中 `filePaths` / 选项目加载逻辑迁移至新 hook，保持 `selectProject` 初次全量加载
- [x] 2.6 为 `mergePaths` 与 debounce 调度添加 Vitest 单测

## 3. 前端：资源管理器刷新

- [x] 3.1 `ProjectFileExplorer` 接收 `fileRevision` prop，`revision` 变化时 reload 当前路径（不 reset 到根）
- [x] 3.2 标题行增加手动刷新按钮，调用当前路径 `list_project_dir_cmd`
- [x] 3.3 `App.tsx` / `RightPanel.tsx` 贯通 `fileRevision` 与 `projectId`

## 4. 验证

- [x] 4.1 `cargo test`（project_files 忽略 + changed_paths 如有单测）
- [x] 4.2 `npm test` + `npm run typecheck`
- [x] 4.3 手动：Agent `fs_write` 后 `@` 与新文件可见；`ooxml_unpack` 后 `@` 无内部 XML、根目录见 `unpacked/` 文件夹
