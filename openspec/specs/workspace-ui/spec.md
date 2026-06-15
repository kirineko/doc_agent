# workspace-ui Specification

## Purpose
TBD - created by archiving change bootstrap-doc-agent-mvp. Update Purpose after archive.
## Requirements
### Requirement: 三栏工作区布局
系统 SHALL 提供三栏布局：左侧为项目 / 会话 / 模型配置，中间为会话与结果，右侧为工具调用链与项目文件浏览（上下分栏）。

#### Scenario: 三栏同时可见
- **WHEN** 用户打开一个项目的会话
- **THEN** 界面同时呈现左侧导航与配置、中间会话区、右侧工具调用链与文件浏览两个区域

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
- **THEN** 侧栏模型与思考配置以只读形式展示，不可修改

### Requirement: 中间区 Markdown 流式渲染

系统 SHALL 在中间区以良好的 Markdown 渲染展示会话与结果，支持流式增量更新、代码高亮与表格；思考内容与正文分区展示。assistant 消息的**流式预览**与**持久化展示** MUST 使用同一消息气泡结构（思考可折叠区 + 正文 Markdown 区），仅允许样式 variant（如边框/动效）区分「生成中」与「已完成」。多轮工具调用时，每一步 LLM 的流式预览 MUST 独立呈现，不得将多步思考/正文累加在同一流式气泡中。收到 `assistant_step_done` 后，该步 assistant MUST 立即出现在消息列表中，并清空当前 streaming 缓冲；`turn_complete` 时仍可全量 `list_messages` 对齐，但 MUST NOT 导致 assistant 消息条数或内容与逐步展示结果发生可见冲突。user 消息若含图片附件，MUST 在文本旁展示缩略图。

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

#### Scenario: 用户消息展示图片附件

- **WHEN** 历史消息含 `attachments_json` 指向 `.cache/attachments/photo.png`
- **THEN** 消息气泡展示该图缩略图与文本内容

#### Scenario: 附件文件缺失时展示占位

- **WHEN** 历史消息含 `attachments_json` 但磁盘文件不存在
- **THEN** 消息气泡展示「无法加载」缩略图占位，文本内容仍可见

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

### Requirement: 右侧工具调用链可视化
系统 SHALL 在右侧栏上半区以简洁美观的方式展示工具调用链，每个调用呈现名称、参数、状态与结果（含耗时）；下半区留给项目文件浏览，二者共享右侧栏宽度且各自可纵向滚动。

#### Scenario: 展示工具调用进展
- **WHEN** Agent 发起并完成一个工具调用
- **THEN** 右侧栏上半区出现对应卡片，状态从「执行中」更新为「完成 / 失败」，并显示结果摘要与耗时

### Requirement: 项目列表展示与隐藏交互
左侧项目列表 SHALL 提供更大的可视区域（较 MVP 至少加大约一倍），并在每个项目卡片上提供移除（隐藏）交互入口；不提供已隐藏项目的管理入口。

#### Scenario: hover 显示移除按钮
- **WHEN** 用户将鼠标悬停在项目卡片上
- **THEN** 卡片显示移除按钮，点击后该项目立即从列表消失

### Requirement: @ 文件引用选择器
输入框 SHALL 支持 `@` 触发的文件引用：检测到光标前的 `@` 及其后查询串时，弹出项目内文件/目录候选列表，支持 fzf 式模糊匹配（子序列匹配 + 评分排序 + 命中高亮）、键盘上下选择与确认；确认后在输入框插入 `@相对路径`。文件清单 MUST 限制遍历深度与数量并忽略隐藏目录/依赖目录/Office 临时文件；**此外 MUST 忽略 OOXML 解压工作目录（路径段名为 `unpacked` 或以 `_unpacked` 结尾的目录）及其全部子树**。清单 MUST 在项目文件变更后更新：优先通过 `tool_result.changed_paths` 增量合并，并在每个 turn 完成时 debounce 全量刷新一次。

#### Scenario: 模糊匹配选择文件
- **WHEN** 用户在输入框键入 `@课程` 且项目内存在「课程体系.xlsx」
- **THEN** 弹层展示按匹配度排序的候选（含该文件），用户按 Enter 后输入框中 `@课程` 被替换为 `@课程体系.xlsx `

#### Scenario: Esc 关闭弹层
- **WHEN** 弹层展示中用户按 Esc 或将光标移出 `@` 区域
- **THEN** 弹层关闭，输入内容保持不变

#### Scenario: Agent 理解 @ 引用
- **WHEN** 用户发送包含 `@相对路径` 的消息
- **THEN** system prompt 已声明该语义，Agent 可直接以该路径调用文件/文档工具读取

#### Scenario: 解压目录内部不出现在 @ 候选
- **WHEN** 项目内存在 `unpacked/word/document.xml`（由 `ooxml_unpack` 产生）
- **THEN** `@` 候选列表 MUST NOT 包含该路径或 `unpacked/` 下任意子路径
- **AND** 用户仍可通过 `@` 引用同级的 `.docx` 成品文件

#### Scenario: Agent 新建文件后可 @ 引用
- **WHEN** Agent 在本会话 turn 中新建了 `summary.md`
- **THEN** turn 结束后用户在 `@` 中可匹配并选中 `summary.md`

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

#### Scenario: 点击推荐填入输入框
- **WHEN** 用户点击一条推荐问胶囊
- **THEN** 该文本出现在输入框中且输入框获得焦点，推荐区在用户发送消息前仍可展示

#### Scenario: 迟到的 followup 被丢弃
- **WHEN** followup 推荐尚未返回时用户已手动发送了新消息
- **THEN** 返回的推荐结果被丢弃，不展示

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
系统 SHALL 在侧栏提供独立于模型 API Key 区域的「Web 搜索 (Tavily)」配置入口，与会话无关；已保存 Key 时摘要显示「已启用」，未配置时显示「未启用」。交互 MUST 支持保存、更换、清空，且 MUST NOT 依赖 activeSession。

#### Scenario: 无会话时可配置 Tavily
- **WHEN** 用户已选项目但无 activeSession
- **THEN** 仍可在侧栏 Web 搜索区块配置 Tavily Key

#### Scenario: 与模型 Key 分区展示
- **WHEN** 用户打开侧栏
- **THEN** Web 搜索配置与 DeepSeek/Kimi API Key 区域分离展示，不混入模型 provider 列表

#### Scenario: 已保存 Key 低干扰展示
- **WHEN** Tavily Key 已保存
- **THEN** 区块以折叠摘要「已启用」展示，不默认展开密码输入框

### Requirement: Web 工具中文标签
系统 SHALL 为 `web_search` 与 `web_extract` 提供中文工具链标签，并在工具名注册列表测试中保持同步。

#### Scenario: 工具卡片显示中文名
- **WHEN** Agent 调用 `web_search` 或 `web_extract`
- **THEN** 右侧工具链卡片显示对应中文标签（非原始英文名）

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

系统 SHALL 在用户确认安装更新后，于 App 根级展示全局更新进度遮罩，覆盖启动静默检查与设置抽屉手动更新等所有调用 `checkForAppUpdates` 的路径。遮罩 MUST 阻止用户与主界面交互，直至更新失败关闭遮罩或应用 `relaunch`。遮罩 MUST 包含圆环式进度指示器与状态文案。

下载阶段（`downloading`）：

- 若 updater 提供总大小（`contentLength`），MUST 展示圆环进度与百分比，文案含目标版本号（如「正在下载 v{version}… {n}%」）
- 若无总大小，MUST 展示旋转圆环与「正在下载更新…」或等效文案

安装阶段（`installing`）：

- MUST 展示「正在安装，即将重启…」或等效文案
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

#### Scenario: 无总大小时旋转指示

- **WHEN** 下载开始但无 `contentLength`
- **THEN** 遮罩 MUST 展示旋转圆环与「正在下载…」文案
- **AND** MUST NOT 展示虚假百分比

#### Scenario: 安装阶段文案

- **WHEN** 下载事件 `Finished` 且安装尚未完成
- **THEN** 遮罩文案 MUST 切换为安装阶段（含即将重启语义）

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

### Requirement: 上下文占用比例展示

系统 SHALL 在会话区标题栏（中间区「会话」标题行右侧）以**最小化**形式展示当前上下文占用比例：仅图标 + 比例百分比值（如 `42%`），MUST NOT 展示 token 绝对值等冗余信息。比例数据来源为 `context_usage` 事件的 `ratio` 以及切换会话时 IPC `get_session_context_usage` 的初始值；无 LLM 调用历史时 MUST 显示 `0%`（仅无项目时隐藏）。指示器颜色 MAY 随接近上限而变化（如转橙/红）。

#### Scenario: 展示当前占用比例

- **WHEN** 当前会话已发生至少一次 LLM 响应且收到 `context_usage`
- **THEN** 会话区标题栏右侧显示图标 + 百分比（如 `42%`），不显示绝对 token 数

#### Scenario: 切换会话重置比例

- **WHEN** 用户切换到另一个会话
- **THEN** 系统通过 `get_session_context_usage` 拉取该会话比例并展示（无历史时为 `0%`）

#### Scenario: 空会话展示零比例

- **WHEN** 用户新建或切换到尚无 LLM 调用的空会话
- **THEN** 上下文占用指示器显示 `0%`，而非隐藏

#### Scenario: 接近上限的视觉提示

- **WHEN** 上下文占用比例接近模型上限（高 ratio）
- **THEN** 指示器以更醒目的颜色提示（如橙/红），帮助用户感知即将压缩

### Requirement: 自动压缩一次性提示

系统 SHALL 在收到 `context_compacted` 事件时展示一次性、非阻断的轻提示（toast 或会话区一行系统提示），文案说明已自动压缩较早历史以节省上下文。该提示 MUST NOT 阻断输入框或弹出模态，且 MUST NOT 常驻。

#### Scenario: 压缩后给出轻提示

- **WHEN** 前端收到 `context_compacted` 事件
- **THEN** 展示一次性轻提示（如「已自动压缩较早的对话历史」），输入框不被阻断

### Requirement: 上下文事件类型契约

前端 `AgentEvent` 类型与 Rust 序列化 MUST 对齐，新增：

- `context_usage`：`session_id`、`used_tokens`、`max_tokens`、`ratio`
- `context_compacted`：`session_id`、`before_tokens`、`after_tokens`

`AgentStreamState` MUST 维护当前会话的上下文比例（如 `contextRatio`），由 `context_usage` 更新；会话切换时 MUST 通过 `get_session_context_usage` 独立拉取，不依赖 stream state reset 后为空。

#### Scenario: 事件驱动更新比例状态

- **WHEN** 收到归属当前 activeSession 的 `context_usage`
- **THEN** `AgentStreamState.contextRatio` 更新为事件 `ratio`，指示器随之刷新

#### Scenario: 非活跃会话事件被忽略

- **WHEN** 收到 `session_id` 非当前 activeSession 的 `context_usage`
- **THEN** 当前展示的比例不受影响

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

