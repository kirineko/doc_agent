## Context

Agent turn 由 `run_turn` / `resume_turn` 驱动 `continue_loop`（最多 64 步），期间通过 Tauri `agent-event` 推送流式 token 与工具状态。前端 `useWorkspace` 用**全局** `stream.busy` 与单一 streaming 缓冲；切换 `activeSessionId` 时 `dispatchStream({ type: "reset" })`，导致后台 turn 进度丢失。后端无 session/project 级 turn 锁，`send_message` 虽被 `sendingRef` 临时挡住双发，但切换会话后 UI 显示「不忙」。

clarify 已有 `turn_awaiting_user` + `resume_turn` 模式，可复用「提前 return + 事件收尾」思路，但 cancel 需补 tool results 且**不可** resume。

## Goals / Non-Goals

**Goals:**

- 用户可在 Chat 输入区 **Stop** 正在运行的 turn（cooperative cancel）
- 侧栏展示 **per-session running** 指示；切换会话可查看后台 turn 进度
- 同 **project** 同时最多 1 个 running turn，避免沙箱文件竞态
- Cancel 后消息序列合法（assistant tool_calls 均有 tool result），不 emit `turn_complete`，不触发 autotitle
- SSE 与 compaction 摘要流可被 cancel 中断

**Non-Goals:**

- skill_run / html_to_pdf 执行中的硬中断（最多等到 handler 返回或既有 timeout）
- 应用重启后恢复 in-flight turn（内存 TurnRegistry 即可）
- 跨 project 全局 turn 队列
- 将 clarify 的 `cancel_clarify` 合并进 `cancel_turn`

## Decisions

### D1：TurnRegistry + `CancellationToken`（内存，按 session 注册）

`AppState` 增加 `TurnRegistry`（`Arc<Mutex<HashMap<session_id, ActiveTurn>>>`），每条记录含 `turn_id`、`project_id`、`cancel: CancellationToken`。

- `run_turn` / `resume_turn` 入口：`register(session_id, project_id, turn_id)`；若同 session 已有 active → 返回错误「会话正在执行任务」
- 若同 **project** 已有**其他 session** active → 返回错误，文案含 running session 标题（查 store）
- loop 各 checkpoint 调用 `token.cancelled()` 或 `is_cancelled()`
- turn 正常结束 / cancel 完成 / 不可恢复错误：`unregister(session_id)`

**备选**：DB 持久化 running 标志 → 重启语义复杂，否决。

### D2：Cooperative cancel 检查点

在 `continue_loop` 每步开头、以及 `run_tool_batch` 每个工具**开始前**检查 cancel。

| 阶段 | 行为 |
|------|------|
| 步间 / 工具前 | 立即退出 cancel 路径 |
| `chat_stream` / compaction SSE | `tokio::select!` cancel vs `stream.next()`；drop response |
| 工具 handler 执行中 | **不**中断；cancel 标记后在 handler 返回后退出 |
| `skill_run` / `html_to_pdf` | 同上，最长等 30s 既有 timeout |

Cancel 路径顺序：

1. 对当前 assistant 轮**尚未有 result** 的 tool_calls 写入 `{ "cancelled": true }` result + tool message + `finish_tool_call(status=done)`
2. `cleanup_skill_run_tmp`（与 turn 结束规则一致：无 `error.json` 则清）
3. emit `turn_cancelled { session_id, turn_id }`
4. unregister；`run_turn` 返回 `Ok(())`

**不** emit `turn_complete`；**不**调用 `maybe_autotitle_session`。

**备选**：立即 kill skill_run 线程 → 不安全，否决。

### D3：`cancel_turn` IPC

```text
cancel_turn({ session_id }) -> Result<(), String>
```

- 无 active turn → 错误「当前没有进行中的任务」
- 有 active：`token.cancel()`；前端进入 `stopping` 态直至收到 `turn_cancelled` 或超时兜底（如 35s 后强制 idle + reload messages）

与 `cancel_clarify` 独立：clarify pending 时 turn 已 return，registry 无 entry；Stop 按钮在 `awaiting_clarify` 时隐藏。

### D4：同 project 单 turn 互斥

`send_message` 与 `resume_turn` 在 register 前查询 registry：若存在 `project_id` 相同且 `session_id` 不同的 active turn → 拒绝。

错误示例：「项目内有其他会话正在执行任务（{session_title}），请先停止或等待完成。」

同 session：若已在 running，拒绝新 `send_message`（与 clarify pending 校验并列）。

**备选 C（完全并行）** → 文件竞态，否决。

### D5：前端 per-session 运行态

新模块 `sessionRunState.ts`：

```text
SessionRunState = idle | running | stopping
Map<sessionId, { status, turnId, streaming*, liveTools, compactionNotice? }>
```

- `agent-event` 监听：**始终**更新 `runs.get(event.session_id)`，不再因非 active 丢弃
- `activeSessionId` 变更：**不** reset 其他 session 的 map entry；仅切换 ChatPanel 绑定的 derived state
- `turn_complete` / `turn_cancelled` / `turn_awaiting_user` → 该 session 置 `idle`（clarify 时 idle + activeClarify）
- Stop 点击：`stopping` + invoke `cancel_turn`

侧栏 `SessionList`：entry.status === running | stopping → 左侧 spinner（stopping 可减 opacity 或文案「停止中」）。

Chat：`running` 时输入 disabled + **停止**按钮；`stopping` 时 placeholder「正在停止当前任务…（等待工具结束，最多约 30 秒）」。

**备选** 保留全局 busy → 无法解决切换会话，否决。

### D6：事件与类型

新增 Rust/TS `AgentEvent::TurnCancelled` / `turn_cancelled`。

前端 `applyAgentEvent`：对 active session 清 streaming；对 map 中该 session 置 idle。

### D7：Mock Provider 测试钩子

Mock loop 增加「慢工具」场景（sleep 2s），集成测试：start → cancel → 收到 `turn_cancelled`、无 `turn_complete`、tool results 含 cancelled。

## Risks / Trade-offs

- [Cancel 期间 skill_run 仍占用最多 30s] → UI `stopping` 文案诚实说明；spec 明确
- [双 cancel 或 cancel 与 complete 竞态] → unregister 幂等；事件带 turn_id，前端忽略 stale
- [前端 map 与 DB 不一致] → `turn_cancelled` / `turn_complete` 后 `list_messages` 对齐（与现 turn_complete 一致）
- [project 互斥误伤「A  clarify 等待、B 想发消息」] → clarify 时 registry 无 entry，B 可发；仅 running 互斥
- [TurnRegistry 进程内丢失] → 重启后无 running，可接受

## Migration Plan

- 无 DB schema 变更
- 升级后行为：旧版若曾「后台 turn + 切换会话」，新版侧栏可见 running；无数据迁移
- 回滚：移除 IPC 与 registry，恢复全局 busy（不推荐）

## Open Questions

（无 — 探索阶段已拍板）
