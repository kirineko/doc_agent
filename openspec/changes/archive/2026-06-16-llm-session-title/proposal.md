## Why

当前会话自动标题依赖 `session_title.rs` 中大量启发式规则（泛化开场检测、任务动词、助手兜底、18 字硬截断），逻辑复杂且标题质量不稳定；侧栏虽已支持可变宽度，后端固定字符截断与 CSS 截断叠加，展示效果仍不理想。产品已确认改为「首轮用用户首条消息、第二轮 LLM 总结前两轮（仅一次）」的更简单策略。

## What Changes

- 移除启发式 `summarize_session_title` 及关联规则，改为两轮自动标题策略
- **第 1 轮 turn 结束**：若标题仍为默认，写入清洗后的首条 user 消息；超过存储上限则截断入库
- **第 2 轮 turn 结束**：使用当前会话模型、**非思考模式**异步调用 LLM，总结前两轮对话生成标题，**覆盖**第 1 轮标题；全生命周期 **仅触发 1 次**
- **第 3 轮及以后**：不再自动改标题；已有 ≥2 条 user 消息的历史会话在后续使用中亦不会补跑 LLM
- 数据库新增 `autotitle_llm_done`（及可选 `title_user_edited`）标记，避免用「仍为默认标题」误判第 2 轮
- 前端侧栏标题改为存储完整标题 + CSS 动态截断（`truncate`）与 `title` tooltip；移除对 18 字后端截断的依赖
- 新增 `session_title_updated` 事件（或等价机制），供 LLM 异步完成后刷新侧栏

## Capabilities

### New Capabilities

（无独立新 capability）

### Modified Capabilities

- `project-session`：重写会话自动标题需求（两轮策略、LLM 单次、存储上限、状态字段）
- `agent-loop`：更新 turn 完成时的 autotitle 触发条件与 resume 语义
- `workspace-ui`：侧栏会话标题展示（动态宽度截断）

## Impact

- **Rust**：`session_title.rs` 大幅瘦身；新增 `title_gen.rs`（仿 `suggest.rs`）；`loop_support.rs` / `loop_runner.rs`；`store` schema migration；`AgentEvent` 扩展
- **前端**：`SessionList.tsx`、`formatTitle.ts`；`useWorkspace` 事件处理；`types.ts` 契约
- **Spec**：`project-session`、`agent-loop`、`workspace-ui` delta
- **迁移**：存量 `user_message_count >= 2` 的会话可设 `autotitle_llm_done = 1`，防止历史数据误触发
