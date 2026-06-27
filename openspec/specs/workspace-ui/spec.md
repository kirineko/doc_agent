# workspace-ui Specification

## Purpose
TBD - created by archiving change bootstrap-doc-agent-mvp. Update Purpose after archive.
## Requirements
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
- **THEN** 消息气泡展示「无法加载」缩略图占位，文本内容仍可见

### Requirement: Header 密钥与设置双入口
系统 SHALL 在应用顶栏右侧提供两个并列 icon 入口：**密钥**（打开「密钥与服务」Drawer）与 **设置**（打开现有设置 Drawer）。两入口 MUST 使用一致的按钮尺寸与边框样式（与现有设置按钮一致）。

#### Scenario: 密钥入口始终可见
- **WHEN** 用户打开应用
- **THEN** Header 显示密钥按钮，不依赖是否已选项目

#### Scenario: 设置入口保持独立
- **WHEN** 用户点击设置按钮
- **THEN** 打开设置 Drawer（版本、布局、余额），不包含 API Key 表单

### Requirement: 密钥与服务 Drawer
系统 SHALL 提供从 Header 密钥按钮打开的右侧 Drawer（标题「密钥与服务」），包含：（1）LLM 分区 — DeepSeek、Kimi、MiMo API Key；（2）搜索服务分区 — Tavily API Key。各 Key MUST 支持保存、更换、清空；已保存 Key 默认折叠摘要展示。

#### Scenario: 打开密钥 Drawer
- **WHEN** 用户点击 Header 密钥按钮
- **THEN** 右侧 Drawer 展示 LLM 与 Tavily Key 配置，不展示模型选择

#### Scenario: 高亮缺 Key 的 Provider
- **WHEN** 系统因发送拦截打开密钥 Drawer 并指定 provider
- **THEN** 对应 Provider 的 Key 行展开并视觉高亮

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

### Requirement: 零 LLM Key 启动弱提醒
当尚未配置**任一** LLM Provider（DeepSeek/Kimi/MiMo）API Key 时，系统 SHALL 在每次应用启动后于 Header 区域展示非阻塞弱提醒条（单行 muted 样式），文案说明发送前需配置 Key，并提供「去配置」打开密钥 Drawer。密钥 Header 按钮 MUST 显示 amber 状态 dot。弱提醒 MUST NOT 使用 modal；用户关闭当次提醒条后，下次启动仍 MUST 再次显示（不持久 dismiss）。

#### Scenario: 每次启动显示弱提醒
- **WHEN** 用户启动应用且 `API_PROVIDERS` 中无任何 provider 已配置 Key
- **THEN** Header 显示弱提醒条与密钥按钮 amber dot

#### Scenario: 配置任一 LLM Key 后隐藏
- **WHEN** 用户已配置至少一个 LLM Provider Key
- **THEN** 弱提醒条与 amber dot 不再显示

#### Scenario: 仅缺 Tavily 不触发
- **WHEN** 用户已配置 DeepSeek Key 但未配置 Tavily Key
- **THEN** 不显示 LLM Key 弱提醒条（Tavily 不阻断发送）

#### Scenario: 关闭提醒不影响下次启动
- **WHEN** 用户在当次会话中关闭弱提醒条但未配置 Key，随后重启应用
- **THEN** 弱提醒条再次显示

### Requirement: 非 vision 粘贴 Toast

当用户在非 vision 模型下粘贴图片时，系统 SHALL 展示非阻塞 toast，文案说明需切换至支持视觉的模型（Kimi K2.6 或 MiMo v2.5）。

#### Scenario: DeepSeek 下粘贴图片

- **WHEN** 会话模型为 DeepSeek V4 Flash 且用户粘贴图片
- **THEN** 出现 toast 且不插入附件

### Requirement: 项目列表展示与隐藏交互

左侧项目树 SHALL 为每个项目行提供足够点击区域与 hover 移除/菜单入口；移除交互 MAY 位于上下文菜单内（不必强制 hover 行内 × 按钮）。系统 MUST NOT 提供已隐藏项目的管理入口。项目根路径 MAY 在 tooltip 或二级信息中展示，不必每条会话重复全路径。

#### Scenario: 从菜单移除项目

- **WHEN** 用户通过项目上下文菜单选择「从列表移除」
- **THEN** 该项目立即从侧栏树消失

#### Scenario: hover 会话显示删除

- **WHEN** 用户 hover 会话项
- **THEN** 仍可按现网规则展示删除会话控件

### Requirement: @ 文件引用选择器
输入框 SHALL 支持 `@` 触发的文件引用：检测到光标前的 `@` 及其后查询串时，弹出项目内文件/目录候选列表，支持 fzf 式模糊匹配（子序列匹配 + 评分排序 + 命中高亮）、键盘上下选择与确认；确认后在输入框插入 `@相对路径`（含空白或解析终止符的路径 MUST 以引号包裹）。文件清单 MUST 限制遍历深度与数量并忽略隐藏目录/依赖目录/Office 临时文件；**此外 MUST 忽略 OOXML 解压工作目录（路径段名为 `unpacked` 或以 `_unpacked` 结尾的目录）及其全部子树**。清单 MUST 按修改时间降序索引，并携带 `is_dir` 供弹层区分目录与文件。清单 MUST 在项目文件变更后更新：优先通过 `tool_result.changed_paths` 增量合并，并在每个 turn 完成时 debounce 全量刷新一次。

空 `@` query MUST 仅展示项目根目录直接子项；query 含 `/` 时 MUST 进入对应目录浏览其子项；全局搜索 MUST 按父目录分组展示，键盘选择顺序 MUST 与弹层渲染顺序一致。目录项 Tab MUST 进入子目录（append `路径/`），Enter MUST 确认引用；Esc MUST 仅关闭弹层，MUST NOT 删除输入内容。

`@` 文件引用弹层字号 MUST 与斜杠命令弹层一致（正文 `text-xs` 基线，不得使用小于 12px 的正文字号）。

#### Scenario: 模糊匹配与确认
- **WHEN** 用户在输入框键入 `@课程` 且项目内存在「课程体系.xlsx」
- **THEN** 弹层展示按匹配度排序的候选（含该文件），用户按 Enter 后输入框中 `@课程` 被替换为 `@课程体系.xlsx `

#### Scenario: Esc 关闭弹层
- **WHEN** 弹层展示中用户按 Esc
- **THEN** 弹层关闭，输入内容保持不变

#### Scenario: Tab 进入子目录
- **WHEN** 用户在根级 `@` 弹层高亮目录 `docs` 并按 Tab
- **THEN** 输入变为 `@docs/` 且弹层展示 `docs/` 下直接子项，消息未发送

#### Scenario: Agent 理解 @ 引用
- **WHEN** 用户发送包含 `@相对路径` 的消息
- **THEN** system prompt 已声明该语义，Agent MUST 可直接以该路径调用文件/文档工具读取

#### Scenario: 解压目录内部不出现在 @ 候选
- **WHEN** 项目内存在 `report_unpacked/` 或 `.../unpacked/...`
- **THEN** `@` 候选列表 MUST NOT 包含该路径或 `unpacked/` 下任意子路径
- **AND** 用户仍 MUST 可通过 `@` 引用同级的 `.docx` 成品文件

#### Scenario: Agent 新建文件后可 @ 引用
- **WHEN** Agent 在本 turn 写入 `summary.md`
- **THEN** turn 结束后用户在 `@` 中 MUST 可匹配并选中 `summary.md`

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

### Requirement: 推荐问展示交互
首次会话与 follow-up 推荐问 SHALL 均以胶囊按钮形式统一展示在输入框上方；点击任一推荐 MUST 将该文本填入输入框供用户编辑，不得直接发送。每条推荐问长度 MUST 不超过 80 个字符。follow-up 生成期间 MUST NOT 禁用输入框；用户先行发送消息或切换会话时，迟到的 follow-up 结果 MUST 被丢弃。

斜杠命令选择器与推荐问 MUST 共存：推荐问在输入框上方，斜杠弹层在 textarea 上方；二者互不阻断。

#### Scenario: 点击推荐填入输入框
- **WHEN** 用户点击一条推荐问胶囊
- **THEN** 该文本出现在输入框中且输入框获得焦点，推荐区在用户发送消息前仍可展示

#### Scenario: 迟到的 followup 被丢弃
- **WHEN** followup 推荐尚未返回时用户已手动发送了新消息
- **THEN** 返回的推荐结果被丢弃，不展示

#### Scenario: 斜杠与推荐问共存
- **WHEN** 空会话同时展示 starter 推荐问且用户在输入框键入 `/`
- **THEN** 推荐问胶囊仍可见，斜杠弹层在 textarea 上方正常展示

### Requirement: 斜杠命令选择器

输入框 SHALL 支持 `/` 触发的斜杠命令选择器：检测到光标前位于**行首或空白字符后**的 `/` 且 `/` 与光标之间无空白时，弹出命令候选列表；支持 fzf 式模糊匹配、按分类分组展示、键盘上下选择与确认。

确认后 MUST 将输入框中的 `/query` 替换为该命令的 `prompt` 文本，**MUST NOT** 自动发送消息；首个 `{{占位符}}` MUST 自动选中供编辑。行为 MUST 与「推荐问点击填入输入框」一致。

斜杠弹层与 `@` 文件引用弹层 MUST 互斥：二者同时满足触发条件时，仅展示 `@` 弹层。

澄清进行中（`activeClarify`）、busy、initializing 时 MUST NOT 展示斜杠弹层（与输入 disabled 一致）。

#### Scenario: 选择命令填入 prompt

- **WHEN** 用户输入 `/word` 并在弹层中选择「精准修改 Word」
- **THEN** 输入框中 `/word` 被替换为对应 prompt（含文件名与改动占位）
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

斜杠命令弹层 SHALL 按分类展示分组标题（通用、Word、PPT、Excel、PDF、Web），每条候选 MUST 展示命令 id、`label` 与一行 `description`；匹配字符 MUST 高亮（与 `@` 弹层同类样式）。

弹层 MUST 可滚动；样式 MUST 复用或延伸现有 `mention-popup` 设计令牌，与明暗主题一致。弹层正文字号 MUST 为 `text-xs`（12px）基线，与 `@` 弹层及斜杠图形菜单一致，不得使用小于 12px 作为正文主字号。

#### Scenario: 分组展示

- **WHEN** 用户输入 `/` 且无 query 过滤
- **THEN** 弹层按 general、word、ppt、excel、pdf、web 顺序展示分组

#### Scenario: 单行展示 id / label / description

- **WHEN** 弹层展示 `word:edit`
- **THEN** 同一行可见命令 id、`label` 与 `description`（过长时 truncate + title）

### Requirement: 应用品牌标识
系统 SHALL 使用定制 Logo 替换 Tauri 默认图标，并在顶栏标题旁展示 Logo 图形；窗口标题文案保持「Doc Agent」。

#### Scenario: 顶栏展示 Logo
- **WHEN** 用户打开应用主窗口
- **THEN** 顶栏左侧显示定制 Logo 与「Doc Agent」文字，而非仅纯文字或 Tauri 默认标识

#### Scenario: 安装包与窗口使用定制图标
- **WHEN** 用户安装或运行打包后的应用
- **THEN** 快捷方式、任务栏与 macOS Dock 显示定制图标，而非 Tauri 默认图标

### Requirement: 安装目录无空格
系统 SHALL 将打包产物的默认安装目录名设为 `DocAgent`（无空格）；用户可见窗口标题不受此约束。

#### Scenario: Windows 默认安装路径
- **WHEN** 用户在 Windows 上执行默认安装
- **THEN** 默认目标目录为 `DocAgent` 而非含空格的 `Doc Agent`

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
- **THEN** 展示提示引导配置 Key，打开 Header 密钥 Drawer 并高亮对应 Provider，输入内容保留

### Requirement: 切换项目与会话选择
系统 SHALL 在用户切换项目时自动选中该项目按 `updated_at` 排序的最近一条会话；若该项目无任何会话则进入草稿态（无 activeSession）。切换项目时 MUST NOT 清空消息输入框内容。

#### Scenario: 切换到有会话的项目
- **WHEN** 用户从项目 A 切换到项目 B，且 B 存在历史会话
- **THEN** 自动选中 B 的最近会话并加载其消息，输入框内容不变

#### Scenario: 切换到无会话的项目
- **WHEN** 用户切换到尚无会话的项目
- **THEN** activeSession 为空、中间区为空白草稿态，输入框内容不变

### Requirement: 工作区空状态弱引导

系统 SHALL 在已选项目且无 chat 消息的中间区通过 **空态居中 composer** 提供弱引导：可选 Init 胶囊（满足条件时）及「或直接输入开始对话」；MUST NOT 使用常驻 hint 条占用输入区。无消息时中间区 MUST NOT 仅为空白 panel 加底部 dock 输入框。

#### Scenario: 空状态居中引导

- **WHEN** 用户已选项目且当前上下文无 user/assistant 消息
- **THEN** 用户于居中 composer 可见 Init 胶囊（若适用）与直输提示，感知两种开始方式

### Requirement: 项目文件索引变更同步
系统 SHALL 在 Agent 成功执行文件变更类工具后，使 `@` 文件引用清单与资源管理器当前目录与磁盘保持一致；同步 MUST 采用事件驱动策略，禁止定时轮询项目目录。

#### Scenario: turn 结束后 @ 清单包含新文件
- **WHEN** Agent 在本 turn 内通过 `fs_write` 创建了 `docs/report.docx` 且 turn 正常结束
- **THEN** 用户在输入框键入 `@report` 时候选列表包含 `docs/report.docx`

#### Scenario: 增量路径即时可见
- **WHEN** Agent 某工具成功执行且 `tool_result` 携带 `changed_paths` 含 `notes.md`
- **THEN** 该路径在 turn 结束前即可出现在 `@` 候选数据源中（经前端 merge）

#### Scenario: 不使用定时轮询
- **WHEN** 用户保持项目打开且 Agent 处于空闲
- **THEN** 系统 MUST NOT 以固定间隔调用 `list_project_files_cmd`

### Requirement: 侧栏 Web 搜索配置区块
系统 SHALL 在侧栏提供「Web 搜索 (Tavily)」**状态摘要**（已启用 / 未启用），与会话、项目选择无关；侧栏 MUST NOT 包含 Tavily Key 输入表单。未启用时 MUST 提供跳转或文案引导用户至 Header 密钥 Drawer 配置 Tavily Key。

#### Scenario: 侧栏仅展示状态
- **WHEN** 用户打开侧栏
- **THEN** Web 搜索区块显示启用状态，不包含 Key 输入框

#### Scenario: 未配置时引导至密钥 Drawer
- **WHEN** Tavily Key 未配置且用户查看侧栏 Web 搜索区块
- **THEN** 显示「未启用」及可点击引导打开 Header 密钥 Drawer

#### Scenario: 已配置时低干扰展示
- **WHEN** Tavily Key 已保存
- **THEN** 侧栏以折叠摘要「已启用」展示，不展开 Key 表单

### Requirement: Web 工具中文标签
系统 SHALL 为 `web_search` 与 `web_extract` 提供中文工具链标签，并在工具名注册列表测试中保持同步。

#### Scenario: 工具卡片显示中文名
- **WHEN** Agent 调用 `web_search` 或 `web_extract`
- **THEN** 右侧工具链卡片显示对应中文标签（非原始英文名）

### Requirement: 图片下载工具中文标签
系统 SHALL 为 `image_download` 提供中文工具链标签（如「下载图片」），并在工具名注册列表测试（`toolLabels.test.ts` 的 `EXPECTED_TOOLS`）中保持同步。

#### Scenario: 工具链展示中文标签
- **WHEN** 右侧工具链渲染一次 `image_download` 调用卡片
- **THEN** 卡片显示对应中文标签（非原始英文名 `image_download`）

#### Scenario: 标签注册表与后端工具一致
- **WHEN** 运行前端工具标签测试
- **THEN** `REGISTERED_TOOL_NAMES` 包含 `image_download`，与后端 `default_tools` 工具名集合一致

### Requirement: 顶栏主题切换 Toggle

系统 SHALL 在应用顶栏右上角提供主题切换 **toggle** 控件，用于在 `dark` 与 `light` 两档主题间切换；该控件 MUST 位于顶栏最右侧（`Doc Agent` 品牌与项目名区域保持在左侧），且 MUST NOT 遮挡或替换现有 Logo 与标题展示。

#### Scenario: 顶栏右上角展示 Toggle

- **WHEN** 用户打开应用主窗口
- **THEN** 顶栏右侧可见主题 toggle，左侧仍显示定制 Logo 与「Doc Agent」文字

#### Scenario: 点击 Toggle 切换主题

- **WHEN** 用户点击顶栏主题 toggle
- **THEN** 应用主题在 `dark` 与 `light` 间立即切换，toggle 视觉状态对应当前主题

#### Scenario: Toggle 可访问性

- **WHEN** 辅助技术聚焦主题 toggle
- **THEN** 控件具备描述当前操作或目标主题的 accessible 名称（如 `aria-label`），且可通过键盘激活

### Requirement: 澄清问题交互卡片

系统 SHALL 在会话区展示 `ClarifyQuestionCard`。活跃卡片（pending 未答）数据源为 `clarify_question` 事件或 session 加载时 bundle 中 `status=awaiting_user` 的 clarify_ask 工具记录；卡片 MUST 渲染在消息列表底部（输入框上方）。卡片 MUST 支持四种 `kind`：

- `single`：选项按钮/卡片，选中高亮；`allow_custom` 时展示「其他」+ 输入框
- `multi`：多选 chip，校验 `min_selections`/`max_selections`；支持自定义追加
- `text`：textarea；可选快捷 chip 辅助填入
- `confirm_brief`：创作简报字段预览 +「确认继续」/「需要修改」；修改时展示 textarea

提交时前端仅发送 `selected` 与 `custom`（`display_text` 由后端组装）。

#### Scenario: 单选含自定义

- **WHEN** pending 问题 `kind=single` 且 `allow_custom=true`
- **THEN** 用户可选择预设选项或填写自定义文本后提交

#### Scenario: 创作简报确认

- **WHEN** pending 问题 `kind=confirm_brief`
- **THEN** 用户可确认（`selected=["confirm"]`）或提交修改意见（`custom`），随后 loop 恢复

#### Scenario: multi 校验

- **WHEN** `kind=multi` 且用户选择数不满足 `min_selections`
- **THEN** 提交按钮禁用或提交时给出前端校验提示，不发起 IPC

#### Scenario: 刷新后恢复活跃卡片

- **WHEN** 用户刷新应用且 bundle 中存在 `status=awaiting_user` 的 clarify_ask 记录
- **THEN** 活跃卡片按 `args_json` 还原渲染，可正常提交

---

### Requirement: 已答澄清卡片展示

已答澄清题 SHALL 以只读卡片形式嵌入消息流（数据源：`list_messages` bundle 中 `status=done` 的 clarify_ask `ToolCallRecord`，`args_json`=问题、`result_json`=答案），用户可回看自己的选择。系统 MUST NOT 为澄清答案生成 user 消息气泡。

#### Scenario: 提交后卡片转为已答态

- **WHEN** 用户成功提交 clarify 答案
- **THEN** 活跃卡片立即转为只读已答态（显示所选项/自定义内容），`busy` 转 true 等待后续流式输出

#### Scenario: 历史会话回看澄清记录

- **WHEN** 用户重新打开包含已完成澄清的会话
- **THEN** 消息流中按位置展示各已答澄清卡片，内容与当时提交一致

---

### Requirement: 澄清进行中输入约束

当 session 存在 clarify pending 时，系统 SHALL suppress 推荐问（`SuggestionCards`），前端 MUST 阻断普通消息发送并提示先完成上方澄清（后端同步强制校验，见 agent-loop spec）。输入框 placeholder MAY 提示「请先回答上方澄清问题」。

#### Scenario: pending 时不可直接发送

- **WHEN** 存在 active clarify pending 且用户尝试发送普通消息
- **THEN** 发送被阻断并展示一次性提示，输入内容保留

#### Scenario: 澄清期间不展示 followup 推荐

- **WHEN** 收到 `turn_awaiting_user`
- **THEN** 不展示 followup/starter 推荐问胶囊

---

### Requirement: clarify 事件与类型契约

前端 `AgentEvent` 与 Rust 序列化 MUST 对齐，新增：

- `clarify_question`：`session_id`、`turn_id`、`tool_call_id`、`question`（ClarifyQuestion）
- `turn_awaiting_user`：`session_id`、`turn_id`

`ToolCall` 事件 status 取值扩展 `awaiting_user`，工具链面板 MUST 为该状态展示「等待回答」而非持续转圈。`submit_clarify_answer` 请求类型（`session_id`、`question_id`、`selected`、`custom`）MUST 在 `types.ts` 定义并与 IPC 一致。

#### Scenario: 事件驱动展示卡片

- **WHEN** 收到 `clarify_question`
- **THEN** 会话区展示对应 ClarifyQuestionCard，且收到随后的 `turn_awaiting_user` 后 `busy` 为 false

#### Scenario: 工具链面板等待态

- **WHEN** clarify_ask 进入 `awaiting_user`
- **THEN** 右侧工具链卡片显示等待回答状态；用户提交后随 `ToolResult` 转为完成

### Requirement: 更新下载进度遮罩

系统 SHALL 在用户确认安装更新后，于 App 根级展示全局更新进度遮罩，覆盖启动静默检查与设置抽屉手动更新等所有调用 `checkForAppUpdates` 的路径。遮罩 MUST 阻止用户与主界面交互，直至更新失败关闭遮罩或应用 `relaunch`。遮罩 MUST 包含圆环式进度指示器与状态文案。遮罩卡片 MUST 具备足够最小宽度，文案 MUST 居中展示；状态文案 MUST NOT 使用省略号（`…` / `...`）作为进度占位，以免用户误判为文字截断。

下载阶段（`downloading`）：

- 若 updater 提供总大小（`contentLength`），MUST 展示圆环进度与百分比；版本号与百分比 SHOULD 分行或分区展示（如主行「正在下载」、副行「v{version}」与「{n}%」）
- 若无总大小，MUST 展示旋转圆环与「正在下载」主文案及版本副文案（若有）

安装阶段（`installing`）：

- MUST 展示「正在安装」主文案与「即将重启」副文案（或等效分行语义）
- MUST NOT 展示安装百分比

#### Scenario: 启动更新确认后展示遮罩

- **WHEN** 启动静默检查发现新版本且用户在 dialog 中确认更新
- **THEN** 主界面 MUST 展示全局更新进度遮罩
- **AND** 遮罩 MUST 在下载完成前保持可见

#### Scenario: 设置抽屉触发更新展示遮罩

- **WHEN** 用户在设置抽屉点击「更新」并确认安装
- **THEN** 全局更新进度遮罩 MUST 可见
- **AND** 设置抽屉「更新」按钮 MUST 处于禁用或「更新中…」状态

#### Scenario: 有总大小时展示百分比

- **WHEN** 下载开始且 `DownloadEvent Started` 含 `contentLength`
- **THEN** 遮罩 MUST 展示圆环进度与 0–100% 数值
- **AND** 文案 MUST NOT 含省略号字符

#### Scenario: 无总大小时旋转指示

- **WHEN** 下载开始但无 `contentLength`
- **THEN** 遮罩 MUST 展示旋转圆环与「正在下载」类主文案
- **AND** MUST NOT 展示虚假百分比

#### Scenario: 安装阶段文案

- **WHEN** 下载事件 `Finished` 且安装尚未完成
- **THEN** 遮罩 MUST 展示安装阶段分行文案（含即将重启语义，无省略号）

#### Scenario: 失败关闭遮罩

- **WHEN** 更新下载或安装失败
- **THEN** 遮罩 MUST 关闭
- **AND** 用户 MUST 可继续操作主界面

### Requirement: 设置抽屉检查更新入口

系统 SHALL 在顶栏提供设置入口，点击后从右侧滑出设置抽屉；抽屉内 MUST 以简洁文案展示「当前版本」与「最新版本」两行信息，且 MUST 仅在用户打开抽屉时请求 `latest.json` 获取最新版本号。当最新版本高于当前版本时，抽屉内 SHALL 提供「更新」入口触发安装流程。

#### Scenario: 设置抽屉展示版本信息

- **WHEN** 用户打开设置抽屉
- **THEN** 抽屉内可见当前版本与最新版本两行信息

#### Scenario: 打开抽屉时查询最新版本

- **WHEN** 用户打开设置抽屉
- **THEN** 系统通过 Tauri 后端请求 updater manifest 获取最新版本号
- **AND** 在用户未打开抽屉前 MUST NOT 为展示版本信息而发起该请求

#### Scenario: 更新进行中反馈

- **WHEN** 用户触发更新且下载或安装尚未完成
- **THEN** 更新入口展示进行中状态（禁用或「更新中…」），防止重复触发
- **AND** 全局更新进度遮罩 MUST 同步展示（见「更新下载进度遮罩」）

### Requirement: 侧栏会话列表拖动排序

系统 SHALL 允许用户在当前项目的侧栏会话列表中通过拖动调整会话展示顺序。拖动 MUST 使用独立 drag handle，不得与点击选中、删除按钮冲突。排序范围 MUST 限定于当前 activeProject 下的会话。

#### Scenario: 拖动手柄重排

- **WHEN** 用户按住某会话项左侧 drag handle 并拖动到新位置后释放
- **THEN** 会话列表立即按新顺序展示

#### Scenario: 拖动不影响选中

- **WHEN** 用户点击会话标题区域（非 drag handle）
- **THEN** 选中该会话并加载消息，不触发拖动

#### Scenario: 删除按钮仍可用

- **WHEN** 用户 hover 会话项并点击删除
- **THEN** 删除该会话，不触发拖动

### Requirement: 会话列表顺序懒激活与前端持久化

系统 SHALL 按项目隔离持久化会话展示顺序于前端 `localStorage`。某项目**从未**被用户拖动排序时，列表 MUST 按后端 `updated_at` 降序展示（与改动前一致）。用户在某项目下**首次**完成拖动排序后，该项目 MUST 进入手动序模式：顺序写入 `localStorage` 并在应用重启后恢复。手动序模式下，后端刷新会话元数据（如 `turn_complete` 后 `list_sessions`）MUST NOT 改变用户设定的展示顺序。

#### Scenario: 未拖动时保持自动序

- **WHEN** 用户在某项目下从未拖动排序，且某会话因新消息导致 `updated_at` 更新
- **THEN** 该会话在列表中按 `updated_at` 规则上移（与改动前一致）

#### Scenario: 首次拖动激活手动序

- **WHEN** 用户在某项目下首次完成拖动排序
- **THEN** 顺序写入 `localStorage` 并立即生效；此后该项目不再因 `updated_at` 变化自动重排

#### Scenario: 重启后恢复手动序

- **WHEN** 用户曾拖动排序并重启应用
- **THEN** 打开同一项目时会话列表按上次保存的顺序展示

#### Scenario: turn_complete 不改变手动序

- **WHEN** 项目处于手动序模式且某会话回合结束触发 `list_sessions` 刷新
- **THEN** 列表顺序保持不变，仅会话标题等元数据可更新

#### Scenario: 手动序下新建仍置顶

- **WHEN** 项目处于手动序模式且用户点击「新建」
- **THEN** 新会话出现在列表顶部，并写入持久化顺序的首位

#### Scenario: 删除会话同步顺序

- **WHEN** 项目处于手动序模式且用户删除某会话
- **THEN** 该会话从列表与持久化顺序中移除，其余顺序不变

#### Scenario: 删至最后一个会话退出手动序

- **WHEN** 项目处于手动序模式且用户删除最后一个会话
- **THEN** 该项目的手动序 `localStorage` 条目被清除，下次新建会话时恢复自动序

#### Scenario: 项目隔离

- **WHEN** 用户在项目 A 拖动排序后切换到项目 B
- **THEN** 项目 B 展示其自身顺序（自动序或各自的手动序），不受项目 A 影响

### Requirement: 侧栏会话标题动态截断展示

系统 SHALL 在侧栏会话列表中展示完整持久化标题，并通过 CSS 文本溢出（`truncate` / `text-overflow: ellipsis`）按当前侧栏宽度动态截断；MUST 为标题元素提供 `title` 属性或等效 tooltip 以展示全文。展示前 MAY 调用 `plainSessionTitle` 去除存量 Markdown 标记。MUST NOT 依赖后端 18 字符固定截断作为唯一展示来源。

#### Scenario: 窄侧栏截断

- **WHEN** 用户收窄侧栏且会话标题较长
- **THEN** 标题在可视区域内 ellipsis 截断，hover 可见完整文本

#### Scenario: 宽侧栏展示更多

- **WHEN** 用户拉宽侧栏
- **THEN** 同一条标题可视字符数增加，无需重新请求后端

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

### Requirement: 自动压缩一次性提示

系统 SHALL 在收到 `context_compacted` 事件时展示一次性、非阻断的轻提示（toast 或会话区一行系统提示）。文案 MUST 根据 `trigger` 区分：

- `trigger: "auto"`：说明已**自动**压缩较早历史以节省上下文
- `trigger: "manual"`：说明已**手动**压缩对话历史

该提示 MUST NOT 阻断输入框或弹出模态，且 MUST NOT 常驻。

当 `compact_session` 返回 `compacted: false`（无可压缩段）时，前端 SHALL 展示一次性轻提示说明当前上下文较短、无需压缩；MUST NOT 展示「已压缩」类文案。

在调用摘要 LLM 之前，系统 SHALL 发出 `compaction_started` 事件；前端 MUST 展示进行中轻提示（自动：`正在压缩较早的对话历史…`；手动：`正在压缩上下文，请稍候…`），直至收到 `context_compacted`、错误或取消事件。进行中提示 MUST NOT 启动自动消失 timer。

#### Scenario: 自动压缩进行中提示

- **WHEN** 循环内自动压缩开始调用摘要 LLM 且前端收到 `compaction_started`（`trigger: auto`）
- **THEN** 展示「正在压缩较早的对话历史…」进行中轻提示
- **AND** 收到 `context_compacted` 后替换为「已自动压缩…」完成提示

#### Scenario: 自动压缩后轻提示

- **WHEN** 前端收到 `context_compacted` 且 `trigger` 为 `auto`
- **THEN** 展示「已自动压缩…」类一次性轻提示，输入框不被阻断

#### Scenario: 手动压缩后轻提示

- **WHEN** 用户通过 `/compact` 成功触发压缩且收到 `context_compacted`（`trigger: manual`）
- **THEN** 展示「已手动压缩…」类一次性轻提示，输入框不被阻断

#### Scenario: 手动压缩无需压缩时提示

- **WHEN** `compact_session` 返回 `compacted: false`
- **THEN** 展示「当前上下文较短，无需压缩」类一次性轻提示
- **AND** 不展示「已压缩」文案

### Requirement: 上下文事件类型契约

前端 `AgentEvent` 类型与 Rust 序列化 MUST 对齐，包含：

- `context_usage`：`session_id`、`used_tokens`、`max_tokens`、`ratio`
- `context_compacted`：`session_id`、`before_tokens`、`after_tokens`、`trigger`（`"auto"` | `"manual"`）
- `compaction_started`：`session_id`、`trigger`（`"auto"` | `"manual"`）

`AgentStreamState` MUST 维护当前会话的上下文比例（如 `contextRatio`），由 `context_usage` 更新；会话切换时 MUST 通过 `get_session_context_usage` 独立拉取，不依赖 stream state reset 后为空。

#### Scenario: context_compacted 含 trigger

- **WHEN** 前端解析 `context_compacted` 事件 payload
- **THEN** `trigger` 字段为 `auto` 或 `manual`

#### Scenario: 事件驱动更新比例状态

- **WHEN** 收到归属当前 activeSession 的 `context_usage`
- **THEN** `AgentStreamState.contextRatio` 更新为事件 `ratio`，指示器随之刷新

#### Scenario: 非活跃会话事件被忽略

- **WHEN** 收到 `session_id` 非当前 activeSession 的 `context_usage`
- **THEN** 当前展示的比例不受影响

### Requirement: Compact slash command UI blocking

The workspace chat UI SHALL block `/compact` execution under the same conditions as `/init` for clarify pending, and SHALL additionally block when the active session turn is `running` or `stopping`.

#### Scenario: Compact blocked while clarify pending

- **WHEN** the active session has a pending clarify question
- **THEN** picking `compact` from the slash menu SHALL show the same user-visible error as `/init`

#### Scenario: Compact blocked while turn running

- **WHEN** the session turn is running or stopping
- **THEN** submitting `/compact` SHALL be prevented with user-visible error

### Requirement: 设置抽屉账户余额展示

系统 SHALL 在设置抽屉内、版本信息区块下方展示账户余额。仅当用户已配置 DeepSeek 和/或 Kimi API Key 时，MUST 展示对应 provider 一行；两者均未配置时 MUST NOT 展示「账户余额」区块。

每行 MUST 以 provider 名称（DeepSeek / Kimi）与右对齐的总余额展示字符串组成。余额 MUST 仅展示人民币总可用余额；加载中 MUST 显示 `…`；查询失败 MUST 显示 `—`。

余额查询 MUST 仅在用户打开设置抽屉时发起；在用户未打开设置抽屉前 MUST NOT 为展示余额而调用 `fetch_provider_balances`。

#### Scenario: 打开抽屉时查询余额

- **WHEN** 用户打开设置抽屉且已配置至少一个 DeepSeek 或 Kimi API Key
- **THEN** 系统调用 `fetch_provider_balances` 获取余额
- **AND** 在用户未打开设置抽屉前 MUST NOT 为展示余额发起该请求

#### Scenario: 均未配置时不查询余额

- **WHEN** 用户打开设置抽屉且未配置 DeepSeek 与 Kimi API Key
- **THEN** MUST NOT 调用 `fetch_provider_balances`

#### Scenario: 已配置 DeepSeek 展示一行

- **WHEN** 用户已配置 DeepSeek API Key 且余额查询成功
- **THEN** 设置抽屉「账户余额」区块可见 DeepSeek 一行及格式化后的 ¥ 金额

#### Scenario: 未配置 Key 不展示行

- **WHEN** 用户未配置 Kimi API Key
- **THEN** 设置抽屉 MUST NOT 展示 Kimi 余额行

#### Scenario: 均未配置隐藏区块

- **WHEN** 用户未配置 DeepSeek 与 Kimi API Key
- **THEN** 设置抽屉 MUST NOT 展示「账户余额」区块

#### Scenario: 查询失败显示占位符

- **WHEN** 用户已配置 Key 但余额查询失败
- **THEN** 对应 provider 行显示 `—`

#### Scenario: 加载中状态

- **WHEN** 用户打开设置抽屉且余额请求尚未完成
- **THEN** 已配置 Key 的 provider 行显示 `…` 直至请求结束

### Requirement: 多工具 streaming 占位平滑过渡

当同轮多个工具处于参数流式生成（`tool_call_stream`）阶段时，收到 `ToolCall { status: running }` MUST 仅升级对应 `index` 的 streaming 占位卡片为 running，MUST NOT 删除其他 index 的 streaming 占位。同批全部工具开始执行后，工具链卡片数量 MUST NOT 少于 streaming 阶段的占位数量（除非某工具已被同一 index 的 running 事件替换）。

#### Scenario: 三个 pdf_read 不整栏清空

- **WHEN** 右侧工具链显示三个 `pdf_read` 的「生成参数中」卡片，且本轮三个工具即将执行
- **THEN** 过渡后仍显示三张卡片且均为「执行中」或已完成，MUST NOT 出现整栏空白占位文案

#### Scenario: 按 index 就地升级

- **WHEN** 收到 `ToolCall { index: 1, status: running }` 且存在 `streaming-1` 占位
- **THEN** 该占位卡片升级为 running 并展示参数，其他 streaming 占位保持不变

### Requirement: 工作区分栏布局持久化
系统 SHALL 将 **主三栏水平比例** 持久化于前端 `localStorage`，应用重启后恢复用户上次选择。首次访问或无有效缓存时 MUST 使用默认水平比例。**Inspector 当前 Tab MUST NOT 持久化**；每次应用启动 MUST 默认选中「项目文件」。布局持久化 MUST NOT 与 `doc-agent-last-session-config` 混用同一存储键。系统 MUST NOT 再持久化右侧 **上下垂直** 分栏比例。

#### Scenario: Inspector Tab 启动默认
- **WHEN** 用户重启应用
- **THEN** 右侧 Inspector 默认展示「项目文件」，MUST NOT 读取上次退出时的 Tab 偏好

### Requirement: Chat 输入工具栏

Chat 输入区 SHALL 在 textarea 下方（或与发送按钮同一 composite 输入框底栏）展示三个工具按钮：**+**（上传文件到项目根）、**图片**（选择图片作为消息附件）、**/**（打开斜杠命令图形菜单）。各按钮 MUST 具备 tooltip 与无障碍 `aria-label`。澄清进行中（`activeClarify`）、busy、initializing 时，三按钮与 textarea MUST 一并 disabled。

#### Scenario: 工具栏可见

- **WHEN** 用户已选项目且输入区未 disabled
- **THEN** 输入框底栏展示 +、图片、/ 三个按钮

#### Scenario: 澄清期间禁用

- **WHEN** session 存在 pending clarify
- **THEN** 三按钮 disabled，与 textarea 一致

### Requirement: 斜杠命令图形菜单（二级分类）

除键盘 `/` 触发的 fuzzy 弹层外，系统 SHALL 提供 **/** 按钮打开的**二级分类**菜单：第一级为分类（通用、Word、PPT、Excel、PDF、Web，顺序与 `CATEGORY_ORDER` 一致），第二级为该分类下全部斜杠命令（展示 `label` 与一行 `description`）。选中命令后 MUST 调用与键盘斜杠相同的 prompt 插入逻辑（`insertSlashPrompt`）：填入 registry `prompt`、选中首个 `{{占位符}}`、**MUST NOT** 自动发送。

菜单正文字号 MUST 与斜杠 fuzzy 弹层、`@` 弹层一致（`text-xs` 基线）。

#### Scenario: 打开二级菜单

- **WHEN** 用户点击输入区 `/` 按钮
- **THEN** 展示分类 tab 与当前分类命令列表

#### Scenario: 选中命令插入 prompt

- **WHEN** 用户在图形菜单中选择某命令
- **THEN** 输入框插入对应 prompt 并聚焦，消息未发送

#### Scenario: 与 @ 弹层互斥

- **WHEN** `@` 文件弹层正在展示
- **THEN** 不展示斜杠图形菜单（若已打开则关闭）

#### Scenario: Esc 或外部点击关闭

- **WHEN** 斜杠图形菜单打开且用户按 Esc 或点击外部
- **THEN** 菜单关闭

### Requirement: 输入区 placeholder 补充上传提示

非 disabled 状态下，textarea placeholder SHOULD 在现有 `@`、`/`、粘贴图片提示基础上，补充 **+** 上传文件至项目根的简短说明（与 clarify/busy/initializing 专用 placeholder 互斥）。

#### Scenario: 常规定位 placeholder

- **WHEN** 用户已选项目、输入区可用
- **THEN** placeholder 提及 `@` 引用、`/` 或图形菜单任务模板、粘贴或按钮添加图片、**+** 上传文件

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

Chat 输入区 SHALL 在当前 active session 为 `running` 时展示 **停止** 按钮（与发送互斥：running 时禁用发送）。点击 MUST 调用 `cancel_turn` 并将该 session 置 `stopping`。`stopping` 时停止按钮 disabled，placeholder 或 activity 文案 MUST 说明可能等待当前工具结束（最长约 35 秒）。`turn_awaiting_user`（clarify）时 MUST NOT 展示 Stop（澄清流程使用 clarify 卡片）。

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

### Requirement: 全局并行上限提示

前端 SHALL 基于 per-session running map 派生当前 running/stopping 数量。当本地已知数量达到 3 时，输入区 MUST 阻止新发送并提示「当前已有 3 个任务正在执行，请稍后重试」。后端仍 MUST 作为权威校验；若后端返回全局满额错误，前端 MUST 保留用户输入，不得清空草稿。

#### Scenario: 本地满额禁用发送

- **WHEN** 前端已知 3 个 session 处于 running 或 stopping
- **THEN** 当前输入区发送按钮 disabled 或点击后展示满额提示

#### Scenario: 后端满额错误保留输入

- **WHEN** 用户发送时后端返回全局满额错误
- **THEN** 输入框内容仍保留，用户可稍后重试

### Requirement: 文件占用错误展示

前端 SHALL 能展示后端文件锁冲突错误。错误文案 MUST 包含被占用路径；当后端提供 blocking session 标题或 id 时，前端 SHOULD 展示「当前 xxx 已被会话 yyy 占用，请稍后重试」。工具链卡片在 `tool_result.ok=false` 时 MUST 解析 `summary` 中的 `file_busy` JSON 并以醒目样式展示 `message`（或等价格式化文案）。

#### Scenario: 工具结果 file_busy

- **WHEN** tool_result 内容表示 `file_busy`
- **THEN** UI 在工具链卡片或 toast 中展示占用路径与重试建议

### Requirement: 后台 session terminal 同步

前端 SHALL 对非 active session 的 `turn_complete`、`turn_cancelled`、`turn_awaiting_user` 事件更新对应 session running 状态。若事件所属 project 是当前 active project，前端 SHOULD 刷新 session list 与项目文件浏览状态；但 MUST NOT 用后台 session 的 messages 覆盖当前 active session 的消息列表。

#### Scenario: 后台完成不覆盖当前消息

- **WHEN** active session 为 B
- **AND** 后台 session A 收到 `turn_complete`
- **THEN** A 的侧栏 running 指示消失
- **AND** 当前中间区仍显示 B 的消息

#### Scenario: 后台文件变更刷新项目文件区

- **WHEN** 后台 session A 的 tool_result 含 `changed_paths`
- **AND** A 属于当前 active project
- **THEN** 项目文件浏览区按现有规则刷新当前目录或文件索引

### Requirement: Init command blocked during pending clarify

The workspace UI SHALL prevent submitting `/init` while a clarify question is pending for the active session.

#### Scenario: Slash init disabled with pending card

- **WHEN** `clarify_pending` is set for the active session
- **AND** the user attempts to run the `init` slash command
- **THEN** the UI SHALL show an error or disabled state explaining that clarification must be completed first
- **AND** SHALL NOT call `send_message`

#### Scenario: Composer init prefix guarded

- **WHEN** the user manually types a message starting with `/init` while clarify is pending
- **THEN** send SHALL be blocked client-side with the same message
- **AND** if bypassed, the backend error from `send_message` SHALL be surfaced

### Requirement: Confirm agents markdown clarify UI

The clarify card SHALL render `confirm_agents_md` questions with a scrollable full-text Markdown preview of `preview_markdown`.

#### Scenario: Preview displays full proposed body

- **WHEN** a pending clarify question has `kind` `confirm_agents_md`
- **THEN** the card SHALL render `preview_markdown` as Markdown inside a scrollable region
- **AND** SHALL provide confirm and reject actions consistent with other confirm-style clarify kinds

#### Scenario: Optional changelog hint

- **WHEN** `changelog_summary` is present on a `confirm_agents_md` question
- **THEN** the card SHALL display it as supplementary text above or below the preview

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

### Requirement: AGENTS.md status indicator

The workspace UI SHALL indicate whether the active project has a non-empty `AGENTS.md` at project root. The indicator SHALL appear in the **composer context bar** (in addition to or instead of the chat panel title row).

#### Scenario: Indicator reflects file presence

- **WHEN** the user selects a project with `AGENTS.md` present
- **THEN** the composer context bar shows a visible indicator

#### Scenario: Indicator hidden or muted when missing

- **WHEN** the active project has no `AGENTS.md` or an empty file
- **THEN** the indicator reflects missing/empty state without blocking chat

### Requirement: Chat 输入区焦点策略

系统 SHALL 使 Chat composer（textarea）在回合结束、切换会话后保持聚焦，无需用户手动点击。当 Settings/Credentials Drawer、图片预览、斜杠或 @ 弹层、Model Flyout、应用更新遮罩打开，或 composer 不可编辑、未选项目时，系统 MUST NOT 自动 refocus。

#### Scenario: 回合结束后自动聚焦

- **WHEN** 用户提交消息且 Agent 回合结束，composer 从 disabled 恢复为可编辑，且无 Overlay 抑制条件
- **THEN** textarea 获得焦点，用户可直接键入下一条消息

#### Scenario: 切换会话后自动聚焦

- **WHEN** 用户在侧栏选择或切换到另一会话，且 composer 可编辑、无 Overlay 抑制
- **THEN** textarea 获得焦点，用户可直接在该会话输入

#### Scenario: 文件导入完成后不强制聚焦

- **WHEN** 用户通过 composer 导入文件，`importing` 从 true 变为 false，且导入流程已通过 `onFocusInput` 设置光标位置
- **THEN** 系统 MUST NOT 因 `composerDisabled` 恢复而额外 refocus 至 `(0, 0)` 重置光标

#### Scenario: Overlay 打开时不聚焦

- **WHEN** Settings 或 Credentials Drawer 打开，或图片预览、Model Flyout、更新遮罩展示中
- **THEN** 系统 MUST NOT 因回合结束或切换会话而 refocus textarea

#### Scenario: 澄清进行中不聚焦

- **WHEN** session 存在 pending clarify 且 composer 为 disabled
- **THEN** 系统 MUST NOT refocus textarea

#### Scenario: IME 组合中 Enter 确认候选词不误发消息

- **WHEN** 用户在 composer 内使用输入法组合输入，按下 Enter 确认候选词（`isComposing` 为 true 或 `keyCode === 229`）
- **THEN** 系统 MUST NOT 触发发送，MUST NOT 执行 mention/斜杠弹层选择或删除占位符；该按键 MUST 交由浏览器/输入法原生处理

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

### Requirement: 设置抽屉界面缩放配置

系统 SHALL 在设置抽屉内、版本信息区块与工作区布局区块之间（或紧邻工作区布局之上）提供 **界面缩放** 配置区块（`config-surface` 样式）。该区块 MUST 包含标题「界面缩放」与当前缩放百分比的可见标签（如 `140%`），以及范围 **100%–200%**、步进 **20%** 的 **滑块**（或等效控件），变更时立即生效。系统 MUST NOT 在该区块展示冗长说明、快捷键提示或窗口尺寸建议；缩放语义由标题与当前值自明。滑块与快捷键 MUST 驱动同一缩放状态。

#### Scenario: 设置抽屉展示缩放区块

- **WHEN** 用户打开设置抽屉
- **THEN** 可见「界面缩放」标题、当前百分比标签与滑块

#### Scenario: 拖动滑块立即生效

- **WHEN** 用户将滑块从 100% 拖至 160%
- **THEN** 主界面立即以 160% 渲染，且偏好持久化

#### Scenario: 滑块可访问性

- **WHEN** 辅助技术聚焦界面缩放滑块
- **THEN** 控件具备描述性 accessible 名称，且 `aria-valuetext`（或等效）反映当前百分比

### Requirement: 恢复默认工作区设置

系统 SHALL 在设置抽屉「工作区布局」区块提供「恢复默认布局」入口；点击后 MUST 清除主三栏布局持久化并恢复默认比例，且 MUST 将界面缩放重置为 100%。

#### Scenario: 恢复默认同时重置缩放

- **WHEN** 用户已将界面缩放设为 160% 并点击「恢复默认布局」
- **THEN** 三栏恢复默认比例且界面缩放变为 100%

