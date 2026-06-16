## REMOVED Requirements

### Requirement: 首轮会话自动标题

**Reason**: 启发式泛化开场、第二轮纯 user 重试、18 字上限等规则由「两轮策略」替代。

**Migration**: 见 ADDED「会话自动标题（两轮策略）」与「会话标题状态字段」；存量标题不回溯改写。

## ADDED Requirements

### Requirement: 会话自动标题（两轮策略）

系统 SHALL 在 `turn_complete` 时按用户消息轮次自动更新仍由系统管理的会话标题，策略如下：

1. **第 1 轮**（`user_count == 1`）：若标题仍为默认（「新会话」或「会话 N」）且 `title_user_edited == false`，将标题设为该轮用户首条消息的清洗纯文本（去除 Markdown 标记与首尾空白）。若清洗后为空则保持默认标题。若文本超过存储上限（`MAX_STORED_TITLE_CHARS`，默认 120 字符），MUST 截断后入库（含省略号）。
2. **第 2 轮**（`user_count == 2`）：若 `autotitle_llm_done == false` 且 `title_user_edited == false`，MUST 使用**当前会话绑定的模型**、**非思考模式**（`thinking_enabled = false`）调用 LLM **一次**，基于**前两轮** user/assistant 对话生成单行标题，并**覆盖**第 1 轮已写入的标题；完成后 MUST 设 `autotitle_llm_done = true`。
3. **第 3 轮及以后**（`user_count >= 3`）：MUST NOT 自动修改标题。
4. 已有 ≥2 条 user 消息的历史会话在升级或后续继续对话时，MUST NOT 补跑 LLM 命名（依赖 `autotitle_llm_done` 迁移与 `user_count == 2` 窗口）。

LLM 标题生成 MUST 异步执行，MUST NOT 阻塞 `turn_complete` 主链路；完成后 MUST 通知前端刷新（如 `session_title_updated` 事件）。自动命名 MUST NOT 覆盖 `title_user_edited == true` 的会话标题。

#### Scenario: 首轮写入用户首条消息

- **WHEN** 用户首条消息为「请帮我分析 SK1002 课程归档资料」且该轮对话完成
- **THEN** 会话标题更新为清洗后的该条文本（不超过存储上限），侧栏可见

#### Scenario: 首轮寒暄也写入标题

- **WHEN** 用户首条消息仅为「你好」且该轮对话完成
- **THEN** 会话标题更新为「你好」（不再保持「新会话」）

#### Scenario: 超长首条消息截断入库

- **WHEN** 用户首条消息超过 `MAX_STORED_TITLE_CHARS` 且该轮对话完成
- **THEN** 标题以截断形式持久化，侧栏以 CSS ellipsis 展示

#### Scenario: 第二轮 LLM 覆盖第一轮标题

- **WHEN** 第 1 轮完成后标题为「请分析 report.docx」，第 2 轮对话完成且 `autotitle_llm_done == false`
- **THEN** 系统异步调用 LLM 生成更短摘要标题并覆盖原标题，且 `autotitle_llm_done` 置为 true

#### Scenario: LLM 仅触发一次

- **WHEN** 第 2 轮 LLM 标题已成功写入，用户继续第 3 轮及以后对话
- **THEN** 标题不再自动更新

#### Scenario: 超过两轮的历史会话不补跑

- **WHEN** 升级前会话已有 3 条及以上 user 消息，用户继续发送新消息
- **THEN** 系统不调用 LLM 生成标题

#### Scenario: 用户手动标题跳过 LLM

- **WHEN** 用户在第 2 轮前手动修改标题（`title_user_edited == true`）
- **THEN** 第 2 轮不调用 LLM，标题保持不变

#### Scenario: LLM 失败保留第一轮标题

- **WHEN** 第 2 轮 LLM 请求超时或失败
- **THEN** 保留第 1 轮清洗标题，不向用户展示错误；第 3 轮及以后 MUST NOT 重试 LLM

#### Scenario: 已有非默认标题不覆盖（用户手动）

- **WHEN** 会话标题已被用户手动设为「课程资料」（`title_user_edited == true`）
- **THEN** 系统不自动修改该标题

### Requirement: 会话标题状态字段

系统 SHALL 在 `sessions` 表持久化 `autotitle_llm_done` 与 `title_user_edited` 布尔字段。用户通过 `update_session` 修改标题时 MUST 将 `title_user_edited` 置为 true。数据库迁移 MUST 对已有 `user_message_count >= 2` 的会话将 `autotitle_llm_done` 设为 true，以避免历史数据误触发 LLM。

#### Scenario: 迁移标记历史会话

- **WHEN** 应用升级并完成 migration
- **THEN** 已有 ≥2 条 user 消息的会话 `autotitle_llm_done == true`

#### Scenario: 手动改标题标记

- **WHEN** 用户通过 `update_session` 修改会话标题
- **THEN** `title_user_edited == true`
