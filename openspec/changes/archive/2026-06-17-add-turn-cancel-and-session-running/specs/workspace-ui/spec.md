## ADDED Requirements

### Requirement: 按会话运行态（per-session running）

前端 SHALL 按 `session_id` 维护运行态（`idle` | `running` | `stopping`），含该 session 的 streaming 缓冲、`liveTools` 与 `turn_id`。`agent-event` 处理 MUST 更新对应 session 的运行态，**不得**因非 active 会话丢弃事件。切换 `activeSessionId` MUST NOT 清除其他 session 的 running 状态。

#### Scenario: 切换会话保留后台进度

- **WHEN** session A 正在 running，用户切换到 session B 再切回 A
- **THEN** A 的工具链与流式预览（若仍在 running）恢复展示，无需重新发送

#### Scenario: 非 active session 仍接收事件

- **WHEN** session A running 且 activeSession 为 B
- **THEN** A 的 `tool_call` / `tool_result` 事件仍更新 A 的运行态 Map

### Requirement: 侧栏 running 指示

侧栏会话列表 SHALL 对 `running` 或 `stopping` 状态的会话显示视觉指示（如 spinner 或圆点）。用户点击 running 的非 active 会话 MUST 切换至该会话查看进度。

#### Scenario: running 会话显示指示

- **WHEN** session A 处于 running
- **THEN** 侧栏 A 项显示 running 指示，与 idle 会话区分

#### Scenario: stopping 状态

- **WHEN** 用户点击停止后 session 处于 stopping
- **THEN** 侧栏显示 stopping 指示（可与 running 区分样式），直至 `turn_cancelled`

### Requirement: Stop 按钮

Chat 输入区 SHALL 在当前 active session 为 `running` 时展示 **停止** 按钮（与发送互斥：running 时禁用发送）。点击 MUST 调用 `cancel_turn` 并将该 session 置 `stopping`。`stopping` 时停止按钮 disabled，placeholder 或 activity 文案 MUST 说明可能等待当前工具结束（最长约 30 秒）。`turn_awaiting_user`（clarify）时 MUST NOT 展示 Stop（澄清流程使用 clarify 卡片）。

#### Scenario: running 时显示停止

- **WHEN** active session 为 running
- **THEN** 输入区 disabled，停止按钮可见且可点击

#### Scenario: stop 后 stopping

- **WHEN** 用户点击停止
- **THEN** session 进入 stopping，直至收到 `turn_cancelled`

#### Scenario: clarify 时不显示 stop

- **WHEN** active session 收到 `turn_awaiting_user` 且展示 ClarifyQuestionCard
- **THEN** 不显示 Stop 按钮，输入区按 clarify 规则启用

### Requirement: turn_cancelled 后 UI 对齐

收到 `turn_cancelled` 后，前端 SHOULD 调用 `list_messages` 与当前 session 的 tool calls 对齐 DB，清空该 session streaming 缓冲，运行态置 idle。用户可见的 assistant 步骤 MUST 与 cancel 前已 emit 的 `assistant_step_done` 一致，不出现重复条。

#### Scenario: cancel 后消息列表一致

- **WHEN** cancel 前已逐步展示 2 条 assistant，随后收到 `turn_cancelled`
- **THEN** 消息列表仍为 2 条 assistant（加 user），无第三条空流式框残留

## MODIFIED Requirements

### Requirement: 中间区 Markdown 流式渲染

系统 SHALL 在中间区以良好的 Markdown 渲染展示会话与结果，支持流式增量更新、代码高亮与表格；思考内容与正文分区展示。assistant 消息的**流式预览**与**持久化展示** MUST 使用同一消息气泡结构（思考可折叠区 + 正文 Markdown 区），仅允许样式 variant（如边框/动效）区分「生成中」与「已完成」。多轮工具调用时，每一步 LLM 的流式预览 MUST 独立呈现，不得将多步思考/正文累加在同一流式气泡中。收到 `assistant_step_done` 后，该步 assistant MUST 立即出现在消息列表中，并清空当前 streaming 缓冲；`turn_complete` 时仍可全量 `list_messages` 对齐，但 MUST NOT 导致 assistant 消息条数或内容与逐步展示结果发生可见冲突。`turn_cancelled` 时 MUST 同样清空 streaming 缓冲且不得错误 emit 完成态。user 消息若含图片附件，MUST 在文本旁展示缩略图。

#### Scenario: 流式渲染回答

- **WHEN** 模型流式返回正文
- **THEN** 中间区随增量平滑渲染 Markdown，代码块高亮、表格正确呈现

#### Scenario: 思考内容可折叠

- **WHEN** 模型返回思考内容
- **THEN** 思考内容以可折叠的独立区域展示，不与正文混排

#### Scenario: 逐步固化与流式预览一致

- **WHEN** 某步 LLM 流式输出结束并收到 `assistant_step_done`
- **THEN** 该步 assistant 以持久消息形式出现在列表中，布局与流式预览一致，且 streaming 预览区被清空

#### Scenario: 多步工具调用分步展示

- **WHEN** Agent 连续执行两轮及以上 LLM（含工具调用）
- **THEN** 中间区按步显示多条 assistant 消息，每条对应该步持久化内容，而非合并为一条超长流式气泡

#### Scenario: turn_complete 无布局跳变

- **WHEN** 回合结束并触发 `turn_complete` 后的 `list_messages`
- **THEN** 用户可见的 assistant 消息条数与内容与逐步展示阶段一致，不出现流式框消失后突然拆条或合并的重排

#### Scenario: turn_cancelled 清空流式预览

- **WHEN** 用户 stop 且收到 `turn_cancelled`
- **THEN** streaming 预览区清空，不出现悬挂中的 indigo 流式框

#### Scenario: 用户消息展示图片附件

- **WHEN** 历史消息含 `attachments_json` 指向 `.cache/attachments/photo.png`
- **THEN** 消息气泡展示该图缩略图与文本内容

#### Scenario: 附件文件缺失时展示占位

- **WHEN** 历史消息含 `attachments_json` 但磁盘文件不存在
- **THEN** 展示占位提示而非崩溃
