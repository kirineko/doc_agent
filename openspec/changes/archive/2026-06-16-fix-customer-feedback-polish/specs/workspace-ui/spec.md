## MODIFIED Requirements

### Requirement: 模型 Flyout 锚定侧栏

系统 SHALL 在用户点击侧栏模型摘要 trigger 时，于 trigger **附近**（优先向上展开）显示固定定位 Flyout，而非屏幕右侧全高 Drawer。Flyout MUST 包含：当前模型摘要、Provider segmented control、可滚动模型列表、底部 sticky 思考配置。切换 Provider Tab 时 MUST 预选该 Provider 的第一个可用模型（与现有 `configForProviderFirstModel` 行为一致）。Flyout 水平宽度 MUST 与模型 trigger 按钮同宽（随侧栏拖拽 resize 实时更新）。

#### Scenario: Flyout 靠近 trigger

- **WHEN** 用户点击侧栏左下模型 trigger
- **THEN** Flyout 在侧栏内、trigger 上方或下方展开，水平对齐 trigger，不要求用户视线移至屏幕最右侧

#### Scenario: Flyout 宽度随侧栏自适应

- **WHEN** 用户拖宽左侧栏且模型 Flyout 处于打开状态
- **THEN** Flyout 宽度 MUST 与 trigger 同宽，不得固定窄于侧栏（如 320px 上限）

#### Scenario: Flyout 不含 Key 配置

- **WHEN** 用户打开模型 Flyout
- **THEN** 界面不包含任何 API Key 输入控件

#### Scenario: Provider Tab 预选首模型

- **WHEN** 用户在 Flyout 切换至 Kimi Provider Tab 且会话模型未锁定
- **THEN**  pending/空会话模型切换为该 Provider 列表中的第一个模型

#### Scenario: 侧栏摘要含 vision 标识

- **WHEN** 当前选中 Kimi K2.6
- **THEN** 侧栏模型 trigger 摘要显示模型名与视觉能力标识

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

### Requirement: 斜杠命令弹层 UI

斜杠命令弹层 SHALL 按分类展示分组标题（通用、Word、PPT、Excel、PDF、Web），每条候选 MUST 展示命令 id、`label` 与一行 `description`；匹配字符 MUST 高亮（与 `@` 弹层同类样式）。

弹层 MUST 可滚动；样式 MUST 复用或延伸现有 `mention-popup` 设计令牌，与明暗主题一致。弹层正文字号 MUST 为 `text-xs`（12px）基线，与 `@` 弹层及斜杠图形菜单一致，不得使用小于 12px 作为正文主字号。

#### Scenario: 分组展示

- **WHEN** 用户输入 `/` 且无 query 过滤
- **THEN** 弹层按 general、word、ppt、excel、pdf、web 顺序展示分组

#### Scenario: 单行展示 id / label / description

- **WHEN** 弹层展示 `word:edit`
- **THEN** 同一行可见命令 id、`label` 与 `description`（过长时 truncate + title）

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
