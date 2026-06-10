# 实施任务：修复流式消息与持久化展示不一致

实现与 `design.md` 冲突时先更新 artifact，再改代码。

## 1. 后端事件（specs/agent-loop）

- [x] 1.1 `AgentEvent` / 前端 `types.ts` 新增 `assistant_step_done`（含 `Message` payload）
- [x] 1.2 `loop_runner.rs`：每次 `persist_assistant` 成功后 emit；顺序在 tool 执行 / `turn_complete` 之前
- [x] 1.3 Mock Provider 路径同样 emit（若与 loop_runner 共用 persist 则验证即可）
- [x] 1.4 Rust 单测或 loop 集成测：多步 persist 时事件顺序正确

## 2. 前端流式状态机（specs/agent-loop、workspace-ui）

- [x] 2.1 `agentEvents.ts`：处理 `assistant_step_done` — append 逻辑由 App 层或 reducer 辅助函数承担；收到后清空 `streamingReasoning/Content`
- [x] 2.2 `App.tsx`：`assistant_step_done` 时按 message id 去重 append；session 门控
- [x] 2.3 保留 `turn_complete` → `list_messages`；确认与 append 幂等、无重复条
- [x] 2.4 `agentEvents.test.ts`：逐步事件 + 清 streaming；多 session 丢弃

## 3. 统一消息气泡（specs/workspace-ui）

- [x] 3.1 新增 `MessageBubble.tsx`：`persisted` / `streaming` variant，思考 `<details>` + `MarkdownView`
- [x] 3.2 `ChatPanel.tsx`：历史消息与流式预览均用 `MessageBubble`；删除重复 indigo 流式容器 JSX
- [x] 3.3 控制 `ChatPanel` 行数 ≤250（必要时抽 `MessageList` 子组件）

## 4. 验收

- [x] 4.1 Rust：`cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test` 全绿
- [x] 4.2 前端：`npm test` + `npm run typecheck` + `npm run build` 全绿
- [x] 4.3 手动冒烟：单轮无工具、单工具一步、多工具三步 — 运行中与结束后条数/布局一致，无多步合并流式框
