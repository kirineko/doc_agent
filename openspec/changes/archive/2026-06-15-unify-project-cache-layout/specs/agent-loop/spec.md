## MODIFIED Requirements

### Requirement: clarify 人机协作断点

当 Agent loop 某轮 tool_calls 中包含 `clarify_ask` 且参数校验通过时，系统 SHALL：

1. 持久化 assistant 消息与全部 tool_calls（与现有行为一致），emit `assistant_step_done`
2. **先按序执行所有非 clarify 工具**，正常写入 result 与 tool message
3. 对第一个 `clarify_ask`：更新 tool_call status 为 `awaiting_user`、写入 `clarify_pending`、emit `ToolCall` 事件（status=`awaiting_user`）、emit `clarify_question` 事件（payload 含完整 ClarifyQuestion + `tool_call_id`）
4. 对其余多出的 `clarify_ask`：立即写入结构化错误 result
5. emit `turn_awaiting_user`（含 session_id、turn_id）
6. `run_turn` 正常 return；MUST NOT emit `turn_complete`，MUST NOT 执行下一轮 LLM，MUST NOT 执行 turn 结束清理（`.cache/skill-run/` 清理留待 turn 真正结束）

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
