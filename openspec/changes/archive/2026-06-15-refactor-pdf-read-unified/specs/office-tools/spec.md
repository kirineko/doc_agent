## ADDED Requirements

### Requirement: PDF 读取工具分工

系统 SHALL 为 PDF 内容理解提供 `pdf_read` 作为默认智能入口（无 `mode`，内部 Judge 决定是否 vision）。`office_read_to_markdown` 对 PDF MUST 保留 PDFium 纯文本路径，供明确只需快速文本、不需 Judge/vision 的场景。

pdf skill 文档 MUST 说明：一般读 PDF 仅 `pdf_read({"path": "..."})`；仅要 PDFium 时用 `office_read_to_markdown`。

#### Scenario: office 纯文本路径不变

- **WHEN** Agent 对 `.pdf` 调用 `office_read_to_markdown`
- **THEN** 行为与变更前一致，不触发 Judge 或 vision

#### Scenario: skill 引导 pdf_read 无 mode

- **WHEN** Agent 加载 pdf skill
- **THEN** 文档不包含 `pdf_read` 的 `mode` 参数说明
