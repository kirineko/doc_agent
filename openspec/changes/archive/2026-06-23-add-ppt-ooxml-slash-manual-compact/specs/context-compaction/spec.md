## ADDED Requirements

### Requirement: 手动上下文压缩

系统 SHALL 提供用户手动触发上下文压缩的能力，通过 IPC `compact_session` 与斜杠 command `/compact` 调用。

手动压缩 MUST 跳过自动压缩的阈值判定（`should_auto_compact`），在任意上下文占用比例下尝试执行压缩核心流程。

手动压缩 MUST 与自动压缩共享同一压缩核心（`prepare_compaction_split`、摘要 LLM、`compact.md` prompt、归档、`add_compaction_summary`、`rebuild_working_messages`、摘要 LLM 失败时的 `truncate_fallback_compact_only`）。

手动压缩 MUST NOT 向会话写入 `/compact` 或任何「用户执行了压缩」类 user 消息。持久化效果 MUST 与自动压缩一致：较早消息归档 + 写入一条压缩摘要 user 消息（固定前缀 + 摘要正文）。

当 `prepare_compaction_split` 返回 `None`（历史过短、无可压缩段）时，手动压缩 MUST 不调用摘要 LLM、不执行 `truncate_fallback`，返回 `compacted: false`，并由前端提示用户无需压缩。

手动压缩 MUST 在以下情况被拒绝：会话不存在、会话 turn 处于 `running` 或 `stopping`、clarify pending（与 `/init` 一致）。

`before_tokens` MUST 使用会话持久化 `token_count`（无进行中 turn 时 `pending_estimate` 为 0）；若尚无 API 用量，MAY 用活跃消息文本估算。

#### Scenario: 手动压缩跳过阈值

- **WHEN** 当前 token 远低于 `max_context_size * 0.85` 且 `prepare_compaction_split` 返回可压缩段
- **THEN** `compact_session` 仍执行 LLM 摘要并归档
- **AND** 不因未达自动阈值而拒绝

#### Scenario: 手动压缩无可压缩段时 no-op

- **WHEN** 活跃消息仅含 1 轮 user+assistant（或更少）导致 `prepare_compaction_split` 为 `None`
- **THEN** `compact_session` 返回 `compacted: false`
- **AND** 不调用摘要 LLM
- **AND** 不写入新的摘要消息
- **AND** 前端展示「当前上下文较短，无需压缩」类提示

#### Scenario: 手动压缩持久化与自动一致

- **WHEN** 手动压缩成功摘要较长历史
- **THEN** 被压缩消息标记 archived
- **AND** 写入一条 `role=user` 的压缩摘要消息（含固定前缀）
- **AND** 聊天历史不出现 `/compact` 文本

#### Scenario: 手动压缩摘要失败走 truncate

- **WHEN** 手动压缩时摘要 LLM 失败且 `prepare_compaction_split` 曾为 `Some`
- **THEN** 系统对可压缩段执行 `truncate_fallback_compact_only`（与自动路径一致）

#### Scenario: 手动压缩在 turn 运行中被拒

- **WHEN** 会话有 turn 处于 running
- **THEN** `compact_session` 返回错误
- **AND** 不修改消息归档状态

#### Scenario: 手动压缩在 clarify pending 中被拒

- **WHEN** 会话存在 pending clarify
- **THEN** `compact_session` 返回错误（或前端阻断调用）

### Requirement: compact_session IPC 响应

`compact_session` SHALL 返回 JSON 包含：`compacted`（bool）、`before_tokens`（u32）、`after_tokens`（u32）、可选 `reason`（如 `nothing_to_compact`）。

成功压缩时 MUST emit `context_compacted`（`trigger: manual`）与 `context_usage`。

#### Scenario: 成功响应形状

- **WHEN** 手动压缩成功完成
- **THEN** IPC 返回 `compacted: true` 及压缩前后 token 数
- **AND** emit `context_compacted` 含 `trigger: "manual"`

#### Scenario: no-op 响应形状

- **WHEN** 无可压缩段
- **THEN** IPC 返回 `compacted: false` 及 `reason` 表明 nothing_to_compact

## MODIFIED Requirements

### Requirement: 压缩触发判定

系统 SHALL 提供统一的压缩触发判定函数，依据当前 token 数与模型上下文上限决定是否**自动**压缩。命中以下任一条件即触发（谁先满足谁触发）：

1. 比例触发：`token_count >= max_context_size * trigger_ratio`
2. 预留触发：`token_count + reserved_context_size >= max_context_size`

其中 `trigger_ratio` 默认 0.85；`reserved_context_size` 默认按模型上限取 `max(50_000, max_context_size * 0.1)`。`token_count <= 0` 或 `max_context_size <= 0` 时 MUST NOT 触发。

**手动压缩（`compact_session`）MUST NOT 使用本判定函数作为门禁。**

#### Scenario: DeepSeek 1M 比例先触发

- **WHEN** 当前 token 为 850_000、模型上限 1_000_000、`trigger_ratio=0.85`、`reserved=100_000`
- **THEN** 判定触发压缩（比例条件 850K ≥ 850K 满足）

#### Scenario: Kimi 256K 预留先触发

- **WHEN** 当前 token 为 210_000、模型上限 256_000、`trigger_ratio=0.85`、`reserved=50_000`
- **THEN** 判定触发压缩（预留条件 210K + 50K ≥ 256K 满足，且早于比例阈值 217.6K）

#### Scenario: 空上下文不触发

- **WHEN** 当前 token 为 0
- **THEN** MUST NOT 触发自动压缩

### Requirement: 压缩与上下文用量事件

系统 SHALL 在以下时机 emit 事件供前端展示：

- `context_usage`：字段 `session_id`、`used_tokens`、`max_tokens`、`ratio`（`= used_tokens / max_tokens`，0~1）。在每次 API token 用量刷新后以及压缩完成后 emit。
- `context_compacted`：字段 `session_id`、`before_tokens`、`after_tokens`、`trigger`（`"auto"` | `"manual"`）。在压缩成功后 emit。

压缩完成后的 `after_tokens` MUST 按「摘要 LLM `usage.output_tokens` + 保留消息文本估算」计算，**不得**将保留 tail 中的图片 base64 计入；下一次主 loop API usage 可校正。

#### Scenario: 用量刷新后通知前端比例

- **WHEN** 一次 LLM 响应返回并刷新精确 token 用量
- **THEN** 系统 emit `context_usage`，`ratio` 反映当前占用比例

#### Scenario: 自动压缩成功通知前端

- **WHEN** 一次自动压缩成功完成
- **THEN** 系统 emit `context_compacted`（`trigger: "auto"`，含压缩前后 token 数），随后 emit 反映新占用的 `context_usage`

#### Scenario: 手动压缩成功通知前端

- **WHEN** 一次手动压缩成功完成
- **THEN** 系统 emit `context_compacted`（`trigger: "manual"`，含压缩前后 token 数），随后 emit 反映新占用的 `context_usage`
