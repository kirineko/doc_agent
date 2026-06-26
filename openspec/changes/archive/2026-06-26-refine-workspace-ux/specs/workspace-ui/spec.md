## ADDED Requirements

### Requirement: 侧栏顶部动作区

系统 SHALL 在左侧栏顶部提供固定动作区，包含：**新建会话**（含快捷键提示 ⌘N / Ctrl+N）与 **搜索**（含 ⌘K / Ctrl+K 提示）。动作区 MUST 位于项目树之上，不使用全宽 primary 按钮样式。

#### Scenario: 新建会话在 active 项目下创建

- **WHEN** 用户已选 active 项目并点击侧栏「新建会话」或按下 ⌘N / Ctrl+N
- **THEN** 系统在该 active 项目下创建会话、选中新会话，且不自动触发 starter

#### Scenario: 无项目时新建被引导

- **WHEN** 用户无 active 项目并点击「新建会话」
- **THEN** 系统展示引导选择/添加项目目录，并高亮侧栏添加项目入口；MUST NOT 创建 orphan 会话

#### Scenario: 搜索打开命令面板

- **WHEN** 用户点击侧栏「搜索」
- **THEN** 打开命令面板（与 ⌘K / Ctrl+K 为同一组件）

### Requirement: 侧栏项目折叠组导航

系统 SHALL 在左侧栏以 **项目 → 会话** 树形结构展示导航：每个项目为一组，其下缩进展示该项目会话列表。**任意时刻 MUST 仅展开 active 项目**；非 active 项目 MUST 以折叠单行展示。项目组 MUST NOT 使用全宽 `btn-primary`「选择目录创建项目」长按钮；添加项目 MUST 使用 ghost 文字链或 list item 风格入口（如「＋ 添加项目目录」）。

#### Scenario: 仅 active 项目展开

- **WHEN** 用户选中项目 A 且项目 B 存在于列表
- **THEN** A 的会话列表可见，B 折叠为单行项目名

#### Scenario: 切换项目切换展开态

- **WHEN** 用户点击项目 B 行（非会话行）
- **THEN** active 项目切换为 B，B 展开会话列表，A 折叠

#### Scenario: 点击会话不切换项目

- **WHEN** 用户点击项目 A 下某会话
- **THEN** 选中该会话并加载消息，active 项目仍为 A

#### Scenario: 项目内新建会话

- **WHEN** 用户点击项目 A 行 hover 出现的 `[+]` 新建入口
- **THEN** 若 A 非 active 则先切换 active 为 A，再在 A 下创建并选中新会话

#### Scenario: 空项目组弱提示

- **WHEN** active 项目下无任何会话
- **THEN** 项目组内展示一行弱提示（如「暂无会话」），用户仍可通过新建会话或 ⌘N 创建

#### Scenario: 添加项目为 ghost 入口

- **WHEN** 用户查看侧栏项目区
- **THEN** 「添加项目目录」以 ghost/list 风格呈现，MUST NOT 占据全宽 primary 按钮位

### Requirement: 项目行上下文菜单

系统 SHALL 为侧栏每个项目行提供 `···` 上下文菜单（或等效 overflow 入口），至少包含：**在文件夹中打开**（macOS 文案可为「在 Finder 中打开」）与 **从列表移除**。打开动作 MUST 调用既有 `open_project_root`（或等价 IPC）在系统文件管理器中打开该项目根目录。移除 MUST 复用既有 hide project 行为。

#### Scenario: 在 Finder 中打开项目根

- **WHEN** 用户在项目 A 的上下文菜单选择「在 Finder 中打开」（或平台等效文案）
- **THEN** 系统打开 A 的 `root_path` 于系统文件管理器

#### Scenario: 从列表移除

- **WHEN** 用户选择「从列表移除」
- **THEN** 项目从侧栏消失；若其为 active 则切换到下一项目或空态

#### Scenario: 菜单不阻断项目切换

- **WHEN** 用户点击项目行主体（非 `···`）
- **THEN** 切换 active 项目，不打开菜单

### Requirement: Composer 上下文条

系统 SHALL 在 Chat 输入区（composer）**上方**展示上下文条，集中呈现：当前 **项目名**（可切换）、**模型选择与摘要 trigger**、**AGENTS.md 状态**、**上下文占用 %**。模型详细配置（Model Flyout）MUST **仅**从上下文条的模型 trigger 打开；侧栏 MUST NOT 再提供模型选择入口。顶栏 MUST NOT 重复展示 active 项目名副标题。上下文条在空态居中布局与 chat 底部 dock 布局 MUST 均可见（有项目时）。

#### Scenario: 顶栏不重复项目名

- **WHEN** 用户已选项目 doc_test
- **THEN** 顶栏仅展示品牌与全局入口；项目名出现在 composer 上下文条与侧栏树

#### Scenario: 模型选择仅在上下文条

- **WHEN** 用户已选项目并查看 composer
- **THEN** 上下文条展示可点击的模型摘要 trigger；侧栏无「模型」区块或 Flyout trigger

#### Scenario: 上下文条展示占用比例

- **WHEN** 当前会话有 context_usage 数据
- **THEN** 上下文条展示图标 + 百分比，规则同原会话标题栏 ContextUsageIndicator

#### Scenario: 无项目时隐藏项目与模型段

- **WHEN** 用户未选项目
- **THEN** 上下文条不展示项目切换段与模型 trigger；发送阻断逻辑不变

### Requirement: 空态居中 Composer

当 active 项目下当前上下文 **无 user/assistant 消息** 且无进行中的 streaming 预览时，系统 SHALL 将 composer（含上下文条、输入框、工具栏、推荐问/Init 胶囊）以 **垂直居中** 方式布局于中间区，最大宽度约 720px 并水平居中。当首条 user 或 assistant 消息出现（或 streaming 开始）后，composer MUST **过渡** 至底部 dock 布局；过渡 MUST NOT 清空输入框内容、附件或 cursor 位置。

#### Scenario: 空态居中展示

- **WHEN** 用户已选项目且当前会话无 chat 消息
- **THEN** composer 居中展示，上方可有问候语，下方展示 Init 胶囊（满足条件时）与「或直接输入开始对话」弱提示

#### Scenario: 有消息后 dock 底部

- **WHEN** 用户发送首条消息或 assistant 首条消息持久化展示
- **THEN** composer 位于中间区底部，消息列表占据上方滚动区

#### Scenario: 布局切换保留草稿

- **WHEN** 用户在空态居中 composer 中输入文字后发送首条消息
- **THEN** 发送内容不变，布局切换后输入框清空行为与现网一致（仅发送后清空）

#### Scenario: 切换会话空态仍居中

- **WHEN** 用户切换到无消息的空会话
- **THEN** 中间区恢复空态居中 composer

### Requirement: 右侧 Inspector 三 Tab

系统 SHALL 将右侧栏改为 **单一 Inspector**，顶部以 segmented control（或等效 Tab）切换三个视图：**项目文件**、**工具调用链**、**构建产物**；同一时刻仅展示其一，内容区占据右栏剩余全高。**默认 Tab MUST 为「项目文件」**。构建产物 Tab MUST 显示本轮产物数量徽标（无产物时为 0 或不显示）。三 Tab 切换 MUST NOT 影响主三栏水平宽度与拖拽语义。

#### Scenario: 默认展示项目文件

- **WHEN** 用户进入工作区或首次打开右栏 Inspector
- **THEN** 选中「项目文件」Tab 并展示 ProjectFileExplorer

#### Scenario: 切换至工具调用链

- **WHEN** 用户点击「工具调用链」Tab
- **THEN** 展示 ToolChainPanel 内容，文件列表隐藏

#### Scenario: 构建产物徽标

- **WHEN** 当前 session 本轮累积 2 个产物
- **THEN** 「构建产物」Tab 显示徽标 `2`

#### Scenario: 无 vertical 分割

- **WHEN** 用户查看右侧栏
- **THEN** MUST NOT 同时上下分栏展示「工具链区 + 文件区」；仅 Tab 切换

### Requirement: Inspector 智能 Tab 切换

系统 SHALL 在当前 turn 内，当 active session 出现 **首个** 工具调用进入 executing 态（`ToolCall { status: running }` 或等效 streaming 占位升级为 running）时，**自动**将 Inspector Tab 切换为「工具调用链」，**除非**用户在本 turn 内曾手动切换过 Inspector Tab（user pin）。用户发送新消息开始新 turn 时 MUST 清除 user pin。构建产物累积 MUST NOT 触发自动 Tab 切换（仅更新徽标）。

#### Scenario: 工具开始执行自动切 Tab

- **WHEN** 用户当前在「项目文件」Tab 且 Agent 首个工具进入 running
- **THEN** Inspector 自动切换至「工具调用链」

#### Scenario: 用户手动切换后本 turn 不自动切

- **WHEN** 本 turn 内用户曾手动切至「项目文件」
- **THEN** 后续工具 running MUST NOT 再次自动切换 Tab，直至用户发送下一条消息开始新 turn

#### Scenario: 新 turn 清除 pin

- **WHEN** 用户发送新用户消息开始新 turn
- **THEN** user pin 清除；若再有工具 running 可再次 auto-switch

#### Scenario: 产物不触发 auto-switch

- **WHEN** 本轮首个产物路径写入 turnArtifacts
- **THEN** 徽标更新，Inspector Tab 保持用户当前选择

### Requirement: 命令面板

系统 SHALL 提供全局命令面板，通过 **⌘K**（macOS）或 **Ctrl+K**（Windows/Linux）及侧栏搜索入口打开。面板 MUST 支持 fuzzy 搜索并分组展示：**快捷操作**（新建会话、添加项目目录等）、**项目**、**会话**（含所属项目上下文）、**斜杠命令**（id / label / description）。选中项目 MUST 切换 active 项目；选中会话 MUST 切换 active 会话（必要时先切换项目）；选中斜杠命令 MUST 向 composer 插入 prompt（同键盘 `/` 与图形菜单，MUST NOT 自动发送）。Esc MUST 关闭面板；↑↓ Enter MUST 导航与确认。

#### Scenario: 快捷键打开

- **WHEN** 用户按下 ⌘K 或 Ctrl+K
- **THEN** 命令面板 modal 打开并 focus 搜索输入

#### Scenario: 搜索并切换会话

- **WHEN** 用户输入会话标题关键词并选中某会话
- **THEN** 切换到该会话所属项目（若需要）并加载该会话

#### Scenario: 选中斜杠命令插入 prompt

- **WHEN** 用户选中 `/word:edit`
- **THEN** composer 填入对应 prompt，首个占位符选中，消息未发送

#### Scenario: 新建会话快捷操作

- **WHEN** 用户在面板选择「新建会话」
- **THEN** 行为同侧栏新建会话（active 项目下创建）

#### Scenario: 添加项目快捷操作

- **WHEN** 用户选择「添加项目目录」
- **THEN** 打开系统目录选择对话框并创建项目（同现网 pickProject）

### Requirement: Notion 风格工作区视觉

系统 SHALL 强化 Notion 风格（尤其 light 主题）：侧栏导航项 **MUST NOT** 使用重 border 卡片包裹每条项目/会话；改用 hover 浅底与 active 左侧 accent 条。工作区三栏 **SHOULD** 减少「卡片套卡片」嵌套（侧栏/主区/右栏优先 flat 分隔线而非外圈大圆角 panel 间距）。Composer **SHOULD** 使用较大圆角（约 12–16px）与轻 shadow。侧栏 **MUST NOT** 使用大写 `tracking` 分区标题（「项目」「会话」）作为唯一层级手段；以缩进与字重区分项目与会话。

#### Scenario: 会话项 Notion 风 hover

- **WHEN** 用户 hover 侧栏某会话行
- **THEN** 展示浅灰 hover 背景，无独立卡片边框

#### Scenario: 三栏 flat 布局

- **WHEN** 用户查看主工作区
- **THEN** 栏间以分隔线或背景差区分，MUST NOT 三层以上嵌套圆角 panel 造成大量留白

#### Scenario: Composer 圆角阴影

- **WHEN** 用户查看 chat composer
- **THEN** 输入 composite 具备 Notion AI 风格的圆角与浅阴影

## MODIFIED Requirements

### Requirement: 三栏工作区布局

系统 SHALL 提供三栏布局：左侧为项目 / 会话树与 Web 搜索等辅助配置，中间为会话与结果（含 composer 上下文条中的模型选择），右侧为 **Inspector**（项目文件、工具调用链、构建产物三 Tab 切换）。三栏之间 MUST 支持水平拖拽调整宽度。中间会话区 MUST 具有最小宽度约束，防止被完全挤压。左侧栏宽度 MUST 可拖拽调整。右侧栏 MUST NOT 再使用工具链与文件浏览的 **上下垂直分栏**。

#### Scenario: 三栏同时可见

- **WHEN** 用户打开一个项目的会话
- **THEN** 界面同时呈现左侧导航与配置、中间会话区、右侧 Inspector 区域

#### Scenario: 水平拖拽调整侧栏与右侧栏宽度

- **WHEN** 用户拖拽侧栏与中间区、或中间区与右侧栏之间的分割条
- **THEN** 对应栏宽度按比例实时调整，松手后新比例生效

#### Scenario: 会话区最小宽度

- **WHEN** 用户持续收窄中间会话区
- **THEN** 会话区宽度 MUST NOT 低于预设最小比例，分割条停止继续向内挤压

### Requirement: 左侧项目/会话/模型配置

系统 SHALL 在左侧栏以 **项目折叠组 → 会话** 树展示导航。侧栏 MUST 在顶部动作区提供「新建会话」（非仅会话分区小按钮）。新建时不自动触发 starter。侧栏 MUST NOT 包含模型选择 trigger 或「模型」配置区块；模型配置 MUST 迁至 composer 上下文条（见「Composer 上下文条」「模型 Flyout 锚定 Composer 上下文条」）。LLM API Key 配置 MUST NOT 出现在侧栏或 Model Flyout 内。Web 搜索状态摘要仍位于侧栏底部。

#### Scenario: 侧栏树形导航

- **WHEN** 用户打开侧栏
- **THEN** 可见项目折叠组与会话层级，而非平级「项目」「会话」两个独立分区列表

#### Scenario: 侧栏无模型入口

- **WHEN** 用户打开侧栏（无论是否已选项目）
- **THEN** 侧栏 MUST NOT 展示模型 Flyout trigger 或模型摘要配置区

#### Scenario: 未选项目时模型在上下文条不可见

- **WHEN** 用户尚未选择 activeProject
- **THEN** composer 上下文条不展示模型 trigger；Header 密钥入口仍可用

#### Scenario: 有消息会话模型只读

- **WHEN** 当前会话已有 user 或 assistant 消息
- **THEN** 自上下文条打开的 Model Flyout 以只读形式展示当前模型与思考配置，不可修改

### Requirement: 模型 Flyout 锚定侧栏

系统 SHALL 在用户点击 **composer 上下文条**内的模型摘要 trigger 时，于 trigger **附近**（优先向上展开，避免遮挡输入框）显示固定定位 Flyout，而非屏幕右侧全高 Drawer。Flyout MUST 包含：当前模型摘要、Provider segmented control、可滚动模型列表、底部 sticky 思考配置。模型配置在已选项目且（草稿态或空会话）时可编辑；会话已有 user/assistant 消息后 Flyout 为只读。切换 Provider Tab 时 MUST 预选该 Provider 的第一个可用模型（与现有 `configForProviderFirstModel` 行为一致）。Flyout 水平宽度 MUST 与上下文条内模型 trigger 同宽（或不低于合理最小宽度如 280px，随 composer 宽度变化实时更新）。侧栏 MUST NOT 再提供模型 Flyout trigger。

#### Scenario: Flyout 靠近上下文条 trigger

- **WHEN** 用户点击 composer 上下文条内的模型 trigger
- **THEN** Flyout 在 trigger 上方或下方展开，水平对齐 trigger，不要求用户视线移至侧栏

#### Scenario: Flyout 宽度随 composer 自适应

- **WHEN** 用户拉宽中间会话区且 Model Flyout 处于打开状态
- **THEN** Flyout 宽度 MUST 与模型 trigger 同宽或保持可读最小宽度，不得错误锚定于侧栏宽度

#### Scenario: Flyout 不含 Key 配置

- **WHEN** 用户打开模型 Flyout
- **THEN** 界面不包含任何 API Key 输入控件

#### Scenario: Provider Tab 预选首模型

- **WHEN** 用户在 Flyout 切换至 Kimi Provider Tab 且会话模型未锁定
- **THEN** pending/空会话模型切换为该 Provider 列表中的第一个模型

#### Scenario: 上下文条摘要含 vision 标识

- **WHEN** 当前选中 Kimi K2.6
- **THEN** composer 上下文条模型 trigger 摘要显示模型名与视觉能力标识

#### Scenario: 在 Flyout 切换模型

- **WHEN** 用户在已选项目的空会话通过 composer 上下文条 Flyout 切换模型
- **THEN** 配置持久化且上下文条模型摘要立即更新

#### Scenario: 侧栏无模型 Flyout

- **WHEN** 用户查看侧栏任意状态
- **THEN** MUST NOT 存在侧栏模型 trigger 或 `#sidebar-model-trigger`

### Requirement: 项目列表展示与隐藏交互

左侧项目树 SHALL 为每个项目行提供足够点击区域与 hover 移除/菜单入口；移除交互 MAY 位于上下文菜单内（不必强制 hover 行内 × 按钮）。系统 MUST NOT 提供已隐藏项目的管理入口。项目根路径 MAY 在 tooltip 或二级信息中展示，不必每条会话重复全路径。

#### Scenario: 从菜单移除项目

- **WHEN** 用户通过项目上下文菜单选择「从列表移除」
- **THEN** 该项目立即从侧栏树消失

#### Scenario: hover 会话显示删除

- **WHEN** 用户 hover 会话项
- **THEN** 仍可按现网规则展示删除会话控件

### Requirement: 工作区空状态弱引导

系统 SHALL 在已选项目且无 chat 消息的中间区通过 **空态居中 composer** 提供弱引导：可选 Init 胶囊（满足条件时）及「或直接输入开始对话」；MUST NOT 使用常驻 hint 条占用输入区。无消息时中间区 MUST NOT 仅为空白 panel 加底部 dock 输入框。

#### Scenario: 空状态居中引导

- **WHEN** 用户已选项目且当前上下文无 user/assistant 消息
- **THEN** 用户于居中 composer 可见 Init 胶囊（若适用）与直输提示，感知两种开始方式

### Requirement: 工作区分栏布局持久化

系统 SHALL 将 **主三栏水平比例** 与 **Inspector 当前 Tab** 持久化于前端 `localStorage`，应用重启后恢复用户上次选择。首次访问或无有效缓存时 MUST 使用默认水平比例；Inspector 默认 Tab MUST 为「项目文件」。布局持久化 MUST NOT 与 `doc-agent-last-session-config` 混用同一存储键。系统 MUST NOT 再持久化右侧 **上下垂直** 分栏比例。

#### Scenario: 拖拽后持久化水平布局

- **WHEN** 用户调整主三栏水平分割条并松手
- **THEN** 新比例写入 localStorage

#### Scenario: Inspector Tab 持久化

- **WHEN** 用户切换 Inspector 至「工具调用链」并重启应用
- **THEN** 恢复该 Tab 选中态

#### Scenario: 无缓存时默认文件 Tab

- **WHEN** 用户首次打开应用
- **THEN** Inspector 默认「项目文件」Tab

#### Scenario: 与会话模型配置存储隔离

- **WHEN** 系统读写布局持久化数据
- **THEN** MUST NOT 修改 `doc-agent-last-session-config`

### Requirement: 构建产物 Tab 视图

系统 SHALL 在右侧 Inspector 内提供「构建产物」Tab（与「项目文件」「工具调用链」并列），同一时刻仅展示其一。「构建产物」Tab 标题 MUST 显示本轮产物数量徽标。Tab 切换 MUST NOT 影响主三栏水平布局。

#### Scenario: 默认不在构建产物 Tab

- **WHEN** 用户进入工作区且 Agent 未运行
- **THEN** Inspector 默认「项目文件」Tab；「构建产物」可见，徽标为 0 或不显示

#### Scenario: Tab 徽标反映本轮产物数

- **WHEN** Agent 在本轮累积产生 2 个去重后的产物路径
- **THEN** 「构建产物」Tab 标题显示徽标 `2`

#### Scenario: 切换 Tab 不影响主布局

- **WHEN** 用户在 Inspector 三 Tab 间切换
- **THEN** 主三栏宽度与中间会话区不受影响

### Requirement: 本轮构建产物列表

系统 SHALL 在 Inspector「构建产物」Tab 内展示「本轮」Agent 产生或修改的项目相对路径列表。产物列表 MUST 按当前 turn 累积：新 turn 开始时清空。产物路径 MUST 去重。每个产物项 SHALL 标注其来源工具调用（工具中文名）。无产物时 MUST 展示空态文案。产物状态 MUST 按 `session_id` 维护于前端内存；切换 `activeSessionId` 时 MUST 恢复该 session 的 `turnArtifacts`，MUST NOT 写入数据库或 `localStorage`。

#### Scenario: 累积本轮产物

- **WHEN** Agent 在本轮产出 `report.docx` 与 `notes.md`
- **THEN** 「构建产物」Tab 列出路径项并标注来源工具

#### Scenario: 新 turn 清空产物

- **WHEN** 用户发送新用户消息开始新 turn
- **THEN** 产物列表清空，徽标归零

#### Scenario: 切换会话保留产物

- **WHEN** session A 有本轮产物，用户切到 B 再切回 A
- **THEN** A 的产物列表与徽标恢复

### Requirement: 构建产物打开与定位

系统 SHALL 为「构建产物」列表项提供两种动作：用默认程序打开文件、在系统文件管理器中定位该文件。打开动作 MUST 复用既有文件打开能力；定位动作 MUST 打开系统文件管理器并尽量定位到该文件。两个动作 MUST 仅对项目根目录内的路径生效。

#### Scenario: 用默认程序打开产物

- **WHEN** 用户在产物列表中点击「打开」
- **THEN** 系统以默认关联程序打开该文件

#### Scenario: 在文件管理器中定位产物

- **WHEN** 用户点击「在文件夹中显示」
- **THEN** 系统打开文件管理器并定位到该路径

#### Scenario: 拒绝越界路径

- **WHEN** 产物路径不在项目根内
- **THEN** 打开与定位均失败

### Requirement: 上下文占用比例展示

系统 SHALL 在 **composer 上下文条**（中间区输入区上方）以最小化形式展示当前上下文占用比例：仅图标 + 比例百分比值（如 `42%`），MUST NOT 展示 token 绝对值。比例数据来源为 `context_usage` 与 `get_session_context_usage`；无 LLM 调用历史时 MUST 显示 `0%`（仅无项目时隐藏）。

#### Scenario: 展示当前占用比例

- **WHEN** 当前会话已发生 LLM 响应且收到 `context_usage`
- **THEN** composer 上下文条显示图标 + 百分比

#### Scenario: 切换会话重置比例

- **WHEN** 用户切换会话
- **THEN** 通过 `get_session_context_usage` 展示该会话比例

#### Scenario: 空会话展示零比例

- **WHEN** 用户切换到尚无 LLM 调用的空会话
- **THEN** 指示器显示 `0%`

#### Scenario: 接近上限的视觉提示

- **WHEN** ratio 接近上限
- **THEN** 指示器以橙/红等醒目颜色提示

### Requirement: AGENTS.md status indicator

The workspace UI SHALL indicate whether the active project has a non-empty `AGENTS.md` at project root. The indicator SHALL appear in the **composer context bar** (in addition to or instead of the chat panel title row).

#### Scenario: Indicator reflects file presence

- **WHEN** the user selects a project with `AGENTS.md` present
- **THEN** the composer context bar shows a visible indicator

#### Scenario: Indicator hidden or muted when missing

- **WHEN** the active project has no `AGENTS.md` or an empty file
- **THEN** the indicator reflects missing/empty state without blocking chat

## REMOVED Requirements

### Requirement: 右侧工具调用链可视化

**Reason**: 右侧栏改为 Inspector 三 Tab 统一容器；工具链改为 Tab 内全高展示，不再使用上下分栏与互斥折叠语义。

**Migration**: 工具链展示、贴底滚动、卡片状态更新由「右侧 Inspector 三 Tab」「Inspector 智能 Tab 切换」及既有 tool chain 组件行为覆盖；垂直分割条与折叠互斥 Scenario 废弃。

#### Scenario: 展示工具调用进展

- **WHEN** Agent 发起并完成一个工具调用
- **THEN** 用户在 Inspector「工具调用链」Tab 内看到对应卡片状态更新（本 Scenario 迁移至 Inspector Tab 上下文，Requirement 本体移除）

#### Scenario: 工具调用链自动贴底滚动

- **WHEN** Agent 执行中向工具调用链追加新卡片
- **THEN** 工具链 Tab 内容区在贴底状态下自动滚动（行为保留于 ToolChainPanel 实现，Requirement 本体移除）
