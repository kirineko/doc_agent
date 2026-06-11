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
系统 SHALL 在左侧栏展示项目与会话列表，并提供模型选择、思考开关与思考强度（按模型差异化）配置入口。

#### Scenario: 在侧栏切换会话与配置
- **WHEN** 用户在左侧栏选择另一个会话并切换模型 / 思考配置
- **THEN** 中间会话区切换为该会话内容，模型 / 思考配置随之更新并持久化

### Requirement: 中间区 Markdown 流式渲染
系统 SHALL 在中间区以良好的 Markdown 渲染展示会话与结果，支持流式增量更新、代码高亮与表格；思考内容与正文分区展示。

#### Scenario: 流式渲染回答
- **WHEN** 模型流式返回正文
- **THEN** 中间区随增量平滑渲染 Markdown，代码块高亮、表格正确呈现

#### Scenario: 思考内容可折叠
- **WHEN** 模型返回思考内容
- **THEN** 思考内容以可折叠的独立区域展示，不与正文混排

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
当首次会话推荐问生成进行中时，界面 SHALL 进入「会话初始化中」状态：禁用消息输入框并展示带动效的进度提示（如「正在阅读项目文档…」）；生成结束（无论成功失败）后 MUST 解锁输入框。

#### Scenario: 初始化期间输入禁用
- **WHEN** 空会话打开且推荐问生成请求进行中
- **THEN** 输入框与发送按钮禁用，会话区显示初始化进度提示

#### Scenario: 失败也解锁
- **WHEN** 推荐问生成失败或超时
- **THEN** 进度提示消失、输入框解锁，不展示错误干扰用户

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

