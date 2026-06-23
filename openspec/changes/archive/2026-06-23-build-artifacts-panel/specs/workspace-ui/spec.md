## ADDED Requirements

### Requirement: 构建产物 Tab 视图

系统 SHALL 在右侧上半区（工具调用链所在区域）提供 Tab 切换：「工具调用链」与「构建产物」两个 Tab，同一时刻仅展示其一。Tab 切换 MUST NOT 影响下半区项目文件浏览、上下分割条、高度比例与既有的折叠 / 拖拽语义。「构建产物」Tab 标题 SHALL 显示本轮产物数量徽标。

#### Scenario: 默认展示工具调用链 Tab

- **WHEN** 用户进入工作区且 Agent 未运行
- **THEN** 右侧上半区默认选中「工具调用链」Tab；「构建产物」Tab 可见但无徽标或徽标为 0

#### Scenario: Tab 切换不影响布局

- **WHEN** 用户在「工具调用链」与「构建产物」Tab 之间切换
- **THEN** 上下分割条位置、高度比例、折叠状态均保持不变；下半区项目文件浏览不受影响

#### Scenario: Tab 徽标反映本轮产物数

- **WHEN** Agent 在本轮累积产生 2 个去重后的产物路径
- **THEN** 「构建产物」Tab 标题显示徽标 `2`；用户未切到该 Tab 也能从徽标感知本轮有产出

#### Scenario: 工具链折叠时 Tab 栏随之收起

- **WHEN** 用户折叠工具调用链分区
- **THEN** Tab 栏与内容区一并收起，仅保留标题行；展开后 Tab 栏恢复

### Requirement: 本轮构建产物列表

系统 SHALL 在「构建产物」Tab 内展示「本轮」Agent 产生或修改的项目相对路径列表。产物列表 MUST 按当前 turn 累积：新 turn（用户发送新消息）开始时清空。产物路径 MUST 去重。每个产物项 SHALL 标注其来源工具调用（工具中文名）。无产物时 MUST 展示空态文案。产物状态 MUST 按 `session_id` 维护于前端内存（与 per-session 工具调用链 `liveTools` 一致）；切换 `activeSessionId` 时 MUST 恢复该 session 的 `turnArtifacts`，MUST NOT 写入数据库或 `localStorage`。

#### Scenario: 累积本轮产物

- **WHEN** Agent 在本轮先后通过 `skill_run` 产出 `report.docx`、`data.xlsx`，再通过 `fs_write` 修改 `notes.md`
- **THEN** 「构建产物」Tab 列出三个路径项，每项标注来源工具（skill_run / fs_write）

#### Scenario: 同路径去重

- **WHEN** 同一轮内两个工具调用都写入了 `report.docx`
- **THEN** 产物列表中 `report.docx` 仅出现一次，来源标注保留首个产生它的工具调用

#### Scenario: 新 turn 清空产物

- **WHEN** 用户在上一轮产物列表存在时发送一条新的用户消息
- **THEN** 「构建产物」Tab 清空，徽标归零，开始累积新 turn 的产物

#### Scenario: 无产物空态

- **WHEN** 本轮 Agent 未产生或修改任何文件（如纯对话或仅只读工具）
- **THEN** 「构建产物」Tab 展示空态文案（如「本轮没有产生或修改文件」），徽标为 0

#### Scenario: 切换会话保留该 session 的产物

- **WHEN** session A 在本轮已累积产物，用户切换到 session B 后再切回 A
- **THEN** A 的「构建产物」Tab 恢复展示切换前累积的产物列表与徽标（与 per-session 工具调用链行为一致）

#### Scenario: 刷新应用后产物不持久化

- **WHEN** 用户刷新应用或重启进程
- **THEN** 各 session 的产物列表为空；MUST NOT 从磁盘恢复历史产物

#### Scenario: 不显示 .cache 中间产物

- **WHEN** Agent 调用 `ooxml_unpack` 生成 `.cache/ooxml/<hash>/` 工作目录，并调用 `ooxml_pack` 产出 `report.docx`
- **THEN** 产物列表仅包含 `report.docx`，MUST NOT 包含 `.cache/` 下的任何路径（中间工作目录、渲染缓存、脚本暂存均不视为交付物）

#### Scenario: 目录类产物按路径本身展示

- **WHEN** Agent 经带 `out_dir` 的工具（如 `pdf_split` 输出到 `output`）产出一个目录
- **THEN** 产物列表展示该目录路径本身；MUST NOT 递归展开目录内的子文件。路径格式不携带目录语义（不补尾部斜杠），以免污染 `@` 文件索引

#### Scenario: 目录与文件统一支持打开

- **WHEN** 用户点击目录项 `output` 的「打开」动作
- **THEN** 系统用默认文件管理器打开该目录（与文件用默认程序打开走同一动作）

### Requirement: 构建产物打开与定位

系统 SHALL 为「构建产物」列表项提供两种动作：用默认程序打开文件、在系统文件管理器中定位该文件。打开动作 MUST 复用既有文件打开能力；定位动作 MUST 打开系统文件管理器并尽量定位到该文件。两个动作 MUST 仅对项目根目录内的路径生效。

#### Scenario: 用默认程序打开产物

- **WHEN** 用户在产物列表中点击 `report.docx` 的「打开」动作
- **THEN** 系统以默认关联程序打开该文件（同既有文件打开行为）

#### Scenario: 在文件管理器中定位产物

- **WHEN** 用户在产物列表中点击 `data.xlsx` 的「在文件夹中显示」动作
- **THEN** 系统打开文件管理器并定位到（选中）`data.xlsx`；平台无统一选中语义时（如 Linux）打开其所在目录

#### Scenario: 拒绝越界路径

- **WHEN** 产物路径经校验不在项目根目录内
- **THEN** 打开与定位动作均失败并返回错误，MUST NOT 访问项目根之外的路径
