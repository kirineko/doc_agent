## MODIFIED Requirements

### Requirement: clarify skill 流程定义

clarify skill 的 SKILL.md SHALL 定义完整的需求澄清流程。澄清过程中 **MUST** 通过 `clarify_ask` 工具发起每一道结构化问题；**禁止**仅用 assistant 纯文本列出选项或问卷。创作简报汇总 **MUST** 使用 `clarify_ask` 且 `kind=confirm_brief`，并等待用户确认后再进入 `skill_read` + 生成。

流程步骤更新为：

1. 识别文档类型与场景
2. 评估已有信息，选择深度路径
3. **逐问澄清** — 每问调用一次 `clarify_ask`（一次一问）
4. **汇总确认** — `clarify_ask` + `confirm_brief`，等待用户确认
5. **转入生成** — `skill_read` 对应格式 skill

#### Scenario: 标准路径使用 clarify_ask

- **WHEN** 用户发送模糊 PPT 创作请求且 Agent 进入 clarify 流程
- **THEN** Agent 通过 `clarify_ask` 逐题澄清，而非在 assistant 消息中列出 A/B/C 文本选项

#### Scenario: 创作简报经 confirm_brief 确认

- **WHEN** 澄清轮次完成
- **THEN** Agent 调用 `clarify_ask`（`kind=confirm_brief`）展示创作简报，用户确认后才 `skill_read` 对应格式 skill

#### Scenario: 澄清完成输出创作简报

- **WHEN** 用户在前端确认创作简报
- **THEN** 对话历史含可读的已答澄清卡片，且 Agent 收到的 tool result 含结构化 brief 字段
