## ADDED Requirements

### Requirement: 斜杠命令选择器

输入框 SHALL 支持 `/` 触发的斜杠命令选择器：检测到光标前位于**行首或空白字符后**的 `/` 且 `/` 与光标之间无空白时，弹出命令候选列表；支持 fzf 式模糊匹配、按分类分组展示、键盘上下选择与确认。

确认后 MUST 将输入框中的 `/query` 替换为该命令的 `prompt` 文本，**MUST NOT** 自动发送消息；输入框 MUST 获得焦点供用户编辑。行为 MUST 与「推荐问点击填入输入框」一致。

斜杠弹层与 `@` 文件引用弹层 MUST 互斥：二者同时满足触发条件时，仅展示 `@` 弹层。

澄清进行中（`activeClarify`）、busy、initializing 时 MUST NOT 展示斜杠弹层（与输入 disabled 一致）。

#### Scenario: 选择命令填入 prompt

- **WHEN** 用户输入 `/word` 并在弹层中选择「精准修改 Word」
- **THEN** 输入框中 `/word` 被替换为对应 prompt（含 `@文件名.docx` 与改动占位）
- **AND** 消息未被发送

#### Scenario: 键盘确认

- **WHEN** 斜杠弹层展示且用户按 Enter
- **THEN** 当前高亮命令的 prompt 填入输入框且不发送

#### Scenario: Esc 关闭

- **WHEN** 弹层展示中用户按 Esc
- **THEN** 弹层关闭，输入内容保持不变

#### Scenario: 与 @ 互斥

- **WHEN** 输入为 `分析 @报 /word` 且光标在 `@报` 区域内
- **THEN** 仅展示 `@` 文件弹层，不展示斜杠命令弹层

#### Scenario: 澄清期间不可用

- **WHEN** session 存在 pending 的 clarify 问题
- **THEN** 不展示斜杠命令弹层

### Requirement: 斜杠命令弹层 UI

斜杠命令弹层 MUST 按分类展示分组标题（通用、Word、PPT、Excel、PDF、Web），每条候选 MUST 展示 `label` 与一行 `description`；匹配字符 MUST 高亮（与 `@` 弹层同类样式）。

弹层 MUST 可滚动；样式 MUST 复用或延伸现有 `mention-popup` 设计令牌，与明暗主题一致。

#### Scenario: 分组标题与默认顺序

- **WHEN** 用户键入 `/` 且 query 为空
- **THEN** 弹层按 general、word、ppt、excel、pdf、web 顺序展示分组
- **AND** 每组内命令顺序与 registry 一致

#### Scenario: 双行展示

- **WHEN** 弹层展示 `word:edit`
- **THEN** 主行显示「精准修改 Word」，副行显示 registry 中的 description

## MODIFIED Requirements

### Requirement: 推荐问展示交互

首次会话与 follow-up 推荐问 SHALL 均以胶囊按钮形式统一展示在输入框上方；点击任一推荐 MUST 将该文本填入输入框供用户编辑，不得直接发送。每条推荐问长度 MUST 不超过 80 个字符。follow-up 生成期间 MUST NOT 禁用输入框；用户先行发送消息或切换会话时，迟到的 follow-up 结果 MUST 被丢弃。

斜杠命令选择器与推荐问 MUST 共存：推荐问在输入框上方，斜杠弹层在 textarea 上方；二者互不阻断。

#### Scenario: 点击推荐填入输入框

- **WHEN** 用户点击一条推荐问胶囊
- **THEN** 该文本出现在输入框中且输入框获得焦点，推荐区在用户发送消息前仍可展示

#### Scenario: followup 迟到丢弃

- **WHEN** followup 推荐尚未返回时用户已手动发送了新消息
- **THEN** 返回的推荐结果被丢弃，不展示

#### Scenario: 斜杠与推荐问共存

- **WHEN** 空会话同时展示 starter 推荐问且用户在输入框键入 `/`
- **THEN** 推荐问胶囊仍可见，斜杠弹层在 textarea 上方正常展示
