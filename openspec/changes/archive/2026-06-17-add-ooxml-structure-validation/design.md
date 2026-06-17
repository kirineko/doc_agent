## Context

- 现状：`validate.rs` 遍历解包目录内 `.xml`，用 `quick-xml` 验证 well-formed；检查 `[Content_Types].xml` 存在；`roundtrip_check` 对 xlsx 用 calamine 重开。
- 原 design D4 目标为 libxml2 全量 XSD；spike 结论（2026-06）：
  - **libxml2**：36/39 schema 编译 + 6/6 smoke，但 vcpkg / Windows 静态链接成本高
  - **xmloxide**：36/39 编译，wml/opc-types 实例校验失败（complexContent / Unicode pattern）
  - **fastxml**：36/39 编译，wml/sml 实例校验失败（qualified 命名空间查找）
  - **xsd-parser**：代码生成器，无运行时 validator（planned v1.6+）
- 约束：零 native 依赖；文件体量（Rust ≤500 行/文件）；规则可测试、可逐条扩展

## Goals / Non-Goals

**Goals:**

- 在 `ooxml_pack` 路径上拦截 Agent 在 docx / pptx / xlsx 解包目录中的高频结构错误
- 每条规则可追溯至 bundled XSD 中的类型/元素/组定义
- 错误信息足够 Agent 自行修复（部件路径 + 规则 ID + XSD 引用）

**Non-Goals:**

- 全量 ISO/IEC 29500 XSD 语义校验（枚举全集、所有 facet、drawing/math 深树）
- 重新引入 libxml2 或任何 XSD 引擎 crate
- 修改 `ooxml_unpack`、JS 运行时或 skill 文档（除必要时补充 pack 错误说明）

## Decisions

### D1：校验架构 = well-formed → 结构规则 → roundtrip（不变顺序）

- **理由**：well-formed 最便宜；结构规则针对已知 Agent 错误；roundtrip 兜底 zip/解析器兼容性
- **实现**：`validate_dir` 对每个 `.xml` 先 well-formed，再按部件类型 dispatch 规则集

### D2：规则实现用 `quick-xml` 流式扫描 + 轻量栈，不构建 DOM

- **理由**：与现有代码一致，内存友好，易报行号
- **备选**：roxmltree — 内存更高，否决

### D3：规则 ID 与 XSD 引用格式

- 规则 ID：`{namespace-prefix}.{category}.{seq}`，如 `opc.content-types.01`、`wml.table.03`
- XSD 引用：`{schema文件}#{类型或组}`，如 `ecma/fouth-edition/opc-contentTypes.xsd#CT_Override`
- 错误示例：`word/document.xml:42 [wml.table.02 wml.xsd#EG_ContentRowContent] w:tbl 下不允许直接出现 w:t，期望 w:tr`

### D4：MVP 规则集（参照 XSD，按优先级）

#### OPC — `ecma/fouth-edition/opc-contentTypes.xsd`

| ID | XSD 依据 | 检查内容 |
|----|----------|----------|
| `opc.ct.01` | `element Types` / `CT_Types` | 根元素 localName=`Types`，namespace=`http://schemas.openxmlformats.org/package/2006/content-types` |
| `opc.ct.02` | `CT_Types` choice | 子元素仅 `Default` 或 `Override` |
| `opc.ct.03` | `CT_Default` | 每个 `Default` 必有属性 `Extension`、`ContentType` |
| `opc.ct.04` | `CT_Override` | 每个 `Override` 必有 `PartName`（以 `/` 开头）、`ContentType` |
| `opc.ct.05` | 包一致性 | 解包目录内每个 `.xml` / `.rels` 部件（除 `[Content_Types].xml`）须在 `Override` 或 `Default` 中有对应登记；`PartName` 与相对路径一致 |

#### OPC — `ecma/fouth-edition/opc-relationships.xsd`

| ID | XSD 依据 | 检查内容 |
|----|----------|----------|
| `opc.rels.01` | `CT_Relationships` | 根元素 `Relationships`，namespace=`.../relationships` |
| `opc.rels.02` | `CT_Relationship` | 每个 `Relationship` 必有 `Id`、`Type`、`Target` |
| `opc.rels.03` | `Id` type=`xsd:ID` | 同一文件内 `Id` 不重复；符合 XML Name 字符（不做全 NCName 校验，仅查重复与空值） |
| `opc.rels.04` | `ST_TargetMode` | 若存在 `TargetMode`，值须为 `Internal` 或 `External` |

#### Word — `ISO-IEC29500-4_2016/wml.xsd`（namespace `.../wordprocessingml/2006/main`）

| ID | XSD 依据 | 检查内容 |
|----|----------|----------|
| `wml.doc.01` | `element document` / `CT_Document` | `word/document.xml` 根为 `document`（任意前缀，namespace 正确） |
| `wml.body.01` | `CT_Body` | `body` 下直接子元素须属于块级内容（`p`、`tbl`、`sdt`、`customXml` 等，见 `EG_ContentBlockContent`）；不允许裸 `r`/`t` |
| `wml.body.02` | `CT_Body` sequence | 至多一个 `sectPr`，且若存在须在 `body` 末尾 |
| `wml.tbl.01` | `CT_Tbl` sequence | `tbl` 内在首个 `tr` 之前须出现 `tblPr` 与 `tblGrid`（顺序可含可选 `EG_RangeMarkupElements`） |
| `wml.tbl.02` | `EG_ContentRowContent` | `tbl` 直接子元素（除 tblPr/tblGrid/markup）须为 `tr`（或 sdt/customXml 包裹的行内容），**禁止**直接 `tc`、`p`、`r`、`t` |
| `wml.tr.01` | `CT_Row` / `EG_ContentCellContent` | `tr` 直接子元素须为 `tc`（或 sdt/customXml），**禁止**直接 `t`/`p`/`r` |
| `wml.tc.01` | `CT_Tc` sequence | `tc` 在可选 `tcPr` 之后须至少有一个块级内容（`p`/`tbl`/…，`EG_BlockLevelElts`），不能为空 |
| `wml.tc.02` | `CT_Tc` | `tc` 内不允许直接出现 `tr` |

#### 跨部件（轻量）

| ID | XSD / OPC 惯例 | 检查内容 |
|----|----------------|----------|
| `pkg.rels.01` | OPC | 各包 `_rels/*.rels` 中 `TargetMode=Internal`（或缺省 Internal）的 `Target` 在解包目录中存在；覆盖 `word/`、`ppt/`、`xl/` 主入口 rels |

#### PowerPoint — `ISO-IEC29500-4_2016/pml.xsd`（namespace `.../presentationml/2006/main`）

| ID | XSD 依据 | 检查内容 |
|----|----------|----------|
| `pml.pres.01` | `element presentation` / `CT_Presentation` | `ppt/presentation.xml` 根为 `presentation`（namespace 正确） |
| `pml.pres.02` | `CT_Presentation` sequence | 子元素须为 XSD 序列允许项（`sldMasterIdLst`、`sldIdLst`、`sldSz`、`notesSz` 等）；禁止未知 localName |
| `pml.pres.03` | `CT_Presentation` / `notesSz` minOccurs=1 | 必须存在恰好一个 `notesSz` |
| `pml.pres.04` | `CT_SlideSize` | 若存在 `sldSz`，必有属性 `cx`、`cy`（`ST_SlideSizeCoordinate`） |
| `pml.sld.01` | `element sld` / `CT_Slide` | `ppt/slides/slide*.xml` 根为 `sld` |
| `pml.sld.02` | `CT_Slide` sequence | `sld` 下首个子元素须为 `cSld`（minOccurs=1）；禁止在 `cSld` 之前出现其他元素 |

#### Excel — `ISO-IEC29500-4_2016/sml.xsd`（namespace `.../spreadsheetml/2006/main`）

| ID | XSD 依据 | 检查内容 |
|----|----------|----------|
| `sml.wb.01` | `element workbook` / `CT_Workbook` | `xl/workbook.xml` 根为 `workbook` |
| `sml.wb.02` | `CT_Workbook` sequence | 必须存在恰好一个 `sheets`（minOccurs=1） |
| `sml.wb.03` | `CT_Sheets` / `CT_Sheet` | `sheets` 下至少一个 `sheet`；每个 `sheet` 必有 `name`、`sheetId`、`r:id` |
| `sml.wb.04` | `ST_SheetState` | 若 `sheet/@state` 存在，值须为 `visible` / `hidden` / `veryHidden` |
| `sml.ws.01` | `element worksheet` / `CT_Worksheet` | `xl/worksheets/sheet*.xml` 根为 `worksheet` |
| `sml.ws.02` | `CT_Worksheet` sequence | 必须存在恰好一个 `sheetData`（minOccurs=1） |
| `sml.ws.03` | `CT_SheetData` | `sheetData` 直接子元素仅允许 `row` |
| `sml.ws.04` | `CT_Row` | `row` 直接子元素仅允许 `c`（及可选 `extLst`）；禁止裸文本或其他标签 |

### D5：后续扩展（本 change 不实现）

- `w:gridSpan` / `vMerge` 与 `w:tblGrid` 列数一致性（wml）
- pptx `slideLayout` / `slideMaster` 深树与 drawingml 嵌套
- xlsx 公式语法与 `sharedStrings` 交叉引用

## Risks / Trade-offs

- [规则不全] → 按 Agent 失败样本迭代；XSD 文件保留作对照
- [误报] → 规则仅检查 localName + namespace URI，对 sdt/customXml 白名单与 XSD choice 对齐
- [性能] → 流式单次扫描；大 document 可接受（通常 < 数 MB）

## Migration Plan

- 无数据迁移；`ooxml_pack` 行为更严格，此前能通过的结构错误将失败并返回可修复信息
- 若阻塞现有测试/fixture，更新 fixture XML 或收窄规则

## Open Questions

- 是否在首版对 `ppt/slideMasters/`、`ppt/slideLayouts/` 做与 `pml.sld.*` 同级的轻量根元素检查（默认纳入 `pml.sld.01` 同类规则，按路径 dispatch）
