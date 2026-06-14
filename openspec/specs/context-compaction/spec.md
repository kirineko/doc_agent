# context-compaction Specification

## Purpose
TBD - created by archiving change add-context-compaction. Update Purpose after archive.
## Requirements
### Requirement: 压缩触发判定

系统 SHALL 提供统一的压缩触发判定函数，依据当前 token 数与模型上下文上限决定是否压缩。命中以下任一条件即触发（谁先满足谁触发）：

1. 比例触发：`token_count >= max_context_size * trigger_ratio`
2. 预留触发：`token_count + reserved_context_size >= max_context_size`

其中 `trigger_ratio` 默认 0.85；`reserved_context_size` 默认按模型上限取 `max(50_000, max_context_size * 0.1)`。`token_count <= 0` 或 `max_context_size <= 0` 时 MUST NOT 触发。

#### Scenario: DeepSeek 1M 比例先触发

- **WHEN** 当前 token 为 850_000、模型上限 1_000_000、`trigger_ratio=0.85`、`reserved=100_000`
- **THEN** 判定触发压缩（比例条件 850K ≥ 850K 满足）

#### Scenario: Kimi 256K 预留先触发

- **WHEN** 当前 token 为 210_000、模型上限 256_000、`trigger_ratio=0.85`、`reserved=50_000`
- **THEN** 判定触发压缩（预留条件 210K + 50K ≥ 256K 满足，且早于比例阈值 217.6K）

#### Scenario: 空上下文不触发

- **WHEN** 当前 token 为 0
- **THEN** MUST NOT 触发压缩

### Requirement: token 用量采集与 pending 估算

系统 SHALL 以 API 回报的精确 token 用量为权威计数，并对「自上次用量回报后新增、尚未发送给 API 的消息」用字符启发式（字符数 / 4）做 pending 估算。压缩触发判定 MUST 使用 `token_count + pending_estimate`。

- 每次 API 流式响应返回后，`token_count` MUST 更新为本次 `usage.total_tokens`，`pending_estimate` MUST 归零。
- 循环内每追加一条工具结果或新消息，`pending_estimate` MUST 累加该消息的估算值。

#### Scenario: API 返回后刷新精确计数

- **WHEN** 一次 LLM 流式请求返回 `usage.total_tokens = 120_000`
- **THEN** `token_count` 更新为 120_000，`pending_estimate` 归零

#### Scenario: 大工具结果计入 pending 防撑爆

- **WHEN** API 上次回报 `token_count = 200_000`（模型上限 256_000），随后追加一条约 60_000 token 的工具结果尚未发出
- **THEN** 触发判定使用 200_000 + 估算 60_000 = 260_000，判定为需压缩（仅看 200_000 会漏判）

### Requirement: 三段式压缩与工具调用配对完整性

系统 SHALL 以「摘要旧消息 + 保留最近若干轮原样」的三段式策略压缩上下文：

1. 从尾部保留最近 `max_preserved_messages` 条 user/assistant 消息（默认 2）原样。
2. 切分保留起点时 MUST 保证 `tool_calls` 与其对应 `tool` 结果（以 `tool_call_id` 关联）不被拆分到压缩段与保留段两侧；必要时将保留起点前移以纳入完整配对。
3. 对压缩段发起一次不含工具的 LLM 摘要请求，使用结构化压缩 prompt。
4. 用「摘要消息 + 保留消息」作为新的工作上下文。

当可压缩消息为空（如全部需保留）时，系统 MUST NOT 发起摘要请求。

#### Scenario: 保留最近两轮并摘要更早历史

- **WHEN** 上下文含 10 条消息且 `max_preserved_messages=2`
- **THEN** 最近 2 条 user/assistant 原样保留，更早消息被摘要为单条摘要消息

#### Scenario: 不拆散工具调用配对

- **WHEN** 保留起点恰好落在某 `tool` 结果与其上游 `tool_calls` 之间
- **THEN** 系统将保留起点前移，使该 `tool_calls` 与对应 `tool` 结果同处一侧

#### Scenario: 无可压缩消息不调用 LLM

- **WHEN** 消息总数不足以在保留最近若干轮后留下可压缩内容
- **THEN** 系统不发起摘要请求，上下文保持不变

### Requirement: 结构化压缩 prompt

系统 SHALL 使用结构化压缩 prompt 指导摘要，按优先级保留「当前任务状态、报错与解法、最终可用代码、系统/环境上下文、设计决策、未完成 TODO」，并要求输出包含 `<current_focus>`、`<environment>`、`<completed_tasks>`、`<active_issues>`、`<code_state>`、`<important_context>` 等结构化区块。摘要 MUST 丢弃失败的中间尝试（保留教训）、合并重复讨论、对长代码仅保留签名与关键逻辑。

#### Scenario: 摘要保留报错与最终解法

- **WHEN** 压缩段包含一次报错及其最终修复方案
- **THEN** 摘要中保留完整报错信息与最终可用解法，丢弃失败的中间尝试

### Requirement: 压缩结果持久化与归档

系统 SHALL 将压缩结果持久化：被压缩的旧消息标记为已归档（archived），摘要作为新消息写入且不归档；会话级 token 基线（最近一次 API 精确用量）MUST 持久化以支持重启后估算。后续构造工作上下文 MUST 仅纳入未归档消息（摘要 + 保留消息）。归档 MUST NOT 物理删除原始消息。

摘要消息 MUST 以 `role="user"` 写入（与参考实现 kimi/deepy 一致），内容以固定前缀（如「Previous context has been compacted. Continue from this summary:」）+ 摘要正文组成。

#### Scenario: 压缩后重建只含摘要与保留消息

- **WHEN** 一次压缩完成后重新构造工作上下文
- **THEN** 上下文仅包含摘要消息与未归档的保留消息，不含已归档旧消息

#### Scenario: 摘要消息为 user 角色

- **WHEN** 压缩生成摘要消息
- **THEN** 该消息 `role` 为 `user`，含固定前缀与摘要正文

#### Scenario: 归档保留原始消息可追溯

- **WHEN** 旧消息被标记归档
- **THEN** 原始消息仍存在于存储中（仅标记，不物理删除）

### Requirement: 压缩与上下文用量事件

系统 SHALL 在以下时机 emit 事件供前端展示：

- `context_usage`：字段 `session_id`、`used_tokens`、`max_tokens`、`ratio`（`= used_tokens / max_tokens`，0~1）。在每次 API token 用量刷新后以及压缩完成后 emit。
- `context_compacted`：字段 `session_id`、`before_tokens`、`after_tokens`。在压缩成功后 emit。

#### Scenario: 用量刷新后通知前端比例

- **WHEN** 一次 LLM 响应返回并刷新精确 token 用量
- **THEN** 系统 emit `context_usage`，`ratio` 反映当前占用比例

#### Scenario: 压缩成功通知前端

- **WHEN** 一次自动压缩成功完成
- **THEN** 系统 emit `context_compacted`（含压缩前后 token 数），随后 emit 反映新占用的 `context_usage`

### Requirement: 压缩失败兜底

当压缩流程（含摘要 LLM 请求）失败时，系统 SHALL 记录错误并执行截断兜底：丢弃最旧的非保留消息直到满足预留预算，避免 turn 因上下文超限而彻底失败。

#### Scenario: 摘要请求失败仍可继续

- **WHEN** 压缩的摘要 LLM 请求抛出错误
- **THEN** 系统记录错误并截断最旧的非保留消息至满足预留预算，loop 继续而非直接中断

