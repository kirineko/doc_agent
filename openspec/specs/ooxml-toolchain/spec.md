# ooxml-toolchain Specification

## Purpose
TBD - created by archiving change add-document-skills-runtime. Update Purpose after archive.
## Requirements
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

### Requirement: OOXML 结构规则 MVP 范围
系统 SHALL 在 `ooxml_pack` 校验阶段实现 design.md D4 中全部规则组：**OPC**（`opc.ct.*`、`opc.rels.*`）、**Word**（`wml.*`）、**PowerPoint**（`pml.pres.*`、`pml.sld.*`）、**Excel**（`sml.wb.*`、`sml.ws.*`），以及 **Internal relationship Target 存在性**（`pkg.rels.01`，覆盖 docx/pptx/xlsx 主入口 rels）。

#### Scenario: Relationships Id 重复被拒绝
- **WHEN** 某 `.rels` 文件中两个 `Relationship` 使用相同 `Id`
- **THEN** `ooxml_pack` 失败，错误引用 `opc-relationships.xsd#CT_Relationship` 与 `opc.rels.03`

#### Scenario: tbl 缺少 tblGrid 被拒绝
- **WHEN** `w:tbl` 含有 `w:tr` 但缺少 `w:tblGrid`（违反 `wml.xsd#CT_Tbl` 序列）
- **THEN** `ooxml_pack` 失败，错误引用 `wml.tbl.01`

#### Scenario: worksheet 缺少 sheetData 被拒绝
- **WHEN** `xl/worksheets/sheet1.xml` 无 `sheetData`（违反 `sml.xsd#CT_Worksheet`）
- **THEN** `ooxml_pack` 失败，错误引用 `sml.ws.02`

#### Scenario: sheet 行下出现非法子元素被拒绝
- **WHEN** `sheetData/row` 下直接出现非 `c` 元素（违反 `sml.xsd#CT_Row`）
- **THEN** `ooxml_pack` 失败，错误引用 `sml.ws.04`

#### Scenario: slide 缺少 cSld 被拒绝
- **WHEN** `ppt/slides/slide1.xml` 的 `sld` 下无 `cSld` 或 `cSld` 不在首位（违反 `pml.xsd#CT_Slide`）
- **THEN** `ooxml_pack` 失败，错误引用 `pml.sld.02`

### Requirement: Word 批注注入
系统 SHALL 提供 `docx_comment` 工具，向解包目录注入批注（含回复），自动维护 comments.xml 及关联部件与 `[Content_Types].xml` 注册，作者默认 "Claude" 可定制。语义对齐原 skill `comment.py`。

#### Scenario: 添加批注
- **WHEN** Agent 调用 `docx_comment` 注入 id=0 的批注，并在 document.xml 加入对应 range 标记后回包
- **THEN** 产物在 Office 中显示该批注

### Requirement: 接受全部修订
系统 SHALL 提供 `docx_accept_changes` 工具，以纯 XML 变换接受文档全部修订（应用 `w:ins` 内容、移除 `w:del` 及段落删除标记），不依赖 LibreOffice。

#### Scenario: 接受修订产出干净文档
- **WHEN** Agent 对含插入与删除修订的 docx 调用 `docx_accept_changes`
- **THEN** 产物不再含 `w:ins` / `w:del` 元素，文本为修订接受后的结果

### Requirement: 旧格式编辑降级
系统 SHALL 在对 `.doc` / `.ppt` / `.xls` 旧格式调用解包或编辑类工具时返回明确错误，提示用户先将文件另存为新格式（读取能力不受影响）。

#### Scenario: 旧格式提示
- **WHEN** Agent 对 `legacy.doc` 调用 `ooxml_unpack`
- **THEN** 返回错误信息，说明需先转换为 `.docx`

