## Why

当前 `doc_agent` 已具备 per-session running UI 与 `TurnRegistry` 取消能力，但后端仍以「同 project 最多 1 个 running turn」作为防竞态策略。这保证了安全，却过度串行：同一项目内两个会话分别处理不同文件时也会被拒绝。用户目标是允许更高吞吐：**全局最多 3 个 running turn**，可跨 project，也可来自同一个 project；只有当两个 running turn 会同时写同一文件、同一目录或同一系统工作区时，才拒绝后者。

放开 project 级互斥前必须先做文件治理。当前至少存在两个高风险共享工作区：

- `skill_run` 固定写 `.cache/skill-run/script.js` / `error.json`，turn 结束清理整个 `.cache/skill-run/`
- `ooxml_unpack` 常按 skill 指南写 `unpacked/`，且目标目录存在时直接删除再解包；docx/pptx/xlsx 任务极易撞名

此外 `fs_write`、`fs_patch`、`excel_write`、`ooxml_pack`、`docx_accept_changes`、`office_convert`、PDF 输出、Typst 输出、HTML 输出、`data_query`、`docx_extract_table` 与 `skill_run` runtime 动态写入均需要统一写锁，否则 `tool_result.changed_paths` 事后通知已经太晚，无法阻止并发损坏。

## What Changes

- **替换**现有同 project 单 turn 互斥：改为全局最多 3 个 running turn
- **新增** `ProjectRunLimiter`（或扩展 `TurnRegistry` 的全局计数层）：所有 project 总 running turn 数达到 3 时，新的 `send_message` / `resume_turn` 直接拒绝
- **新增** `FileLockRegistry`：按 project 内规范化资源加锁，支持 `read` 共享锁、`write` 独占锁、目录子树锁与系统工作区锁
- **新增** `ToolIoPlan`：工具执行前从 tool name + args 推导 read/write/workspace 资源，先申请锁，失败时返回「当前 {path} 已被会话 {title/id} 占用，请稍后重试」
- **新增** session/ooxml scratch workspace：系统临时目录由后端自动生成（8 位 hex 段名），不依赖模型命名
  - `.cache/skill-run/<session_key>/script.js`（同会话跨 turn 路径不变）
  - `.cache/ooxml/<session_key>/<work_key>/`（`work_key` 含 turn+source，无独立 turn 目录层；不嵌入文件名 stem）
  - `.cache/tmp/<session_key>/<turn_key>/...`（**预留**，helper 已定义，尚无调用方）
- **更新** `ooxml_unpack`：允许 `out_dir` 省略；省略时自动返回 `.cache/ooxml/<session_key>/<work_key>/`（`work_key` 含 turn+source）；显式 `out_dir` 仍允许但必须经写锁且不得删除其他 active turn 持有的目录
- **更新** docx/pptx/xlsx/runtime skills：示例禁止固定 `unpacked/` 与固定 `.cache/skill-run/script.js`；必须使用工具返回路径
- **更新** `skill_run` runtime op：`__doc_write` / `fs.writeFileSync` 动态写入前必须通过 lock guard 兜底申请写锁；写入路径记录仍用于 `changed_paths`
- **更新**前端运行态：保留 per-session running，但新增全局 3 并行限制提示与文件占用错误展示；后台 session 完成时刷新对应 project/session 元数据

## Capabilities

### New Capabilities

- `file-governance`：项目文件锁、工具 IO 规划、session/work 隔离工作区、全局并行限制

### Modified Capabilities

- `project-session`：同 project 单 turn 互斥改为全局 3 并行 + 文件写冲突互斥
- `agent-loop`：工具执行前申请 IO locks；turn 生命周期持有/释放锁；cancel/error/clarify 均必须释放
- `project-cache-layout`：`.cache/skill-run` 按 session 隔离；`.cache/ooxml` 按 session+work 隔离
- `ooxml-toolchain`：`ooxml_unpack` 自动生成隔离工作区，`ooxml_pack` 绑定解包目录和输出写锁
- `script-runtime`：`skill_run` 临时脚本恢复区按 session 隔离（同会话跨 turn 路径不变），runtime 写入动态锁兜底
- `workspace-ui`：显示全局并行上限与文件占用错误；后台完成事件同步

## Impact

### 后端文件

- `src-tauri/src/agent/turn_control.rs`：移除同 project active 拒绝逻辑，增加全局 running 限流或委托新 registry
- `src-tauri/src/core/file_locks.rs`（新）：`FileLockRegistry`、`LockRequest`、`LockGuard`、冲突文案
- `src-tauri/src/tools/io_plan.rs`（新）：静态工具 read/write/workspace 推导
- `src-tauri/src/tools/mod.rs`：`ToolContext` 增加 session/turn/project metadata 与可选 lock handle
- `src-tauri/src/agent/loop_tool_batch.rs`：`execute_one` 前计算 `ToolIoPlan` 并申请锁；`skill_run` 传入 runtime lock context
- `src-tauri/src/agent/loop_runner.rs`：turn start 获取全局 slot；turn complete/cancel/error/awaiting_user 释放 slot 与 locks
- `src-tauri/src/core/cache_paths.rs`：新增 `skill_run_*`、`ooxml_work_dir`、`turn_tmp_dir`（后者预留）
- `src-tauri/src/tools/skill_run_tmp.rs`：脚本/error 路径按 session 生成；turn 结束清理 session scratch
- `src-tauri/src/tools/skill.rs`：`skill_run` 响应返回 session-scoped `script_path`（同会话跨 turn 不变）
- `src-tauri/src/tools/runtime/ops.rs`：`__doc_write` 动态申请写锁
- `src-tauri/src/tools/ooxml/mod.rs`、`unpack.rs`：`out_dir` 可选；默认 `ooxml_work_dir`；禁止删除 active locked 目录
- `src-tauri/src/core/skills.rs` 与 `src-tauri/assets/skills/**`：更新 docx/pptx/runtime 示例路径

### 前端文件

- `src/lib/sessionRunState.ts`：保留 per-session map，支持全局 running count 派生
- `src/hooks/useWorkspace.ts`：后台 turn terminal event 同步 sessions/messages/files；文件占用错误 toast/banner
- `src/components/SessionList.tsx`：最多 3 个 running 会话的视觉状态保持
- `src/components/ChatPanel.tsx` / `ChatInputToolbar.tsx`：全局并行满额时发送阻断文案
- `src/types.ts`：新增错误 payload 可选字段（若后端选择结构化 error）

### 测试

- Rust：`file_locks` 单元测试、`io_plan` 单元测试、loop 并行集成测试、skill_run/ooxml scratch 隔离测试
- TS：全局 3 running 状态、后台 session terminal 刷新、文件占用错误展示

## Non-Goals

- 不实现等待队列；全局满额或文件冲突时直接拒绝后者
- 不做跨进程/跨应用实例文件锁；本变更只治理当前应用进程内 Agent turns
- 不做 OS 文件锁或 Office 外部编辑器占用检测
- 不保证模型永不选择相同用户输出文件名；同名输出由文件写锁拒绝，而不是自动改名
- 不迁移旧 `.cache/skill-run/script.js` 或项目根已有 `unpacked/`
