## MODIFIED Requirements

### Requirement: resume_turn 恢复循环

系统 SHALL 提供 `resume_turn(session_id, turn_id)`：从 DB 全量重建 `working_messages`（经 `messages_from_store` + system prompt 注入，MUST NOT 追加新 user message），在同一 `turn_id` 下继续工具循环，直至无 tool_calls（emit `turn_complete`）或再次 clarify 暂停。每次 resume 步数预算重置为 `MAX_TOOL_STEPS`。该 turn 最终 `turn_complete` 时的自动标题逻辑 MUST 与 project-session spec「会话自动标题（两轮策略）」一致：以 DB 中当前 user 消息计数（`user_count`）判定第 1/2/3 轮窗口，MUST NOT 使用启发式摘要；第 2 轮 LLM 标题仅当 `autotitle_llm_done == false` 且 `title_user_edited == false` 时触发一次。

#### Scenario: 提交答案后继续澄清

- **WHEN** 用户提交 clarify 答案且 Agent 再次调用 `clarify_ask`
- **THEN** 再次进入暂停，直至澄清完成

#### Scenario: 澄清完成后进入生成

- **WHEN** 用户确认创作简报且 Agent 不再调用 `clarify_ask`
- **THEN** loop 正常执行 skill_read / skill_run 等，最终 emit `turn_complete`

#### Scenario: resume 重建含 reasoning_content

- **WHEN** resume 重建 working_messages 且历史含带 tool_calls 的 assistant 消息
- **THEN** 该消息的 `reasoning_content` 正确回填（沿用现有「工具调用轮的 reasoning_content 回填」要求）

#### Scenario: resume 完成时按 user_count 触发标题

- **WHEN** resume 后 loop 结束并 emit `turn_complete`，且该会话累计 user 消息数为 2
- **THEN** 若满足 LLM 标题条件则异步生成标题，否则按两轮策略第 1 轮或跳过
