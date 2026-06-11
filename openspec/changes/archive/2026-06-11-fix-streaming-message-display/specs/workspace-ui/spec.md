# workspace-ui 能力增量

## MODIFIED Requirements

### Requirement: 中间区 Markdown 流式渲染
系统 SHALL 在中间区以良好的 Markdown 渲染展示会话与结果，支持流式增量更新、代码高亮与表格；思考内容与正文分区展示。assistant 消息的**流式预览**与**持久化展示** MUST 使用同一消息气泡结构（思考可折叠区 + 正文 Markdown 区），仅允许样式 variant（如边框/动效）区分「生成中」与「已完成」。多轮工具调用时，每一步 LLM 的流式预览 MUST 独立呈现，不得将多步思考/正文累加在同一流式气泡中。收到 `assistant_step_done` 后，该步 assistant MUST 立即出现在消息列表中，并清空当前 streaming 缓冲；`turn_complete` 时仍可全量 `list_messages` 对齐，但 MUST NOT 导致 assistant 消息条数或内容与逐步展示结果发生可见冲突。

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
