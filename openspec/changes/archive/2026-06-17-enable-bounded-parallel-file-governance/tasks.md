## 1. 文件治理基础设施

- [x] 1.1 新增 `src-tauri/src/core/file_locks.rs`：实现 `LockMode`、`FileResource`、`LockRequest`、`FileLockRegistry`、`FileLockGuard`、冲突检测与结构化错误
- [x] 1.2 `src-tauri/src/core/mod.rs` 导出 `file_locks`
- [x] 1.3 `src-tauri/src/state.rs`：`AppState` 增加 `Arc<FileLockRegistry>` 与全局 `RunLimiter`
- [x] 1.4 `src-tauri/src/core/file_locks.rs` 单测：read/read、read/write、write/write、subtree ancestor/descendant、跨 project、acquire_many all-or-none、冲突文案

## 2. 全局 3 并行与 TurnRegistry 调整

- [x] 2.1 `src-tauri/src/agent/turn_control.rs`：移除同 project active turn 拒绝逻辑；保留同 session active/reserved 拒绝与 cancel
- [x] 2.2 新增或同文件实现 `RunLimiter`：全局最多 3 个 running turns，RAII `RunSlotGuard` 释放
- [x] 2.3 `src-tauri/src/agent/loop_runner.rs`：`send_message` / `resume_turn` 在写 user message 或提交 clarify answer 前申请 global slot
- [x] 2.4 `loop_runner.rs`：所有 terminal path（complete、cancelled、awaiting_user、max steps、provider error、tool-loop error）释放 slot 与 turn registry；`cancel_turn` IPC **不** release slot（stopping 期间仍占名额，见 design D1/D9）
- [x] 2.5 更新 Rust 测试：同 project 第二 session 可启动；第 4 个全局 running 被拒且不写 user message；clarify awaiting 不占 global slot；cancel 已触发但 loop 未退出时仍占 slot

## 3. Tool IO Plan 与工具执行前锁

- [x] 3.1 新增 `src-tauri/src/tools/io_plan.rs`：实现 `ToolIoPlan`、路径规范化、tool matrix（`pdf_read` / `pdf_render_pages` 仅源 `path` Read；`.cache/pdf/**` 不加 file lock，见 design D10）
- [x] 3.2 `src-tauri/src/tools/mod.rs`：`ToolContext` 增加 `project_id`、`session_id`、`turn_id`、`session_title`、`file_locks`
- [x] 3.3 `src-tauri/src/agent/loop_tool_batch.rs`：`execute_one` 在工具执行前计算 IO plan 并申请 locks；锁冲突转为 tool result
- [x] 3.4 `src-tauri/src/tools/changed_paths.rs`：保持 UI 刷新职责，修正 `ooxml_unpack` arg missing 时从 result 读取 `out_dir`
- [x] 3.5 `io_plan` 单测覆盖所有注册工具；新增测试确保未分类 filesystem tool 会失败提示补 plan

## 4. `.cache` 路径治理

- [x] 4.1 `src-tauri/src/core/cache_paths.rs`：新增 `.cache/ooxml`、session-scoped `skill_run_*`、`ooxml_work_dir`；`turn_tmp_dir` 预留（尚无调用方）
- [x] 4.2 `cache_paths` 单测：safe id、POSIX 分隔符、禁止 `..`、同 source 不同 turn 目录不同
- [x] 4.3 `src-tauri/src/tools/skill_run_tmp.rs`：固定 `.cache/skill-run/script.js` 改为 `.cache/skill-run/<session_key>/script.js`（同会话跨 turn 共用）
- [x] 4.4 `loop_runner.rs` / `loop_support.rs`：turn 结束清理该 session 的 skill-run scratch 目录（无 `error.json` 时删整目录），不删除其他 session 的目录
- [x] 4.5 更新 project-cache-layout spec 对应实现：`.cache/ooxml` 隐藏且不进入 `@` 候选

## 5. `skill_run` 动态写锁

- [x] 5.1 `src-tauri/src/tools/runtime/mod.rs`：`execute_script` 接受 runtime write gate / lock set
- [x] 5.2 `src-tauri/src/tools/runtime/ops.rs`：`__doc_write` / `fs.writeFileSync` 写入前动态申请 Write lock
- [x] 5.3 `src-tauri/src/tools/skill.rs`：`skill_run` inline code 保存与 error 返回使用 session-scoped `script_path`
- [x] 5.4 `src-tauri/src/tools/runtime/diagnostics.rs`：错误中的 `script_path` 读取上下文路径，禁止写死 `.cache/skill-run/script.js`
- [x] 5.5 测试：两个 session 同时 `skill_run` inline 生成不同脚本路径；动态写同一输出文件时后者 file_busy；同脚本重复写同一路径可复用锁

## 6. OOXML 工作区隔离

- [x] 6.1 `src-tauri/src/tools/ooxml/mod.rs`：`ooxml_unpack` schema 中 `out_dir` 改为可选；未传时生成 `.cache/ooxml/<session_key>/<work_key>/`
- [x] 6.2 `src-tauri/src/tools/ooxml/unpack.rs`：删除/重建目录前要求调用方已持有 subtree write lock；错误文案保留目标目录
- [x] 6.3 `ooxml_unpack` 返回相对 `out_dir`；不再只返回绝对 display path
- [x] 6.4 `docx_comment` / `ooxml_pack` / XML 编辑链按返回目录继续工作
- [x] 6.5 测试：两个 session 同 project 分别 unpack 不同 docx/pptx 自动目录不冲突；两个 explicit `out_dir: "unpacked"` 后者 file_busy；后者失败不删除前者目录

## 7. 其他写文件工具补锁

- [x] 7.1 `office_convert`：默认 `*-converted` 输出进入 IO plan，已有输出仍按现行为错误
- [x] 7.2 `pdf_merge` / `pdf_split` / `pdf_rotate` / `pdf_delete_pages`：输入 read、输出 write/subtree locks
- [x] 7.3 `typst_to_pdf`：入口 read、最终 out_path write；内部 staging 保持 UUID 唯一
- [x] 7.4 `html_to_pdf`：输入 read、输出 write lock 覆盖 async WebView 导出
- [x] 7.5 `data_query` / `excel_normalize` / `docx_extract_table` / `xlsx_recalc`：补齐 read/write/subtree locks
- [x] 7.6 集成测试覆盖至少 `fs_write`、`excel_write`、`pdf_split burst`、`typst_to_pdf` 同名输出冲突

## 8. Skill 文档与 prompt 更新

- [x] 8.1 `src-tauri/assets/skills/docx/editing.md`：示例改为省略 `out_dir` 并使用返回的 `out_dir`
- [x] 8.2 `src-tauri/assets/skills/pptx/editing.md`：同上，所有 `unpacked/ppt/...` 改为 `<out_dir>/ppt/...`
- [x] 8.3 `src-tauri/assets/skills/runtime/SKILL.md`：故障修复流程改为使用工具返回的 `script_path`
- [x] 8.4 `src-tauri/assets/skills/docx/SKILL.md` / `pptx/SKILL.md` / `xlsx/SKILL.md`：删除固定 `.cache/skill-run/script.js` 与 `unpacked/` 指令
- [x] 8.5 `src-tauri/src/agent/loop_support.rs` system prompt：提醒并行下不要自造共享临时目录，优先使用工具返回路径
- [x] 8.6 OpenSpec delta（`project-cache-layout` / `ooxml-toolchain` / `script-runtime` / `design.md`）：同步 8 位 hex 段名与 ooxml 两层 hash 布局
- [x] 8.7 系统 prompt / 工具 description / tool_args hint：去除固定 `.cache/skill-run/script.js`，统一强调工具返回的 `script_path` / `out_dir`
- [x] 8.8 skill-run 改为单层 `.cache/skill-run/<session_key>/`（同会话跨 turn 共用；turn 结束无 error 则删整目录）

## 9. 前端并行状态与错误展示

- [x] 9.1 `src/lib/sessionRunState.ts`：新增 running count selector；保持 per-session map
- [x] 9.2 `src/lib/sendReadiness.ts`：可选增加本地 `parallel_limit` blocker（后端权威）
- [x] 9.3 `src/hooks/useWorkspace.ts`：send 被全局满额拒绝时保留输入；显示明确提示
- [x] 9.4 `useWorkspace.ts`：非 active session 收到 terminal event 时刷新 session list / project file list，但不覆盖 active messages
- [x] 9.5 `ChatPanel` / `ChatInputToolbar`：全局 3 running 时发送按钮 disabled 或显示提示
- [x] 9.6 TS/组件测试：3 running blocker、后台 terminal 刷新（file_busy UI 展示待后续补测）

## 10. OpenSpec 与验证

- [x] 10.1 `openspec validate enable-bounded-parallel-file-governance --strict`
- [x] 10.2 Rust：`cargo fmt --check`
- [x] 10.3 Rust：`cargo clippy -- -D warnings`
- [x] 10.4 Rust：`cargo test`
- [x] 10.5 Frontend：`npm run typecheck`
- [x] 10.6 Frontend：`npm test`
- [x] 10.7 Frontend：`npm run build`
- [x] 10.8 手动验证：3 个并行任务可跑；第 4 个被拒；同 project 不同文件可跑；同文件写冲突拒绝后者；skill_run/ooxml 临时目录不重名
