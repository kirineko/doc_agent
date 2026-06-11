## ADDED Requirements

### Requirement: 草稿态输入与懒创建会话
系统 SHALL 在用户已选择项目但尚无 activeSession（草稿态）时保持消息输入框可用；用户发送消息时 MUST 先自动创建会话（使用当前 pending 模型配置，默认 DeepSeek V4 Flash + thinking enabled + effort high），再发送该消息，且 MUST NOT 触发首次推荐问（starter）生成。

#### Scenario: 草稿态直接发送
- **WHEN** 用户已选项目、无 activeSession，在输入框输入内容并发送
- **THEN** 系统创建新会话、选中该会话、发送消息，且不进入 starter 初始化状态

#### Scenario: 草稿态可正常输入
- **WHEN** 用户已选项目但无 activeSession
- **THEN** 输入框可用（非 busy/initializing 时），且不展示阻断性常驻 hint

### Requirement: 显式初始化胶囊
系统 SHALL 在用户已选项目、当前上下文无 user/assistant 消息、已配置 DeepSeek API Key、且非 busy/initializing 时，于中间空状态区展示「初始化推荐」胶囊入口；用户点击后 MUST 创建或选中会话并触发 starter 推荐问生成。未配置 DeepSeek Key 时 MUST NOT 展示该胶囊。

#### Scenario: 点击胶囊触发初始化
- **WHEN** 用户在有项目的草稿态或空会话中点击初始化胶囊
- **THEN** 系统确保存在 activeSession、进入 initializing 状态并生成 starter 推荐问

#### Scenario: 无 DeepSeek Key 不展示胶囊
- **WHEN** 用户未配置 DeepSeek API Key
- **THEN** 中间区不展示初始化胶囊，且无 starter 相关 LLM 调用

#### Scenario: 直接发送跳过 starter
- **WHEN** 用户未点击初始化胶囊而直接发送首条消息
- **THEN** 系统不触发 starter，消息正常进入对话流

### Requirement: 发送阻断一次性引导
系统 SHALL 在用户尝试发送（点击发送或 Enter）但因缺少前置条件而失败时，展示一次性、非常驻的引导提示，且 MUST NOT 清空输入框已有内容。缺少前置条件包括：未选择项目、当前模型对应 provider 未配置 API Key。

#### Scenario: 未选项目时发送
- **WHEN** 用户未选择项目但输入框有内容并尝试发送
- **THEN** 展示提示引导选择/创建项目，并高亮左侧项目区，输入内容保留

#### Scenario: 无 Key 时发送
- **WHEN** 用户已选项目且输入框有内容，但当前模型 provider 的 API Key 未配置，并尝试发送
- **THEN** 展示提示引导配置 Key，展开对应 Key 输入区，输入内容保留

### Requirement: 切换项目与会话选择
系统 SHALL 在用户切换项目时自动选中该项目按 `updated_at` 排序的最近一条会话；若该项目无任何会话则进入草稿态（无 activeSession）。切换项目时 MUST NOT 清空消息输入框内容。

#### Scenario: 切换到有会话的项目
- **WHEN** 用户从项目 A 切换到项目 B，且 B 存在历史会话
- **THEN** 自动选中 B 的最近会话并加载其消息，输入框内容不变

#### Scenario: 切换到无会话的项目
- **WHEN** 用户切换到尚无会话的项目
- **THEN** activeSession 为空、中间区为空白草稿态，输入框内容不变

### Requirement: 工作区空状态弱引导
系统 SHALL 在已选项目且无 chat 消息的中间区展示简洁空状态：可选初始化胶囊（满足条件时）及一行弱提示「或直接输入开始对话」；MUST NOT 使用常驻 hint 条占用输入区。

#### Scenario: 空状态展示
- **WHEN** 用户已选项目且当前上下文无 user/assistant 消息
- **THEN** 中间区非完全空白，用户可感知两种开始方式（初始化或直输）

## MODIFIED Requirements

### Requirement: 会话初始化交互
当用户**点击初始化胶囊**且首次会话推荐问生成进行中时，界面 SHALL 进入「会话初始化中」状态：禁用消息输入框并展示带动效的进度提示（如「正在阅读项目文档…」）；生成结束（无论成功失败）后 MUST 解锁输入框。打开空会话或新建会话 alone MUST NOT 自动进入该状态。

#### Scenario: 初始化期间输入禁用
- **WHEN** 用户点击初始化胶囊且推荐问生成请求进行中
- **THEN** 输入框与发送按钮禁用，会话区显示初始化进度提示

#### Scenario: 失败也解锁
- **WHEN** 推荐问生成失败或超时
- **THEN** 进度提示消失、输入框解锁，不展示错误干扰用户

#### Scenario: 新建会话不自动初始化
- **WHEN** 用户通过侧栏「新建」创建空会话
- **THEN** 不进入 initializing 状态，输入框立即可用

### Requirement: 左侧项目/会话/模型配置
系统 SHALL 在左侧栏展示项目与会话列表；API Key 配置 MUST 位于与会话无关的全局区域（项目区与会话区之间或等效位置），与模型配置分区展示。模型配置在草稿态与空会话时可编辑；会话已有 user/assistant 消息后为只读。侧栏 MUST 保留「新建」会话按钮，新建时不自动触发 starter。

#### Scenario: Key 与模型分区
- **WHEN** 用户打开侧栏
- **THEN** API Key 区域与模型选择区域分离展示，Key 不依赖 activeSession 才可见

#### Scenario: 在侧栏切换会话与配置
- **WHEN** 用户在左侧栏选择另一个空会话并切换模型 / 思考配置
- **THEN** 中间会话区切换为该会话内容，模型 / 思考配置随之更新并持久化

#### Scenario: 有消息会话模型只读
- **WHEN** 当前会话已有 user 或 assistant 消息
- **THEN** 侧栏模型与思考配置以只读形式展示，不可修改
