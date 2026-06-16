## 1. 后端 TurnRegistry 与类型

- [x] 1.1 新增 `agent/turn_control.rs`：`TurnRegistry`、`ActiveTurn`（session_id、turn_id、project_id、CancellationToken）、register / unregister / cancel / get_active_for_project
- [x] 1.2 `state.rs`：AppState 挂载 `TurnRegistry`；`AppState::new` 初始化
- [x] 1.3 `agent/types.rs` + `types.ts`：新增 `TurnCancelled` / `turn_cancelled` AgentEvent
- [x] 1.4 `turn_control` 单元测试：register 冲突、同 project 查询、unregister 幂等

## 2. Loop cancel 与互斥

- [x] 2.1 `loop_runner.rs`：`run_turn` / `resume_turn` 入口 register；出口（complete / awaiting_user / cancel / max steps）unregister；步间与 `run_tool_batch` 工具前检查 cancel
- [x] 2.2 实现 cancel 收尾：补 cancelled tool results + tool messages、`cleanup_skill_run_tmp`、emit `turn_cancelled`、跳过 autotitle
- [x] 2.3 `send_message` / `resume_turn` IPC：同 session running 拒发；同 project 其他 session running 拒发（错误含 session 标题）
- [x] 2.4 IPC `cancel_turn` command + `lib.rs` 注册
- [x] 2.5 Mock provider：可选「慢工具」场景供 cancel 集成测试

## 3. SSE 与压缩可取消

- [x] 3.1 `provider/sse.rs`：`consume_openai_sse` 接受 cancel token，`select!` 中断读取
- [x] 3.2 `provider/openai_compat.rs` / compaction 摘要调用链传入 cancel token
- [x] 3.3 测试：cancel 后不再追加 token（mock stream 或单元测试）

## 4. 前端 per-session 运行态

- [x] 4.1 新增 `lib/sessionRunState.ts`：Map 结构、reducer、`idle|running|stopping`、按 session 应用 agent-event
- [x] 4.2 重构 `useWorkspace`：移除切换会话时的全局 stream reset；active session 从 Map derive；`sendingRef` 改为 per-session 或依赖 running 表
- [x] 4.3 `agentEvents.ts`：支持 `turn_cancelled`；非 active session 事件写入 Map（通过新 reducer 层）
- [x] 4.4 `sessionRunState` / `agentEvents` 单测：多 session 并行事件、切换会话不丢 A 的 liveTools

## 5. UI：Stop 与侧栏

- [x] 5.1 `SessionList.tsx`：running / stopping 视觉指示
- [x] 5.2 `ChatPanel` / `ChatInputToolbar`：running 时 Stop 按钮、stopping 文案与 disabled；clarify 时不显示 Stop
- [x] 5.3 `turn_cancelled` 后 `list_messages` 对齐；stopping 超时兜底（~35s 强制 idle）
- [x] 5.4 组件测试：Stop 按钮可见性、侧栏 running 指示

## 6. 集成测试与验证

- [x] 6.1 `loop_runner_tests`：mock start → cancel → `turn_cancelled`、无 `turn_complete`、tool results 含 cancelled
- [x] 6.2 `loop_runner_tests`：A running 时 B send_message 被拒；A clarify pending 时 B 可 send
- [x] 6.3 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` + `npm run typecheck && npm test && npm run build`
- [x] 6.4 手动：A 长任务 running → 切 B 见侧栏 A 指示 → 切回 A 见进度 → Stop → stopping → cancelled；同 project B 发送被拒
