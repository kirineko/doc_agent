## ADDED Requirements

### Requirement: clarify 人机协作断点

当 Agent loop 某轮 tool_calls 中包含 `clarify_ask` 且参数校验通过时，系统 SHALL：

1. 持久化 assistant 消息与全部 tool_calls（与现有行为一致），emit `assistant_step_done`
2. **先按序执行所有非 clarify 工具**，正常写入 result 与 tool message
3. 对第一个 `clarify_ask`：更新 tool_call status 为 `awaiting_user`、写入 `clarify_pending`、emit `ToolCall` 事件（status=`awaiting_user`）、emit `clarify_question` 事件（payload 含完整 ClarifyQuestion + `tool_call_id`）
4. 对其余多出的 `clarify_ask`：立即写入结构化错误 result
5. emit `turn_awaiting_user`（含 session_id、turn_id）
6. `run_turn` 正常 return；MUST NOT emit `turn_complete`，MUST NOT 执行下一轮 LLM，MUST NOT 执行 turn 结束清理（`.skill-run/` 清理留待 turn 真正结束）

#### Scenario: clarify 后 loop 暂停

- **WHEN** 模型返回 `clarify_ask` 且校验通过
- **THEN** 前端收到 `turn_awaiting_user` 而非 `turn_complete`，工具链卡片显示等待状态

#### Scenario: clarify 与普通工具混合

- **WHEN** 模型同轮返回 `[fs_read, clarify_ask]`
- **THEN** `fs_read` 先正常执行并写入 result，随后才进入 clarify 暂停；暂停时刻该 assistant 轮仅 clarify 调用缺 result

#### Scenario: 参数非法不暂停

- **WHEN** `clarify_ask` 参数校验失败
- **THEN** 写入错误 result 后 loop 继续（视同普通工具失败），不创建 pending

#### Scenario: 非 clarify 工具不受影响

- **WHEN** 模型返回 `fs_read` 等普通工具
- **THEN** 行为与变更前一致，同步 execute 并继续 loop

---

### Requirement: resume_turn 恢复循环

系统 SHALL 提供 `resume_turn(session_id, turn_id)`：从 DB 全量重建 `working_messages`（经 `messages_from_store` + system prompt 注入，MUST NOT 追加新 user message），在同一 `turn_id` 下继续工具循环，直至无 tool_calls（emit `turn_complete`）或再次 clarify 暂停。每次 resume 步数预算重置为 `MAX_TOOL_STEPS`。最终回答轮的 autotitle 判定 MUST 以 history 中最后一条 `role='user'` 消息为 user_text 依据。

#### Scenario: 提交答案后继续澄清

- **WHEN** 用户提交 clarify 答案且 Agent 再次调用 `clarify_ask`
- **THEN** 再次进入暂停，直至澄清完成

#### Scenario: 澄清完成后进入生成

- **WHEN** 用户确认创作简报且 Agent 不再调用 `clarify_ask`
- **THEN** loop 正常执行 skill_read / skill_run 等，最终 emit `turn_complete`

#### Scenario: resume 重建含 reasoning_content

- **WHEN** resume 重建 working_messages 且历史含带 tool_calls 的 assistant 消息
- **THEN** 该消息的 `reasoning_content` 正确回填（沿用现有「工具调用轮的 reasoning_content 回填」要求）

---

### Requirement: turn_awaiting_user 事件

系统 SHALL 新增 AgentEvent `turn_awaiting_user`，字段含 `session_id`、`turn_id`。语义：本轮用户请求尚未完成，等待 clarify 交互。

#### Scenario: 前端 busy 状态

- **WHEN** 收到 `turn_awaiting_user`
- **THEN** 前端 MUST 将 `busy` 置为 false，以便用户操作澄清卡片（与 `turn_complete` 区分）

---

### Requirement: pending 期间拒绝新消息

当 session 存在 clarify pending 时，`send_message` command MUST 返回错误（提示先完成澄清），后端入口强制校验，不依赖前端拦截。

#### Scenario: pending 时发送被后端拒绝

- **WHEN** session 存在 clarify pending 且 `send_message` 被调用
- **THEN** command 返回错误，不写入 user message、不启动新 turn

## MODIFIED Requirements

### Requirement: Agent 多轮工具调用循环

系统 SHALL 实现一个 Agent 执行循环：构造会话上下文后向模型发起流式请求，若返回包含工具调用则在沙箱内执行并将结果回填，再次请求模型，如此重复，直到模型返回不含工具调用的最终回答为止。**例外**：`clarify_ask` 工具调用进入 `awaiting_user` 状态时，loop 暂停等待 `submit_clarify_answer`，通过 `resume_turn` 继续，暂停期间不视为 turn 完成。

#### Scenario: 单轮工具调用后给出答案

- **WHEN** 用户提问需要读取一个文档，模型返回一个 `read_to_markdown` 工具调用
- **THEN** 系统在沙箱内执行该工具，将结果作为 `tool` 消息回填，并再次请求模型
- **AND** 模型基于工具结果返回最终回答时，循环结束

#### Scenario: 多轮连续工具调用

- **WHEN** 模型在一次回答中先后需要「列目录」再「读取某文件」
- **THEN** 系统按序执行每个工具调用、逐次回填，并持续循环直到模型不再请求工具

#### Scenario: 达到最大轮次保护

- **WHEN** 工具调用轮次达到配置的上限
- **THEN** 系统终止循环并向用户返回已产出的内容与「已达最大步数」提示

#### Scenario: clarify 暂停后继续

- **WHEN** 模型返回 `clarify_ask` 且用户提交答案
- **THEN** 系统通过 `resume_turn` 继续循环，且不计为新的用户 turn

---

### Requirement: 无密钥可运行的 Mock Provider

系统 SHALL 提供一个 Mock Provider，在未配置真实 API Key 时仍可驱动循环、工具调用与前端事件，用于开发与测试。Mock Provider MUST 支持 clarify 场景：用户文本命中约定关键词（如「澄清」）时返回一个 `clarify_ask` tool call，以便无 Key 验证暂停/恢复全链路。

#### Scenario: Mock 模式跑通端到端

- **WHEN** 未配置任何模型 API Key 且选择 Mock Provider
- **THEN** 系统可产生模拟的思考 / 正文 / 工具调用事件，完整跑通 UI 与工具执行

#### Scenario: Mock clarify 场景

- **WHEN** Mock Provider 收到含「澄清」关键词的用户消息
- **THEN** 返回一个合法 `clarify_ask` tool call，触发暂停流程；收到对应 tool result 后返回最终回答
