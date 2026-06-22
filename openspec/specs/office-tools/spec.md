# office-tools Specification

## Purpose

定义 Agent 对 Word、Excel、PPT 的读取、写入与编辑能力，含旧版 Office 转换入口。PPT 生成与模板编辑经 Document Skills（`skill_run` + pptxgenjs / OOXML 解包回包）实现；任意 Office 格式均可读取为 Markdown 供模型理解。
## Requirements
### Requirement: PPT 生成与编辑
系统 SHALL 支持在项目目录内生成新 PPT（经脚本运行时 pptxgenjs）及基于既有 PPT 模板的内容编辑（经 OOXML 解包 / 回包），产物 MUST 为可被 Office 打开的合法 OOXML。

#### Scenario: 从零生成 PPT
- **WHEN** Agent 按 pptx skill 指引经 `skill_run` 生成多页演示文稿
- **THEN** 项目内生成 `.pptx`，可被 Office 正常打开且页面内容正确

#### Scenario: 基于模板编辑
- **WHEN** Agent 对既有 `.pptx` 解包、替换文本与图片引用后回包
- **THEN** 产物保留模板母版与样式，仅目标内容被修改

### Requirement: 任意 Office 文档读取为 Markdown
系统 SHALL 提供工具，将项目目录内的 Word / Excel / PPT（含旧格式 doc / xls / ppt）读取并转换为 Markdown，供 Agent 作为上下文。

#### Scenario: 读取 Word 为 Markdown
- **WHEN** Agent 对项目内一个 `.docx` 调用读取工具
- **THEN** 返回该文档的 Markdown 文本（标题、段落、列表、表格结构保留）

#### Scenario: 读取 PPT 为 Markdown
- **WHEN** Agent 对项目内一个 `.pptx` 调用读取工具
- **THEN** 返回每页文本内容的 Markdown（MVP 仅读取，不要求生成）

### Requirement: Word 文档保格式编辑
系统 SHALL 提供工具，对项目内既有 Word 文档进行文本替换式编辑，并保留未改动的部件（图片、样式、关系等）。

#### Scenario: 替换文本并保留格式
- **WHEN** Agent 请求把某 `.docx` 中的「季度报告」替换为「年度报告」
- **THEN** 系统完成替换并另存，未改动内容的格式被保留，返回替换计数

### Requirement: Excel 读取
系统 SHALL 提供工具，读取项目内 Excel 工作表的单元格数据。

#### Scenario: 读取工作表
- **WHEN** Agent 对一个 `.xlsx` 调用读取工具并指定工作表
- **THEN** 返回该表的行列单元格内容（含数字与文本）

### Requirement: Excel 写入
系统 SHALL 提供工具，在项目内创建或修改 Excel，写入单元格值（文本 / 数字）。

#### Scenario: 写入单元格并保存
- **WHEN** Agent 请求在某工作表写入若干单元格并保存
- **THEN** 系统生成 / 更新 `.xlsx`，文件为可被 Office 打开的合法 OOXML

### Requirement: 旧版 Office 经 Agent 工具转换
系统 SHALL 在 `office-tools` 能力域注册 `office_convert` 工具（详见 `legacy-office-convert` 能力），供 Agent 将 `.doc/.xls/.ppt` 转为现代 OOXML；输出文件名 MUST 带 `-converted` 后缀以区别于用户手动另存为的文件。

#### Scenario: Word 旧格式转换
- **WHEN** Agent 对 `memo.doc` 调用 `office_convert`
- **THEN** 生成 `memo-converted.docx` 且可被 Office 打开

#### Scenario: Excel 旧格式转换
- **WHEN** Agent 对 `数据.xls` 调用 `office_convert`
- **THEN** 生成 `数据-converted.xlsx` 且可被 Office 打开

#### Scenario: PPT 旧格式转换
- **WHEN** Agent 对 `slides.ppt` 调用 `office_convert`
- **THEN** 生成 `slides-converted.pptx` 且可被 Office 打开

### Requirement: PDF 读取工具分工

系统 SHALL 为 PDF 内容理解提供 `pdf_read` 作为默认智能入口（无 `mode`，内部 Judge 决定是否 vision）。`office_read_to_markdown` 对 PDF MUST 保留 PDFium 纯文本路径，供明确只需快速文本、不需 Judge/vision 的场景。

pdf skill 文档 MUST 说明：一般读 PDF 仅 `pdf_read({"path": "..."})`；仅要 PDFium 时用 `office_read_to_markdown`。

#### Scenario: office 纯文本路径不变

- **WHEN** Agent 对 `.pdf` 调用 `office_read_to_markdown`
- **THEN** 行为与变更前一致，不触发 Judge 或 vision

#### Scenario: skill 引导 pdf_read 无 mode

- **WHEN** Agent 加载 pdf skill
- **THEN** 文档不包含 `pdf_read` 的 `mode` 参数说明

