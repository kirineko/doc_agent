## ADDED Requirements

### Requirement: 模型与密钥 Drawer

系统 SHALL 将模型选择、思考配置与 API Key 配置从侧栏主列表迁入「模型与密钥」Drawer（右侧滑出）。侧栏 MUST 保留当前模型摘要（名称、vision 标识、思考状态）与打开 Drawer 的入口。

#### Scenario: 打开 Drawer 配置模型

- **WHEN** 用户点击侧栏「模型与密钥」
- **THEN** 右侧 Drawer 展示按 Provider 分组的 5 个模型、思考开关、DeepSeek 强度（若适用）及三 Provider API Key

#### Scenario: 侧栏摘要含 vision 标识

- **WHEN** 当前选中 Kimi K2.6
- **THEN** 侧栏摘要显示模型名与视觉能力图标（如 Eye）

### Requirement: 非 vision 粘贴 Toast

当用户在非 vision 模型下粘贴图片时，系统 SHALL 展示非阻塞 toast，文案说明需切换至支持视觉的模型（Kimi K2.6 或 MiMo v2.5）。

#### Scenario: DeepSeek 下粘贴图片

- **WHEN** 会话模型为 DeepSeek V4 Flash 且用户粘贴图片
- **THEN** 出现 toast 且不插入附件

## MODIFIED Requirements

### Requirement: 左侧项目/会话/模型配置

系统 SHALL 在左侧栏展示项目与会话列表；模型与 API Key 的详细配置 MUST 位于「模型与密钥」Drawer，侧栏仅展示摘要。模型配置在草稿态与空会话时可编辑（经 Drawer）；会话已有 user/assistant 消息后为只读。侧栏 MUST 保留「新建」会话按钮，新建时不自动触发 starter。

#### Scenario: 侧栏精简

- **WHEN** 用户打开侧栏
- **THEN** 项目与会话列表可见，完整模型下拉与 Key 表单不在侧栏主区域展开

#### Scenario: 在 Drawer 切换会话与配置

- **WHEN** 用户在空会话通过 Drawer 切换模型
- **THEN** 配置持久化且侧栏摘要更新

#### Scenario: 有消息会话模型只读

- **WHEN** 当前会话已有 user 或 assistant 消息
- **THEN** Drawer 内模型与思考配置以只读形式展示，不可修改

### Requirement: 中间区 Markdown 流式渲染

系统 SHALL 在中间区以良好的 Markdown 渲染展示会话与结果，支持流式增量更新、代码高亮与表格；思考内容与正文分区展示。assistant 消息的**流式预览**与**持久化展示** MUST 使用同一消息气泡结构（思考可折叠区 + 正文 Markdown 区），仅允许样式 variant（如边框/动效）区分「生成中」与「已完成」。多轮工具调用时，每一步 LLM 的流式预览 MUST 独立呈现，不得将多步思考/正文累加在同一流式气泡中。收到 `assistant_step_done` 后，该步 assistant MUST 立即出现在消息列表中，并清空当前 streaming 缓冲；`turn_complete` 时仍可全量 `list_messages` 对齐，但 MUST NOT 导致 assistant 消息条数或内容与逐步展示结果发生可见冲突。user 消息若含图片附件，MUST 在文本旁展示缩略图。

#### Scenario: 用户消息展示图片附件

- **WHEN** 历史消息含 `attachments_json` 指向 `.uploads/photo.png`
- **THEN** 消息气泡展示该图缩略图与文本内容
