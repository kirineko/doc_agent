## 1. 模块结构

- [x] 1.1 将 `validate.rs` 拆为 `validate/mod.rs`（入口）、`validate/wellformed.rs`、`validate/rules/`（`opc.rs`、`wml.rs`、`pml.rs`、`sml.rs`）
- [x] 1.2 `pack.rs` 继续调用统一 `validate_dir`，顺序：well-formed → 结构规则 →（回包后）roundtrip
- [x] 1.3 按部件路径 dispatch：`[Content_Types].xml`、`*.rels`、`word/document.xml`、`ppt/presentation.xml`、`ppt/slides/*.xml`、`xl/workbook.xml`、`xl/worksheets/*.xml`

## 2. 规则基础设施

- [x] 2.1 定义 `ValidationError` 结构：部件路径、规则 ID、XSD 引用、消息、可选行号
- [x] 2.2 实现 namespace 感知 helper（localName + URI 匹配，兼容任意前缀）
- [x] 2.3 实现轻量元素栈/子元素白名单扫描器（基于 `quick-xml` events）

## 3. OPC 规则（design D4 opc.* / pkg.rels.01）

- [x] 3.1 实现 `[Content_Types].xml` 规则 `opc.ct.01`–`opc.ct.05`
- [x] 3.2 实现 `*.rels` 规则 `opc.rels.01`–`opc.rels.04`
- [x] 3.3 实现 `pkg.rels.01`（word/ppt/xl 主入口 `_rels` Internal Target 存在性）

## 4. Word 规则（design D4 wml.*）

- [x] 4.1 对 `word/document.xml`（及可选 `word/glossaryDocument.xml`）dispatch WML 规则
- [x] 4.2 实现 `wml.doc.01`、`wml.body.01`–`wml.body.02`
- [x] 4.3 实现 `wml.tbl.01`–`wml.tbl.02`、`wml.tr.01`、`wml.tc.01`–`wml.tc.02`

## 5. PowerPoint 规则（design D4 pml.*）

- [x] 5.1 对 `ppt/presentation.xml` 实现 `pml.pres.01`–`pml.pres.04`
- [x] 5.2 对 `ppt/slides/slide*.xml` 实现 `pml.sld.01`–`pml.sld.02`

## 6. Excel 规则（design D4 sml.*）

- [x] 6.1 对 `xl/workbook.xml` 实现 `sml.wb.01`–`sml.wb.04`
- [x] 6.2 对 `xl/worksheets/sheet*.xml` 实现 `sml.ws.01`–`sml.ws.04`

## 7. 测试

- [x] 7.1 单元测试：docx / pptx / xlsx 各一个合法最小样例通过
- [x] 7.2 单元测试：每类规则至少一个 FAIL fixture（wml tbl、pml notesSz、sml sheetData、opc ct、rels Id）
- [x] 7.3 `cargo test` + 现有 ooxml 相关测试仍通过

## 8. 收尾

- [x] 8.1 确认无新增 native / XSD 引擎依赖
- [x] 8.2 `openspec validate add-ooxml-structure-validation --strict`
