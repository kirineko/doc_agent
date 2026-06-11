## ADDED Requirements

### Requirement: 工具结果携带变更路径
系统 SHALL 在 Agent 循环执行文件变更类工具成功后，于 `tool_result` 事件中携带 `changed_paths` 字段（相对项目根的路径字符串数组，POSIX 分隔符）；前端据此增量更新文件索引。失败或只读工具 MUST NOT 携带有效变更路径。

#### Scenario: 写文件工具返回路径
- **WHEN** `fs_write` 成功写入 `notes/todo.md`
- **THEN** 对应 `tool_result` 的 `changed_paths` 包含 `notes/todo.md`

#### Scenario: 解压工具返回目录而非内部 XML
- **WHEN** `ooxml_unpack` 成功解压到 `unpacked/`
- **THEN** `changed_paths` 包含 `unpacked/`（或等效目录路径），MUST NOT 枚举 `unpacked/word/*.xml` 等内部部件

#### Scenario: skill_run 追踪 doc_write
- **WHEN** `skill_run` 内脚本通过 `doc_write` / `__doc_write` 写入 `out.xlsx`
- **THEN** `tool_result.changed_paths` 包含 `out.xlsx`

#### Scenario: 工具失败无变更路径
- **WHEN** 某写文件工具执行失败
- **THEN** `tool_result` 的 `changed_paths` 为空或省略，且 `ok` 为 false
