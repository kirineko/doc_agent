# 设计：流式消息与持久化展示对齐

## Context

**现状**

```
运行中                          turn_complete
────────                        ─────────────
messages: user + 历史           list_messages 全量替换
streaming: 单 indigo 框累加     streaming 清空
                                → 多条 slate assistant 框
```

- `loop_runner` 每步 `persist_assistant` 已写 DB，但前端 `messages` 仅在 `turn_complete` 更新。
- 工具 loop 多步之间 **不** 清空 `streamingReasoning/Content`，导致多轮思考/正文堆在同一流式框。
- 流式框（indigo）与持久框（slate）DOM/CSS 不同，结束时有跳变。

**约束**

- 不改变 tool 消息在中间区的隐藏策略。
- `turn_complete` 保留，用于 optimistic user 替换与最终对齐。
- React 组件软上限 150 行；ChatPanel 已接近上限，需拆子组件。

## Goals / Non-Goals

**Goals:**

- 每步 LLM 结束后，UI 立即展示与该步 DB 一致的 assistant 消息。
- 下一步 LLM 流式输出在**新**的预览框中展示，不与上一步合并。
- 流式预览与持久消息视觉结构一致（思考区 + 正文区）。
- `turn_complete` 后无明显布局重组。

**Non-Goals:**

- 移除 `list_messages` 最终刷新。
- 在中间区渲染 tool 消息。
- 改动 OpenAI SSE 解析或 Provider 层。

## Decisions

### 1. 新增 `assistant_step_done` 事件（推荐方案）

**选择**：每次 `persist_assistant` 成功后 emit，payload 含完整 `Message`（与 `list_messages` 单条结构一致）。

**理由**：比每步 `list_messages` IPC 更轻；比仅 `stream_flush` 更能保证 UI 与 DB 同步。

**备选**：每步 `list_messages` — 简单但 IPC 频繁；仅 `stream_flush` — 仍需另途获取 message 内容。

### 2. 前端处理顺序

```
assistant_step_done:
  1. 若 session_id !== active → 丢弃
  2. appendMessage(msg) — 按 id 去重
  3. streamingReasoning = ""; streamingContent = ""

turn_complete:
  1. streaming 清空（已有）
  2. list_messages 全量 setMessages（幂等）
```

**去重**：append 前检查 `messages.some(m => m.id === msg.id)`，避免 turn_complete 重复。

### 3. 统一 `MessageBubble` 组件

Props 示意：

```tsx
interface MessageBubbleProps {
  role: "user" | "assistant";
  content?: string | null;
  reasoning?: string | null;
  variant: "persisted" | "streaming";
  pending?: boolean;
}
```

- `streaming` variant：思考 summary 文案「思考中…」，外框 subtle 动效（可选 pulse border）。
- `persisted` variant：「思考过程」，与现样式对齐。
- 流式态：ChatPanel 在消息列表末尾渲染 `<MessageBubble variant="streaming" ... streaming* />`，不再单独 indigo 容器 duplicate 逻辑。

### 4. 多步工具 loop 行为

```
Step N LLM stream → streaming bubble 增量
Step N persist   → assistant_step_done → bubble 固化 + streaming 清空
Tool execute     → activity / ToolChainPanel（不变）
Step N+1 LLM     → 新 streaming bubble
Final answer     → assistant_step_done → turn_complete → list_messages
```

中间步 assistant 若 `content` 为空仅有 tool_calls，`isVisibleMessage` 规则需与现逻辑一致（有 reasoning 或 content 才显示）。

### 5. Mock Provider

Mock 路径同样 emit `assistant_step_done`，保证无 key 开发体验一致。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| `assistant_step_done` 与 `turn_complete` 竞态导致重复/闪动 | append 按 id 去重；turn_complete 全量 replace |
| 切换会话时迟到事件污染 | `session_id` 门控（复用 `isStaleSessionResult` 模式） |
| ChatPanel 超行数 | 抽出 `MessageBubble` + 可选 `MessageList` |
| 空 content 仅 tool_calls 的 assistant 步 UI 稀疏 | 保持现有 visibility 规则；必要时显示「正在调用工具…」占位（非 MVP） |

## Migration Plan

1. 后端先加事件 + loop_runner emit（前端忽略也不破坏）。
2. 前端接事件 + 清 streaming。
3. 抽 `MessageBubble` 替换双份 JSX。
4. 补 vitest / Rust 事件序列测试。
5. 手动冒烟：单轮无工具、单工具、多工具三步。

回滚：停止 emit 新事件即可恢复旧行为（前端兼容未知 event kind）。

## Open Questions

- 是否在 `assistant_step_done` 中携带 `tool_calls` 摘要供中间区展示？**MVP 否**，仍靠右侧工具链。
- streaming variant 是否保留 indigo 色调？**建议保留轻微差异**（让用户感知「仍在生成」），但布局与 persisted 一致。
