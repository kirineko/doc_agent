## Context

`doc_agent` 当前 Agent loop（`agent/loop_runner.rs`）每轮通过 `build_working_messages` 从 SQLite store 全量重建上下文，没有任何 token 预算管理：

- `provider/sse.rs` 只解析 `finish_reason`，忽略 API 回报的 `usage`；`AssistantTurn` 无 token 字段。
- `ModelId` 不含上下文上限概念（DeepSeek 1M、Kimi 256K 未编码）。
- 工具循环 `for _step in 0..MAX_TOOL_STEPS`（当前 32）内不断 `push` 工具结果，从不回查上下文大小。

两个参考实现（`reference/kimi-cli`、`reference/deepy`）已有成熟方案，且算法基本一致。本设计**还原它们的做法**并适配本项目「每轮从 DB 重建」的架构。

参考关键文件：
- `kimi-cli/src/kimi_cli/soul/compaction.py`（`should_auto_compact`、`SimpleCompaction`、token 估算）
- `kimi-cli/src/kimi_cli/soul/context.py`（`token_count_with_pending`）
- `kimi-cli/src/kimi_cli/soul/kimisoul.py`（循环内触发点）
- `kimi-cli/src/kimi_cli/prompts/compact.md`（结构化摘要 prompt）
- `deepy/src/deepy/llm/compaction.py`（`_expand_preserve_start_for_tool_group` tool-call 配对完整性、manual/auto reason）
- `deepy/src/deepy/llm/context.py`（tiktoken / 字符估算）

## Goals / Non-Goals

**Goals:**
- 发往 API 的上下文 token 数永不超过模型上限，避免 API 报错。
- 还原 kimi/deepy 的 `should_auto_compact`（比例 + 预留双触发）与三段式压缩（摘要旧消息 + 保留最近若干轮）。
- token 计数以 API 精确 usage 为准，对未上报部分用「字符/4」pending 估算补足。
- 压缩结果持久化并归档，摘要长期复用，不每轮重复摘要。
- 循环每一步开头触发，覆盖单 turn 内大工具输出累加场景。
- 提高 `MAX_TOOL_STEPS`。

**Non-Goals:**
- 不引入向量检索 / RAG 式上下文召回。
- 不做跨会话上下文共享。
- 不引入 tokenizer 依赖（如 tiktoken）做精确本地计数——本地一律用字符启发式（deepy 的 tiktoken 是可选 fallback，本项目 Rust 侧从简）。
- 不改动前端流式展示协议（仅可选地新增 context usage 状态展示，留作后续）。

## Decisions

### D1. 压缩触发算法：完整还原 `should_auto_compact`
新增 `agent/compaction.rs`，实现与 kimi/deepy 字面一致的判定：

```
should_auto_compact(token_count, max_context_size, trigger_ratio, reserved_context_size) =
       token_count >= max_context_size * trigger_ratio
    OR token_count + reserved_context_size >= max_context_size
```

默认 `trigger_ratio = 0.85`。`reserved_context_size` **按模型上限取值**而非固定 50K：固定 50K 对 Kimi 256K 偏小（仅 ~19.5%），且摘要请求本身也要占预算。决策：`reserved = max(50_000, max_context_size * 0.1)`，并允许配置覆盖。理由见 D6。

### D2. token 计数：API usage 为准 + pending 估算
对齐 kimi 的 `token_count_with_pending`：
- `token_count`：最近一次 API 回报的 `usage.total_tokens`（精确）。
- `pending_estimate`：自上次 usage 之后新增、尚未发给 API 的消息（主要是工具结果）的「字符数 / 4」估算。
- 判定与压缩均使用 `token_count + pending_estimate`。
- 每次 API 返回后：`token_count = usage.total`，`pending = 0`；之后每 push 一条工具结果/消息：`pending += estimate(msg)`。

字符估算：`chars / 4`（与两参考一致；对 CJK 偏低但作为临时值，下次 API 调用即被精确值替换）。

### D3. 压缩执行：三段式 + tool-call 配对完整性
`compact_context` 流程（还原 `SimpleCompaction` + deepy 的配对保护）：
1. 从尾部保留最近 `max_preserved_messages` 条 user/assistant（默认 2）。
2. **保留起点前移**：移植 deepy `_expand_preserve_start_for_tool_group`——若切分点紧邻 `function_call`/`tool` 消息，或保留段内的 `tool` 结果其对应 `tool_calls` 落在压缩段，则把起点前移，绝不拆散 `tool_calls`↔`tool` 配对（本项目用 `tool_call_id` 关联）。
3. 旧消息（to_compact）拼成单条输入，发一次**无工具**的 LLM 摘要请求，system 用移植版 `compact.md` 结构化 prompt（`<current_focus>`/`<code_state>`/`<active_issues>` 等区块）。
4. 摘要写为一条消息持久化；被压缩的旧消息标记 archived。
5. `after_tokens = summary_usage.completion + estimate(保留消息)`，写回会话 token 基线。

**摘要消息 role = `user`（已调研 kimi/deepy 确认）**：两个参考实现均把摘要作为 `role="user"` 消息喂回，而非 assistant：
- kimi `compaction.py`：`Message(role="user", content=[system("Previous context has been compacted. Here is the compaction output:"), ...摘要 parts])`。
- deepy `compact.py` `build_compact_summary_message`：`{"role": "user", "content": "Previous context has been compacted by Deepy. Continue from this summary:\n\n<摘要>"}`。

本项目据此采用 `role="user"`，内容以固定前缀「Previous context has been compacted. Continue from this summary:」+ 摘要正文。理由：作为输入上下文喂回语义最贴切，且规避连续 assistant 消息序列问题。

### D4. 持久化与归档：messages.archived 列 + 会话 token 基线
沿用现有迁移风格（`ALTER TABLE ... ADD COLUMN` + `let _ =` 幂等）：
- `messages` 增列 `archived INTEGER NOT NULL DEFAULT 0`；压缩时把 to_compact 段置 1。摘要消息以新 role（如 `assistant` 或专用标记）写入，`archived = 0`。
- 会话 token 基线持久化：`sessions` 增列 `last_token_count INTEGER`（最近 API usage），供重启后估算基线；或复用 settings 表。倾向加列。
- `build_working_messages` 改为只取 `archived = 0` 的消息重建 → 摘要 + 未归档消息自然成为新上下文。
- 不做物理删除，保留可追溯（对齐 deepy archive_and_replace 的「归档而非丢弃」思想；revert/checkpoint 暂不实现，列为 Non-Goal 的简化）。

### D5. 触发点：循环每一步开头
在 `continue_loop` 的 `for _step` 循环体最前面插入触发检查（对齐 kimi `kimisoul.py` 循环结构）：构造 `request` 前判定 `should_auto_compact(token_count + pending, model.max_context, ...)`，命中则先 `compact_context()` 再重建 `working_messages`。压缩失败：记录日志并**截断兜底**（丢弃最旧的非保留消息直到满足预留），避免彻底卡死——比 kimi 直接 raise 更稳健，理由见 R 区。

### D6. per-model 上限与 usage 采集
- `ModelId::max_context_size()`：DeepSeek* = 1_000_000，Kimi = 256_000，Mock = 100_000（便于测试小阈值）。
- 流式请求体加 `stream_options: { include_usage: true }`（OpenAI 兼容，DeepSeek/Kimi 均支持）。
- `sse.rs` 解析末尾含 `usage` 的 chunk（`prompt_tokens`/`completion_tokens`/`total_tokens`），填入 `AssistantTurn.usage`。Mock provider 返回估算 usage 以贯通测试。

### D7. MAX_TOOL_STEPS 提升
当前 32。文档生成类任务（澄清 → 多次 skill_read/skill_run → 校验）易触顶。决策：提升到 **64**，并抽为常量便于后续配置化。压缩已能防上下文撑爆，步数上限主要防失控循环，可安全放宽。

### D8. 前端上下文比例展示与压缩提示
**数据下发**：新增两个 `AgentEvent`：
- `context_usage`：`{ session_id, used_tokens, max_tokens, ratio }`。在每次 API usage 刷新后（以及压缩后）emit，携带 `ratio = used_tokens / max_tokens`（0~1）。
- `context_compacted`：`{ session_id, before_tokens, after_tokens }`。压缩成功后 emit。

**UI 比例展示**：在 `ChatPanel` 顶部「会话」标题栏右侧展示**极简**指示器——一个图标 + 比例值（如 `◔ 42%`），不展示 token 绝对值等冗余信息。`AgentStreamState` 增加 `contextRatio?: number` 字段，由 `context_usage` 事件更新；切换会话时重置。比例颜色可随阈值变化（接近上限时变橙/红），但 MVP 仅需图标 + 百分比。

**压缩提示**：收到 `context_compacted` 时，前端展示一次性、非阻断的轻提示（toast 或会话区一行系统提示），文案如「已自动压缩较早的对话历史以节省上下文」。MUST NOT 阻断输入或弹模态。

理由：kimi web 端有完整 context 用量可视化（`ai-elements/context.tsx`），本项目按用户要求只做「图标 + 比例值」的最小化展示，避免信息过载。

## Risks / Trade-offs

- **摘要请求自身超限** → 被压缩的旧消息可能本身就接近上限。Mitigation：保留尾部 + 摘要头部已大幅缩小输入；极端单条超大消息（如巨型工具结果）在拼接前按预留预算做硬截断（保留头尾、中间省略标记）。
- **字符/4 估算对 CJK 偏低，可能漏触发** → Mitigation：以 API 精确 usage 为主，pending 仅覆盖一轮内增量；`reserved` 留足缓冲（D1 按比例放大）；可后续把 CJK 系数调为更保守（如 /2）。
- **压缩丢失关键上下文** → Mitigation：结构化 prompt 强制保留「当前任务/报错与解法/最终代码/TODO」；保留最近若干轮原样；归档不物理删除，便于排查。
- **压缩失败中断 turn** → Mitigation：D5 截断兜底而非直接失败。
- **DB 迁移** → 仅新增列且带默认值，旧库幂等兼容，无破坏性。
- **额外 LLM 成本** → 压缩触发一次额外调用并消耗 token，属预期代价；仅在接近上限时发生。

## Migration Plan

1. `messages.archived`、`sessions.last_token_count` 通过幂等 `ALTER TABLE` 添加，旧库自动兼容（默认值保证旧数据 `archived=0`、基线 NULL→按 0 处理）。
2. 无 API 破坏；前端无需改动即可工作（context usage 展示作为可选增量）。
3. 回滚：移除触发点调用即恢复旧行为；新增列保留无副作用。

## Open Questions

- `reserved_context_size` 与 `compaction_trigger_ratio` 是否需要进 UI 配置，还是仅 settings/默认值？（倾向先默认值 + settings，UI 后续）

## Resolved Questions

- **摘要消息 role**：已调研确认 kimi 与 deepy 均用 `role="user"`，本项目据此采用 `user`（见 D3）。
- **压缩前端提示**：确认需要。新增 `context_compacted` 事件 + 一次性轻提示（见 D8）。
- **上下文比例展示**：确认需要。`ChatPanel` 标题栏右侧「图标 + 比例值」最小化展示，数据由新增 `context_usage` 事件下发（见 D8）。
