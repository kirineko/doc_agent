## Why

当前 Agent loop 每轮从 DB 全量重建上下文，且**完全没有上下文长度管理**：既不解析 API 回报的 token 用量，也不知道各模型的上下文上限（DeepSeek 1M、Kimi 256K）。长会话或单轮内连续大工具输出（读大 Excel、解压 OOXML）会让发往 API 的上下文超过模型上限，导致 API 直接返回错误、turn 失败。需要引入自动压缩与 token 预算管理，对齐成熟实现（reference/kimi-cli、reference/deepy）。

## What Changes

- 新增**自动上下文压缩**能力：当预估 token 数接近模型上限时，对较早的历史消息做一次 LLM 结构化摘要，保留最近若干轮原样，用「摘要 + 保留消息」替换上下文。算法还原 kimi/deepy 的 `should_auto_compact`（比例触发 `ratio` 或预留触发 `reserved`，谁先到谁触发）。
- 新增**token 用量采集**：流式请求带 `stream_options.include_usage`，`sse.rs` 解析末尾 usage chunk，`AssistantTurn` 携带精确 token 用量；对「API 回报之后新增、尚未发出的工具结果」用「字符/4」做 pending 估算补足（对齐 kimi 的 `token_count_with_pending`，防大工具输出撑爆）。
- 新增**per-model 上下文上限**：`ModelId` 暴露 `max_context_size()`（DeepSeek 1_000_000、Kimi 256_000、Mock 小值便于测试）。
- **压缩结果持久化**：压缩时写入一条 summary 消息并将被压缩的旧消息标记为 archived；后续 `build_working_messages` 从「summary + 未归档消息」重建，摘要长期复用。
- 在 loop **每一步开头**检查并触发压缩（而非仅 turn 开始），覆盖单 turn 内工具结果累加撑爆的风险。
- 切分保留点采用 deepy 的 tool-call group 完整性逻辑，绝不拆散 `tool_calls`↔`tool` 配对。
- 摘要消息以 `role="user"` 写回（经调研 kimi/deepy 确认两者均用 user role）。
- 提高单 turn 工具调用步数上限 `MAX_TOOL_STEPS`（从 32 提升）。
- **前端**：新增 `context_usage` 与 `context_compacted` 事件；在会话区标题栏以「图标 + 比例值」最小化展示上下文占用比例；压缩时给出一次性轻提示。

## Capabilities

### New Capabilities
- `context-compaction`: 会话上下文的 token 预算管理与自动压缩——token 用量采集与 pending 估算、per-model 上限、压缩触发判定、LLM 结构化摘要、压缩结果持久化与归档、压缩失败兜底。

### Modified Capabilities
- `agent-loop`: 在工具循环每一步开头注入压缩触发点；提高 `MAX_TOOL_STEPS` 上限；`build_working_messages` 改为基于 summary + 未归档消息重建。
- `model-config`: 各模型暴露 `max_context_size()`；`AssistantTurn` 与 provider 流式响应携带 token 用量。
- `workspace-ui`: 会话区展示上下文占用比例（图标 + 比例值）；新增 `context_usage`/`context_compacted` 事件契约；压缩时一次性轻提示。

## Impact

- 后端代码：`src-tauri/src/agent/loop_runner.rs`、`loop_support.rs`、`types.rs`、`provider/{sse.rs, openai_compat.rs, mock.rs}`；新增 `agent/compaction.rs`（压缩与 token 估算）；`core/store`（消息 archived 标记 / 会话级 token 基线持久化 + 迁移）。
- 前端代码：`src/types.ts`（AgentEvent 扩展）、`src/lib/agentEvents.ts`（contextRatio 状态）、`src/components/ChatPanel.tsx`（比例指示器）、新增上下文指示器/压缩提示组件。
- Prompt：新增移植自 kimi `prompts/compact.md` 的结构化压缩 prompt。
- 行为：长会话不再因超限报错；压缩会额外消耗一次 LLM 调用与 token；DB schema 变更需迁移。
- 配置：新增可调参数（`compaction_trigger_ratio`、`reserved_context_size`、`max_preserved_messages`）。
