## ADDED Requirements

### Requirement: 工具调用文件锁准入

Agent loop 在执行每个非网络工具调用前 MUST 构造 `ToolIoPlan` 并申请文件锁。锁申请成功后才可调用工具 handler；锁申请失败时 MUST 将冲突写入失败 tool result，不得执行工具 handler，不得修改磁盘。

#### Scenario: 文件锁失败不中断 loop

- **WHEN** `fs_write` 因 `out.docx` 被其他 session 占用而无法申请锁
- **THEN** loop 持久化该 tool call 的失败 result
- **AND** 后续 LLM step 可基于失败信息向用户解释或选择其他路径

### Requirement: 全局 running slot 生命周期

`run_turn` 与 `resume_turn` MUST 在进入主 loop 前申请全局 running slot，并在 turn 进入任一 terminal 或 paused 状态时释放。terminal 或 paused 状态包括 `turn_complete`、`turn_cancelled`、`turn_awaiting_user`、达到最大工具步数、provider 不可恢复错误与工具批处理不可恢复错误。

#### Scenario: turn_complete 释放 slot

- **WHEN** session A emit `turn_complete`
- **THEN** 全局 running 计数减少 1

#### Scenario: turn_awaiting_user 释放 slot

- **WHEN** session A emit `turn_awaiting_user`
- **THEN** 全局 running 计数减少 1，允许其他 session 启动

#### Scenario: provider error 释放 slot

- **WHEN** LLM 请求失败导致 turn 退出
- **THEN** 全局 running slot MUST 被释放

### Requirement: send_message 启动前拒绝全局满额

当全局已有 3 个 running turns 时，`send_message` MUST 在持久化新 user message 前拒绝。该拒绝不创建 assistant/tool messages，不改变 session 上下文。

#### Scenario: 满额发送不污染历史

- **WHEN** 全局已有 3 个 running turns
- **AND** 用户在 idle session 输入「生成报告」并发送
- **THEN** 后端返回满额错误
- **AND** `list_messages` 中不出现该 user message

### Requirement: resume_turn 启动前拒绝全局满额

当全局已有 3 个 running turns 时，`resume_turn` MUST 在提交 clarify answer 或写入 clarify tool result 前拒绝，并保持 clarify pending 可继续回答。

#### Scenario: 满额时澄清答案不提交

- **WHEN** session A 处于 clarify pending
- **AND** 全局已有 3 个其他 running turns
- **WHEN** 用户提交 A 的澄清答案
- **THEN** 后端返回满额错误
- **AND** A 的 clarify pending 仍存在

## MODIFIED Requirements

### Requirement: 异步工具执行

系统 SHALL 支持异步工具 handler；`loop_runner` 在沙箱、Secrets、session/turn metadata 与文件锁上下文内 await 工具执行结果。文件系统类工具 MUST 在执行前通过 `ToolIoPlan` 申请所需锁。网络 I/O MUST NOT 阻塞 tokio worker 以外的同步阻塞调用。

#### Scenario: web 工具异步完成

- **WHEN** 模型调用 `web_search` 且 Tavily API 需要网络等待
- **THEN** loop 异步等待 handler 完成后再 persist tool result 并继续下一轮 LLM

#### Scenario: 文件工具先锁后执行

- **WHEN** 模型调用 `ooxml_pack`
- **THEN** loop 先申请解包目录 read lock、original read lock 与 out_path write lock，再执行 handler
