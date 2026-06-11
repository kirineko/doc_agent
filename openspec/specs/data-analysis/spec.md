# data-analysis Specification

## Purpose
TBD - created by archiving change add-document-skills-runtime. Update Purpose after archive.
## Requirements
### Requirement: Word 表格提取
系统 SHALL 提供 `docx_extract_table` 工具，解析 docx 中的 `w:tbl` 结构，将指定（或全部）表格导出为沙箱内 CSV 文件，正确处理合并单元格（gridSpan / vMerge 以空值或重复值策略展开）。

#### Scenario: 提取表格为 CSV
- **WHEN** Agent 对含 2 个表格的 docx 调用 `docx_extract_table`
- **THEN** 沙箱内生成 2 个 CSV 文件，行列内容与文档中表格一致

### Requirement: PDF 表格提取
系统 SHALL 提供 `pdf_extract_table` 工具，对原生文本型 PDF 按 pdfplumber 同类策略（lines / text 等）提取表格为 CSV；扫描件（无文本层）MUST 返回明确错误而非空结果。

#### Scenario: 提取有线框表格
- **WHEN** Agent 对含网格线表格的 PDF 调用 `pdf_extract_table`
- **THEN** 生成的 CSV 行列与 PDF 中表格一致

#### Scenario: 扫描件明确报错
- **WHEN** Agent 对扫描版 PDF 调用该工具
- **THEN** 返回「无文本层，需 OCR（不在当前能力范围）」类错误

### Requirement: SQL 数据整理
系统 SHALL 提供 `data_query` 工具，对沙箱内 CSV / xlsx / **xls** 数据源执行 SQL（polars-sql），支持过滤、聚合、排序、UNION 等整理操作，结果写回沙箱内 CSV 或直接返回（小结果集）。

#### Scenario: 聚合查询
- **WHEN** Agent 以某 CSV 为源执行 `SELECT category, SUM(amount) FROM t GROUP BY category`
- **THEN** 返回聚合结果，数值类型正确

#### Scenario: xlsx 作为数据源
- **WHEN** Agent 以某 `.xlsx` 的指定工作表为源执行查询
- **THEN** 系统经 calamine 读取并完成查询，无需先手工转 CSV

#### Scenario: xls 作为数据源
- **WHEN** Agent 以某 `.xls` 的指定工作表为源执行查询
- **THEN** 系统读取该旧格式表格并完成查询，无需 Agent 事先手动另存为 xlsx

### Requirement: 公式重算校验
系统 SHALL 提供 `xlsx_recalc` 工具，用 IronCalc 载入 xlsx 并重算全部公式，报告各单元格公式错误（#REF!、#DIV/0!、#VALUE!、#N/A）；IronCalc 不支持的函数导致的 #NAME? SHALL 作为 warning 而非错误返回。

#### Scenario: 检出公式错误
- **WHEN** Agent 对含 `=1/0` 公式的 xlsx 调用 `xlsx_recalc`
- **THEN** 返回结果中列出该单元格地址与 `#DIV/0!` 错误

#### Scenario: 零错误通过
- **WHEN** Agent 对公式全部合法的 xlsx 调用 `xlsx_recalc`
- **THEN** 返回 `errors: []`，可作为交付前验收依据

### Requirement: 端到端提取-整理-输出管道
系统 SHALL 支持如下闭环：从 Word / PDF 提取表格 → `data_query` 整理汇总 → 经 `skill_run`（exceljs）或 Excel 工具输出样式化表格，全程中间产物为沙箱内文件，Agent 可用现有 fs 工具检查。

#### Scenario: PDF 到美观 Excel
- **WHEN** Agent 依次调用 `pdf_extract_table` → `data_query`（汇总）→ `skill_run`（exceljs 样式化输出）
- **THEN** 最终产物 `.xlsx` 含汇总数据与样式，且 `xlsx_recalc` 校验零错误

