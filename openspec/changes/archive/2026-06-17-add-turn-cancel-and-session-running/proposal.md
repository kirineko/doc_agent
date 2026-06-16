## Why

当前 Agent turn 一旦启动就无法由用户停止；前端用全局 `busy` 表示运行态，切换会话会 reset 流式状态，导致后台 turn 进度不可见，且用户可能在另一会话误以为可以发送。同一项目下多个会话还可能 accidental 并行写文件。需要「可停止的 turn」与「按会话/项目的运行态可视化」，消除切换会话时的 UX 与沙箱竞态混乱。

## What Changes

- **新增** `cancel_turn` IPC：用户主动停止当前 session 的 in-flight turn（cooperative cancel，步间检查 + SSE 中断）
- **新增** AgentEvent `turn_cancelled`（含 `session_id`、`turn_id`）；cancel 时补全未完成 tool 的 cancelled result，避免后续 API 400
- **新增** 后端 `TurnRegistry`（内存）：跟踪 active turn、`CancellationToken`；turn 结束 unregister
- **新增** 同 **project** 同时最多 **1** 个 running turn：`send_message` / `resume_turn` 若 project 内已有其他 session running → 拒绝并提示
- **新增** 前端 per-session 运行态 Map（streaming、liveTools、status：`running` | `stopping` | `idle`）；切换会话不再 reset 后台 session 的 running 状态
- **新增** 侧栏会话项 running 指示（spinner/dot）；非 active 的 running 会话可点击切换查看进度
- **新增** Chat 输入区 **停止** 按钮（`running` 时替换或并列发送；`stopping` 时 disabled 并显示「正在停止…」）
- **更新** SSE / compaction 摘要流：支持 cancel token 提前结束读取
- **更新** `send_message`：同 session running 时拒绝；`turn_awaiting_user`（clarify）不算 running

## Capabilities

### New Capabilities

（无 — 能力归入既有 agent-loop / workspace-ui / project-session）

### Modified Capabilities

- `agent-loop`：`cancel_turn`、cooperative cancellation、`turn_cancelled` 事件、cancel 时 tool result 补全与 cleanup 规则
- `workspace-ui`：per-session busy/running、Stop 按钮、侧栏 running 指示、切换会话保留后台流式状态
- `project-session`：同 project 单 turn 互斥；running session 标识供侧栏与 send 拦截

## Impact

- **后端**：`agent/turn_control.rs`（新）、`loop_runner.rs`、`provider/sse.rs`、`compaction.rs`、`ipc/mod.rs`、`agent/types.rs`、`state.rs`
- **前端**：`hooks/useWorkspace.ts`、`lib/agentEvents.ts`、`lib/sessionRunState.ts`（新）、`components/SessionList.tsx`、`ChatPanel.tsx` / `ChatInputToolbar.tsx`、`types.ts`
- **测试**：mock loop cancel 集成测试；前端 reducer 多 session + stop 流程单测
- **依赖**：无新 crate；可选使用 `tokio_util::sync::CancellationToken`（tokio 生态已有）
- **非目标**：skill_run 线程硬中断、应用重启后恢复未完成 turn、跨 project 全局队列
