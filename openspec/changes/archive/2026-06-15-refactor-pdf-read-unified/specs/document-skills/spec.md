## ADDED Requirements

### Requirement: pdf skill 反映统一 pdf_read

内置 pdf skill（`SKILL.md`、`reference.md`）MUST 将 `pdf_read` 描述为仅传 `path` 的智能读取；MUST NOT 文档化已移除的 `mode` 参数。MUST 说明 `office_read_to_markdown` 用于显式纯 PDFium 快速读取。

#### Scenario: reference 无 mode 示例

- **WHEN** Agent 通过 `skill_read` 加载 `pdf/reference.md`
- **THEN** `pdf_read` 示例仅为 `{ "path": "doc.pdf" }` 形式
