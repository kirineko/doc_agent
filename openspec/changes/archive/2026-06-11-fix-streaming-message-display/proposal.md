# 提案：修复流式消息与持久化展示不一致（fix-streaming-message-display）

## Why

当前会话区存在两套互不相通的渲染路径：运行中靠 `streamingReasoning/streamingContent` 在底部单独 indigo 流式框累加；回合结束后 `turn_complete` 清空流式状态并 `list_messages` 全量重载，改为按 DB 逐条 slate 消息框展示。多轮工具调用时，各步 LLM 输出会堆进同一个流式框，而 DB 已有多条 assistant 记录，导致运行中与结束后的布局、条数、样式明显不一致，用户感知为「闪一下重画」。

## What Changes

- **逐步固化事件**：每次 `persist_assistant` 后 emit `assistant_step_done`，携带已持久化的 assistant 消息；前端 append 到 `messages` 并清空 streaming 缓冲。
- **流式缓冲按步清空**：多轮工具 loop 的每一步 LLM 结束后清空 streaming，下一步在新流式框中增量展示，避免多步内容合并。
- **统一消息气泡组件**：抽取 `MessageBubble`（或等价组件），流式预览与持久消息共用结构与样式 variant，减少 turn 结束时的视觉跳变。
- **保留 turn_complete 全量对齐**：`turn_complete` 仍调用 `list_messages`，用于替换 optimistic user、最终幂等同步；预期此时 streaming 已空、变化最小。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-loop`：新增 assistant 逐步持久化后向前端推送的事件契约；明确多步 loop 间 streaming 生命周期。
- `workspace-ui`：明确中间区 assistant 消息的流式预览与持久展示一致性要求；禁止多步 LLM 输出合并为单一流式框。

## 纳入 / 排除

**纳入**：`assistant_step_done` 事件、前端 append + 清 streaming、统一气泡组件、相关单测。

**排除**：

- 去掉 `turn_complete` 时的 `list_messages`（保留作安全网）
- 在中间区展示 tool 角色消息（仍仅在右侧工具链）
- 跨会话 streaming 状态恢复
- 修改 Provider/SSE 解析逻辑（仅消费侧与事件编排变更）

## Impact

- **后端**：`agent/types.rs`（新 AgentEvent 变体）、`loop_runner.rs`（每步 persist 后 emit）、IPC 事件序列化。
- **前端**：`agentEvents.ts`、`App.tsx`、`ChatPanel.tsx`；新增 `MessageBubble.tsx`（或同类）；`agentEvents.test.ts` 增补。
- **风险**：①事件与 `list_messages` 顺序竞态需 session 门控；②append 与全量 reload 重复消息需按 id 去重或 replace；③组件拆分后 ChatPanel 行数需控制。
