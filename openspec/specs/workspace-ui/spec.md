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
输入框 SHALL 支持 `@` 触发的文件引用：检测到光标前的 `@` 及其后查询串时，弹出项目内文件/目录候选列表，支持 fzf 式模糊匹配（子序列匹配 + 评分排序 + 命中高亮）、键盘上下选择与确认；确认后在输入框插入 `@相对路径`。文件清单 MUST 限制遍历深度与数量并忽略隐藏目录/依赖目录/Office 临时文件。

#### Scenario: 模糊匹配选择文件
- **WHEN** 用户在输入框键入 `@课程` 且项目内存在「课程体系.xlsx」
- **THEN** 弹层展示按匹配度排序的候选（含该文件），用户按 Enter 后输入框中 `@课程` 被替换为 `@课程体系.xlsx `

#### Scenario: Esc 关闭弹层
- **WHEN** 弹层展示中用户按 Esc 或将光标移出 `@` 区域
- **THEN** 弹层关闭，输入内容保持不变

#### Scenario: Agent 理解 @ 引用
- **WHEN** 用户发送包含 `@相对路径` 的消息
- **THEN** system prompt 已声明该语义，Agent 可直接以该路径调用文件/文档工具读取

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

