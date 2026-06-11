# project-file-browser Specification

## Purpose
TBD - created by archiving change workspace-product-polish. Update Purpose after archive.
## Requirements
### Requirement: 项目目录单层浏览
系统 SHALL 在右侧栏下半区提供项目文件浏览，仅展示当前项目根下的**单层**目录项；用户进入子目录后只显示该目录直接子项，不提供递归展开整棵树。非根目录时系统 MUST 提供 Finder 风格的列表首行「返回上级」入口，并在路径行展示可点击面包屑；项目根在面包屑中以 `⌂` 符号表示（须配备「项目根目录」无障碍标签），不再依赖标题栏角落的 `..` 作为唯一返回入口。**Agent 写文件或用户手动刷新后，当前目录列表 MUST 与磁盘一致。**

#### Scenario: 列出项目根目录
- **WHEN** 用户选中某项目且浏览路径为 `.`
- **THEN** 列表展示该项目根目录下的文件与子目录（一层），忽略隐藏目录、`node_modules`、`target` 及 `~$` 临时 Office 文件（与 `list_project_files` 规则一致）；路径行展示 `⌂` 为当前位置，不显示「返回上级」列表项

#### Scenario: 进入子目录
- **WHEN** 用户点击某子目录 `docs/`
- **THEN** 列表更新为 `docs/` 下的直接子项；列表首项为「返回上级」；路径行展示可点击面包屑 `⌂ / docs`，其中 `docs` 为当前段

#### Scenario: 列表返回上一级
- **WHEN** 用户在 `docs/reports/` 下点击列表首项「返回上级」
- **THEN** 列表更新为 `docs/` 下的直接子项

#### Scenario: 面包屑返回项目根
- **WHEN** 用户在 `docs/reports/` 下点击面包屑中的 `⌂`
- **THEN** 列表回到项目根目录内容，路径行仅显示 `⌂`

#### Scenario: 面包屑跳转中间层级
- **WHEN** 用户在 `docs/reports/` 下点击面包屑中的 `docs`
- **THEN** 列表更新为 `docs/` 下的直接子项，路径行面包屑为 `⌂ / docs`

#### Scenario: 无项目时不展示
- **WHEN** 用户尚未选择项目
- **THEN** 文件浏览区显示占位提示，不发起目录列表请求

#### Scenario: 解压工作目录仍可在浏览区看到文件夹
- **WHEN** Agent 执行 `ooxml_unpack` 输出到 `unpacked/`
- **THEN** 项目根列表展示 `unpacked/` 目录项；用户点击进入后可浏览其内部 XML（与 `@` 索引忽略规则独立）

### Requirement: 用系统默认应用打开文件
系统 SHALL 允许用户从文件浏览区打开项目内文件，调用操作系统默认关联应用。

#### Scenario: 双击打开文件
- **WHEN** 用户双击浏览列表中的 `报告.docx`
- **THEN** 系统用默认应用打开该项目目录下对应文件

#### Scenario: 目录不可打开
- **WHEN** 用户双击某目录项
- **THEN** 系统进入该目录（与单击行为一致），不调用外部打开

#### Scenario: 打开越界路径被拒绝
- **WHEN** 前端传入的相对路径经 sandbox 解析后越界
- **THEN** IPC 返回错误，不调用系统打开

### Requirement: 扁平文件清单忽略 OOXML 解压目录
`list_project_files`（供 `@` 引用与全量索引）SHALL 在现有忽略规则基础上，**跳过路径段名为 `unpacked`（大小写不敏感）或以 `_unpacked` 结尾的目录及其全部 descendant**。`list_project_dir` 单层列举不受此规则影响。

#### Scenario: 解压目录内部不计入 flat 清单
- **WHEN** 项目存在 `contract_unpacked/word/document.xml`
- **THEN** `list_project_files` 返回的 entries MUST NOT 含该路径

#### Scenario: 解压目录同级文件仍可见
- **WHEN** 项目根同时存在 `contract.docx` 与 `contract_unpacked/`
- **THEN** `list_project_files` entries 包含 `contract.docx`，不包含 `contract_unpacked/` 下任意路径

### Requirement: 文件浏览区变更同步
系统 SHALL 在 Agent 成功变更项目文件后，自动刷新资源管理器**当前浏览目录**的列表；刷新 MUST 使用 `list_project_dir_cmd`（单层），MUST NOT 在每次变更时递归 walk 全项目。

#### Scenario: 当前目录出现新文件
- **WHEN** 用户浏览项目根目录 `.` 且 Agent 在根目录创建了 `output.docx`
- **THEN** 无需手动切换目录，列表中出现 `output.docx`

#### Scenario: 子目录内变更刷新子目录
- **WHEN** 用户正在浏览 `docs/` 且 Agent 在 `docs/` 下创建 `draft.md`
- **THEN** `docs/` 列表刷新并显示 `draft.md`

#### Scenario: 变更不在当前目录时不误跳路径
- **WHEN** 用户正在浏览 `docs/` 但 Agent 在项目根创建了 `new.docx`
- **THEN** 列表仍停留在 `docs/` 内容，不自动跳转到根目录

### Requirement: 手动刷新当前目录
资源管理器 SHALL 提供手动刷新入口，重新加载当前路径的单层目录列表。

#### Scenario: 点击刷新按钮
- **WHEN** 用户点击文件浏览区的刷新控制且当前路径为 `reports/`
- **THEN** 系统调用 `list_project_dir_cmd(project_id, "reports/")` 并更新列表

#### Scenario: 外部新建文件通过手动刷新可见
- **WHEN** 用户在系统文件管理器中向项目目录添加了文件，并在应用内点击刷新
- **THEN** 当前浏览目录列表包含该新文件（若位于当前路径下）

