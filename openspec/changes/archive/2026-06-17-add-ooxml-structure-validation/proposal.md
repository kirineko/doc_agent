## Why

`ooxml_pack` 目前仅做 well-formed XML 与 roundtrip，无法拦截 Agent 常见的 OOXML 结构错误（如 `w:tbl` 嵌套错误、`[Content_Types].xml` 与部件不一致）。全量 XSD 路线（libxml2、xmloxide、fastxml、xsd-parser）经 spike 评估后均因构建成本或 OOXML 覆盖缺口不适合纳入产品；现正式采用 design D4 退路：**参照 bundled ISO/IEC 29500 / OPC XSD 编写针对性结构规则**，在零 native 依赖前提下提升回包稳定性。

## What Changes

- 在 `tools/ooxml/validate.rs` 增加 **结构规则校验层**（保留现有 well-formed + roundtrip）
- 规则集以 `src-tauri/assets/schemas/` 中 XSD 类型/元素/序列为依据，每条规则标注 XSD 出处（如 `wml.xsd#CT_Tbl`）
- MVP 覆盖：**OPC 包级** + **Word**（`document.xml`）+ **PowerPoint**（`presentation.xml`、`slides/*.xml`）+ **Excel**（`workbook.xml`、`worksheets/*.xml`）
- 校验失败返回：`部件相对路径` + `规则 ID` + `XSD 引用` + `人类可读说明` + 可选行号
- **不引入** libxml2 / xmloxide / fastxml / xsd-parser 等新依赖
- **不删除** `assets/schemas/`（仍作规则来源文档与后续扩展参考）

## Capabilities

### New Capabilities

（无 — 能力归入现有 `ooxml-toolchain`）

### Modified Capabilities

- `ooxml-toolchain`：将「XSD 或等效规则集」具体化为「well-formed + 参照 XSD 的结构规则集 + roundtrip」；明确 MVP 规则范围与错误格式

## Impact

- **代码**：`src-tauri/src/tools/ooxml/validate.rs` 拆分为 `validate/` 模块（well-formed、rules、roundtrip）；新增规则实现与单元测试
- **依赖**：无新增 Cargo 依赖（继续 `quick-xml`、`walkdir`、`zip`、`calamine`）
- **Agent 行为**：非法结构在 `ooxml_pack` 阶段失败，错误信息可指导修正 XML
- **排除**：全量 39 XSD 运行时校验、drawing/math 深树与表格合并单元格高级约束（见 design.md D5）
