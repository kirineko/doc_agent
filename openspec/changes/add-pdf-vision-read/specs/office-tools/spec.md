## ADDED Requirements

### Requirement: PDF 双路径读取指引

系统 SHALL 为 PDF 内容理解提供 `pdf_read` 作为首选工具；`office_read_to_markdown` 对 PDF 仍保留 PDFium 纯文本路径。pdf skill 文档 MUST 说明：`pdf_read` 默认（无 mode / `mode=auto`）兼容文本与 vision 模型；仅需纯文本或强制 vision 时显式传 `mode=text` 或 `mode=vision`。

#### Scenario: 有文本层时 office 只读仍可用

- **WHEN** Agent 对 `.pdf` 调用 `office_read_to_markdown`
- **THEN** 行为与变更前一致，返回 PDFium 文本（不自动触发 vision）

#### Scenario: skill 引导 pdf_read 默认 auto

- **WHEN** Agent 加载 pdf skill 处理 PDF
- **THEN** skill 文档指示默认使用 `pdf_read`（无 mode）；含公式/扫描件在 vision 模型上由 auto 自动走图片理解
