## 1. 数据层与类型

- [x] 1.1 `store.rs`：新增 `clarify_pending` 表（`CREATE TABLE IF NOT EXISTS`）与 CRUD（save / get / delete 返回行数）；`delete` 用于原子防双提交
- [x] 1.2 `agent/types.rs`：新增 `ClarifyQuestion`、`ClarifyOption`、`ClarifyAnswer` 结构与 AgentEvent 变体（`clarify_question`、`turn_awaiting_user`）
- [x] 1.3 `types.ts`：对齐 ClarifyQuestion、AgentEvent 新变体、`SubmitClarifyAnswerRequest`（仅 `session_id`/`question_id`/`selected`/`custom`）

## 2. clarify_ask 工具

- [x] 2.1 新增 `tools/clarify.rs`：`clarify_ask` tool spec + 参数校验（kind 枚举、single/multi 2–6 options、confirm_brief 必带 brief），注册到 `registry.rs`
- [x] 2.2 单元测试：合法/非法 schema、options 数量边界、brief 缺失

## 3. Loop 暂停与恢复

- [x] 3.1 `loop_runner.rs` 暂停分支：非 clarify 工具先执行；首个 clarify_ask 置 `awaiting_user` + 写 pending + emit `ToolCall(awaiting_user)` / `clarify_question` / `turn_awaiting_user` 后 return；多余 clarify_ask 写错误 result；校验失败视同普通工具错误继续 loop
- [x] 3.2 实现 `resume_turn`：从 store 重建 working_messages（不追加 user message）、沿用 turn_id、autotitle user_text 取最后一条 user 消息
- [x] 3.3 IPC `submit_clarify_answer`：事务删 pending（0 行报错）→ 按 question_json 校验答案 → 后端组装 display_text 与 tool result → finish_tool_call + tool message → emit ToolResult → resume_turn
- [x] 3.4 IPC `send_message` 入口增加 pending 检查（存在则拒绝）
- [x] 3.5 IPC `cancel_clarify`（可选）：result 写 `{"cancelled":true}` 后 resume
- [x] 3.6 Mock provider 新增「澄清」关键词场景：返回合法 clarify_ask，收到 tool result 后给最终回答
- [x] 3.7 集成测试（mock provider）：clarify_ask → pause（无 turn_complete）→ submit → resume → turn_complete；双 submit 第二次报错；混合工具轮次（非 clarify 先执行）；pending 时 send_message 被拒

## 4. 更新 clarify skill

- [x] 4.1 更新 `clarify/SKILL.md`：每问 MUST `clarify_ask`（一次一问）、简报确认 MUST `confirm_brief`、禁止纯文本问卷；同步更新相关断言测试

## 5. 前端交互

- [x] 5.1 新增 `ClarifyQuestionCard`：四种 kind + 自定义输入 + multi min/max 校验 + confirm_brief 预览
- [x] 5.2 已答卡片：从 bundle ToolCallRecord（clarify_ask, done）渲染只读态嵌入消息流
- [x] 5.3 `useWorkspace`：处理 `clarify_question` / `turn_awaiting_user` 事件、pending 状态、submit invoke、bundle 恢复活跃卡片
- [x] 5.4 `ChatPanel`：pending 时 suppress SuggestionCards、阻断发送并提示、placeholder 引导
- [x] 5.5 工具链面板：`awaiting_user` 状态显示「等待回答」；clarify_ask 中文标签
- [x] 5.6 组件测试：各 kind 渲染与 submit payload、multi 校验、已答态渲染

## 6. 验证

- [x] 6.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` + `npm run typecheck && npm test && npm run build`
- [x] 6.2 手动：模糊 PPT 请求 → 多轮 clarify 卡片 → 简报确认 → 生成交付物；刷新恢复活跃卡片；澄清后 autotitle 与推荐问行为正常
