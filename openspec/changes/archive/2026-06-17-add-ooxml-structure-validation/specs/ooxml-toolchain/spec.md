## MODIFIED Requirements

### Requirement: OOXML 回包与校验
系统 SHALL 提供 `ooxml_pack` 工具，将解包目录回包为文档：XML 压缩回写、自动修复常见问题（durableId/paraId 越界、`w:t` 缺失 `xml:space="preserve"`）、执行 **well-formed XML 校验 + 参照 bundled XSD 的结构规则集校验 + roundtrip 重新解析**。结构规则集 MUST 以 `src-tauri/assets/schemas/` 中 OPC / Word / Presentation / Spreadsheet XSD 类型与元素定义为依据（每条规则标注 XSD 引用与规则 ID）。**不**依赖 libxml2 或任何运行时 XSD 引擎。校验失败 MUST 返回包含部件路径、规则 ID、XSD 引用与具体错误位置的信息，且不产出损坏文件。语义对齐原 skill `pack.py` + `validate.py` 中可拦截的结构错误子集。

#### Scenario: 正常回包
- **WHEN** Agent 对合法修改后的解包目录调用 `ooxml_pack`
- **THEN** 产出可被 Office 打开的文档，且 zip 内含 `[Content_Types].xml`

#### Scenario: 自动修复
- **WHEN** 解包目录中存在 `<w:t> 文本 </w:t>`（带前后空格但缺 `xml:space`）
- **THEN** 回包产物中该元素带有 `xml:space="preserve"`

#### Scenario: 校验失败给出可修复信息
- **WHEN** Agent 写入了嵌套错误的 `w:tbl` 结构后调用 `ooxml_pack`
- **THEN** 工具返回错误，含出错部件（如 `word/document.xml`）、规则 ID（如 `wml.tbl.02`）、XSD 引用（如 `wml.xsd#EG_ContentRowContent`）与错误描述，Agent 可据此修正 XML 后重试

#### Scenario: 表格 XML 生成稳定性
- **WHEN** Agent 按 skill 指引在 document.xml 中插入含 `w:tbl`/`w:tr`/`w:tc`/`w:p` 合法嵌套并回包
- **THEN** 结构规则校验通过，产物在 Office 中表格结构正确

#### Scenario: Content Types 与部件一致
- **WHEN** 解包目录新增 `word/header1.xml` 但未在 `[Content_Types].xml` 登记 Override
- **THEN** `ooxml_pack` 失败，错误引用 `opc-contentTypes.xsd#CT_Types` 相关规则

#### Scenario: 非法表格嵌套被拒绝
- **WHEN** `word/document.xml` 中 `w:tbl/w:tr` 下直接出现 `w:t`（违反 `CT_Row` / `EG_ContentCellContent`）
- **THEN** `ooxml_pack` 失败，错误含 `wml.tr.01` 规则 ID

#### Scenario: pptx presentation 缺少 notesSz 被拒绝
- **WHEN** `ppt/presentation.xml` 缺少 `notesSz`（违反 `pml.xsd#CT_Presentation`）
- **THEN** `ooxml_pack` 失败，错误含 `pml.pres.03` 规则 ID

#### Scenario: xlsx workbook 缺少 sheets 被拒绝
- **WHEN** `xl/workbook.xml` 无 `sheets` 元素（违反 `sml.xsd#CT_Workbook`）
- **THEN** `ooxml_pack` 失败，错误含 `sml.wb.02` 规则 ID

## ADDED Requirements

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
