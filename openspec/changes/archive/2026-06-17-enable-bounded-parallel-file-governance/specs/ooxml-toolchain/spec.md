## MODIFIED Requirements

### Requirement: OOXML 解包

系统 SHALL 提供 `ooxml_unpack` 工具，将项目内 docx / pptx / xlsx 解包为目录：XML 部件 pretty-print 便于编辑、合并相邻同格式 run（可关闭）、智能引号转 XML 实体以保证编辑往返安全。语义对齐原 skill `unpack.py`。

`out_dir` SHALL 变为可选参数。若 Agent 未提供 `out_dir`，系统 MUST 自动生成工作区 `.cache/ooxml/<session_key>/<work_key>/`（`work_key = hash(session_id, turn_id, source_path)`；目录名 MUST NOT 嵌入源文件名 stem），并在工具结果中返回 project-relative `out_dir`。若 Agent 显式提供 `out_dir`，系统 MUST 在删除或创建该目录前申请 `SubtreeWrite(out_dir)` 文件锁；锁冲突时 MUST 拒绝后者，且不得删除已有目录。

#### Scenario: 自动隔离解包 docx

- **WHEN** Agent 对 `report.docx` 调用 `ooxml_unpack` 且不传 `out_dir`
- **THEN** 目标目录位于 `.cache/ooxml/<session_key>/<work_key>/`
- **AND** 工具结果包含该相对 `out_dir`
- **AND** 目录内出现 `word/document.xml` 等部件

#### Scenario: 两个会话自动解包不重名

- **WHEN** 同一 project 中 session A 与 session B 同时对不同 docx/pptx 调用 `ooxml_unpack` 且均不传 `out_dir`
- **THEN** 两个工具返回不同的 `.cache/ooxml/...` 目录
- **AND** 两者互不删除对方目录

#### Scenario: 显式 out_dir 冲突拒绝

- **WHEN** session A 正在写 `unpacked/`
- **AND** session B 调用 `ooxml_unpack {"path":"b.pptx","out_dir":"unpacked/"}`
- **THEN** B 的工具调用失败，错误说明 `unpacked/` 已被占用
- **AND** A 的 `unpacked/` 内容保持不变

#### Scenario: 解包目录可直接用 fs 工具编辑

- **WHEN** Agent 用现有 `fs_read` / `fs_write` 修改工具返回 `out_dir` 中的 XML
- **THEN** 修改后的目录仍可被 `ooxml_pack` 正确回包

### Requirement: OOXML 回包与校验

系统 SHALL 提供 `ooxml_pack` 工具，将解包目录回包为文档：XML 压缩回写、自动修复常见问题、执行 well-formed XML 校验、结构规则集校验与 roundtrip 重新解析。执行前 MUST 申请解包目录 Read lock、`original`（若有）Read lock 与 `out_path` Write lock。

#### Scenario: 回包输出写冲突拒绝

- **WHEN** session A 正在写 `output.docx`
- **AND** session B 调用 `ooxml_pack` 输出同一 `output.docx`
- **THEN** B 的工具调用失败，不产出半成品文件

#### Scenario: 正常回包

- **WHEN** Agent 对合法修改后的解包目录调用 `ooxml_pack`
- **THEN** 产出可被 Office 打开的文档，且 zip 内含 `[Content_Types].xml`
