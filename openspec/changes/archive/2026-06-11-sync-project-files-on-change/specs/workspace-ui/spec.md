## ADDED Requirements

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

## MODIFIED Requirements

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
