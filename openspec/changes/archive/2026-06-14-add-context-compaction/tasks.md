## 1. 模型上限与 token 用量类型

- [x] 1.1 `agent/types.rs`：为 `ModelId` 新增 `max_context_size()`（DeepSeek*=1_000_000、Kimi=256_000、Mock=100_000）
- [x] 1.2 `agent/types.rs`：新增 `TokenUsage { prompt: u32, completion: u32, total: u32 }`，并在 `AssistantTurn` 增加 `usage: Option<TokenUsage>`
- [x] 1.3 提高 `MAX_TOOL_STEPS` 至 64（抽为常量，附注释说明压缩已防上下文撑爆）

## 2. Provider：流式 usage 采集

- [x] 2.1 `provider/openai_compat.rs`：请求体增加 `stream_options: { include_usage: true }`
- [x] 2.2 `provider/sse.rs`：解析末尾含 `usage` 的 chunk（prompt/completion/total_tokens），填入 `AssistantTurn.usage`；补单测覆盖含 usage chunk 的解析
- [x] 2.3 `provider/mock.rs`：返回估算 `usage`（按消息字符数估算）以贯通测试
- [x] 2.4 `provider/{deepseek,kimi}.rs`：确认 usage 透传无回归

## 3. 存储：归档标记与 token 基线

- [x] 3.1 `core/store.rs` migrate：幂等 `ALTER TABLE messages ADD COLUMN archived INTEGER NOT NULL DEFAULT 0`
- [x] 3.2 `core/store.rs` migrate：幂等 `ALTER TABLE sessions ADD COLUMN last_token_count INTEGER`
- [x] 3.3 `Message` 结构与 `list_messages` 增加 `archived` 字段；新增 `list_active_messages`（仅 archived=0）
- [x] 3.4 新增 `mark_messages_archived(ids)`、`set_session_token_count` / `get_session_token_count`
- [x] 3.5 为新增 store 方法补 CRUD 与 archived 持久化测试

## 4. 压缩核心模块（还原 kimi/deepy）

- [x] 4.1 新增 `agent/compaction.rs`：`should_auto_compact(token_count, max_context_size, trigger_ratio, reserved_context_size)` 与默认参数（ratio=0.85、reserved=max(50_000, max_context*0.1)）
- [x] 4.2 `agent/compaction.rs`：`estimate_text_tokens`（字符数/4）与 pending 估算辅助
- [x] 4.3 `agent/compaction.rs`：切分保留点 + `expand_preserve_start_for_tool_group`（移植 deepy，按 tool_call_id 保证配对完整）
- [x] 4.4 新增 `prompts/compact`（移植 kimi `compact.md` 结构化 prompt 到项目 prompt 体系）
- [x] 4.5 `agent/compaction.rs`：`compact_context` —— 拼接压缩段、发起无工具 LLM 摘要请求、生成摘要消息（`role="user"` + 固定前缀，对齐 kimi/deepy）
- [x] 4.6 `agent/compaction.rs`：摘要段极端单条超大消息的硬截断保护（保留头尾 + 省略标记）
- [x] 4.7 `agent/compaction.rs`：压缩失败兜底（截断最旧非保留消息至满足预留）
- [x] 4.8 单测：should_auto_compact（DeepSeek 比例先触发 / Kimi 预留先触发 / 空上下文不触发）、配对完整性、无可压缩不调用 LLM

## 5. Loop 集成

- [x] 5.1 `loop_support.rs`：`build_working_messages` 改用 `list_active_messages`（仅未归档）
- [x] 5.2 `loop_runner.rs`：维护 `token_count`（API usage 刷新）与 `pending_estimate`（push 消息时累加）
- [x] 5.3 `loop_runner.rs`：在 `for _step` 循环开头插入触发判定，命中则 `compact_context` 并重建 working_messages
- [x] 5.4 `loop_runner.rs`：压缩成功后持久化摘要消息、标记旧消息归档、写回会话 token 基线
- [x] 5.5 Mock Provider 多步循环集成测试：构造超阈值上下文 → 验证触发压缩、归档、重建后请求不超限

## 6. 事件下发（后端）

- [x] 6.1 `agent/types.rs`：`AgentEvent` 新增 `context_usage { session_id, used_tokens, max_tokens, ratio }` 与 `context_compacted { session_id, before_tokens, after_tokens }`
- [x] 6.2 `loop_runner.rs`：每次 usage 刷新后 emit `context_usage`；压缩成功后 emit `context_compacted` 并再 emit 一次 `context_usage`

## 7. 前端：比例展示与压缩提示

- [x] 7.1 `src/types.ts`：`AgentEvent` 联合类型新增 `context_usage`、`context_compacted`（与 Rust 序列化对齐）
- [x] 7.2 `src/lib/agentEvents.ts`：`AgentStreamState` 增加 `contextRatio?`，处理 `context_usage`（更新比例）、会话切换/重置时清空；补单测
- [x] 7.3 新增上下文比例指示器组件（图标 + 百分比），接入 `ChatPanel` 标题栏右侧；高 ratio 变色
- [x] 7.4 `context_compacted` 一次性轻提示（toast 或会话区系统提示行），非阻断、非常驻
- [x] 7.5 组件/工具函数测试：比例展示与事件状态更新

## 8. 校验与收尾

- [x] 8.1 `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 全绿
- [x] 8.2 `npm run typecheck`、`npm test`、`npm run build` 全绿
- [x] 8.3 `openspec validate add-context-compaction --strict` 通过
