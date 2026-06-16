## ADDED Requirements

### Requirement: 用户导入文件至项目根

系统 SHALL 允许用户通过 Chat 输入区 **+** 按钮，从操作系统文件对话框**多选**本地文件，将其复制到当前 active 项目的**根目录**（相对路径为 `./文件名`）。允许**任意扩展名**；单文件大小 MUST NOT 超过 100MB。导入 MUST 经 sandbox 校验，禁止路径穿越（文件名 MUST NOT 含 `/`、`\` 或 `..`）。

#### Scenario: 多文件导入成功

- **WHEN** 用户已选项目、选择 `a.docx` 与 `b.pdf` 且根目录不存在同名文件
- **THEN** 两文件写入项目根、文件索引刷新、输入框光标处插入 `@a.docx @b.pdf `（顺序与选择一致）

#### Scenario: 无项目时阻断

- **WHEN** 用户未选择项目并点击 **+**
- **THEN** 展示与发送阻断一致的引导（不打开文件对话框或打开后立即提示），不写入磁盘

#### Scenario: 超大文件拒绝

- **WHEN** 用户选择的文件超过 100MB
- **THEN** 该文件导入失败并展示明确错误，不影响同批其他文件（若已选多个）

### Requirement: 导入文件名冲突处理

当目标路径已存在时，系统 MUST 询问用户：**覆盖**、**另存为**、**取消**。多文件导入时 MUST **逐文件**询问。选择**另存为**时，系统 MUST 自动递增生成 `文件名 (1).ext`、`文件名 (2).ext` … 直至不冲突，并使用最终路径刷新索引与 `@` 插入。

#### Scenario: 覆盖已有文件

- **WHEN** 根目录已有 `report.docx` 且用户选择覆盖
- **THEN** 新内容写入 `report.docx`，索引更新，输入框插入 `@report.docx `

#### Scenario: 另存为自动递增

- **WHEN** 根目录已有 `report.docx` 且用户选择另存为
- **THEN** 写入 `report (1).docx`（若仍存在则递增为 `(2)` 等），输入框插入 `@report (1).docx `（含引号规则若路径含空格）

#### Scenario: 取消跳过该文件

- **WHEN** 用户对某冲突文件选择取消
- **THEN** 该文件不写入，继续处理队列中剩余文件

### Requirement: 导入后 @ 索引与输入框联动

导入成功的每个文件 MUST 合并进 `@` 文件索引（`list_project_files` 规则一致），并 MUST 在输入框**当前光标位置**插入 `@相对路径`（多个路径空格分隔，末尾留空格）；MUST NOT 自动发送消息。路径格式化 MUST 复用 `formatMentionPath`（含空格/标点引号包裹）。

#### Scenario: 光标在中间插入

- **WHEN** 输入框内容为 `请分析 |光标|` 且用户导入 `data.xlsx`
- **THEN** 变为 `请分析 @data.xlsx |光标|`（`|` 表示光标，不强制换行）

#### Scenario: 导入文件可被 @ 弹层搜到

- **WHEN** 导入完成后用户键入 `@data`
- **THEN** `@` 候选包含刚导入的 `data.xlsx`

#### Scenario: busy 或 clarify 时禁用导入

- **WHEN** 会话 busy、initializing 或存在 pending clarify
- **THEN** **+** 按钮 disabled，不触发导入
