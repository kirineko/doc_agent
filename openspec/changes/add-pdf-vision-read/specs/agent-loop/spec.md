## ADDED Requirements

### Requirement: PDF vision 工具注册

系统 SHALL 在默认工具列表中注册 `pdf_render_pages` 与 `pdf_read`（所有模型可见）。`pdf_read` 默认 `mode=auto`：非 vision 会话走 PDFium 文本分支，vision 会话在文本提取后再走 vision 子调用；不要求非 vision 会话显式传 `mode=text`。

`pdf_read` vision 路径分批理解时 MAY 内部调用共享 vision helper 或已注册的 `image_read` 逻辑，每批图片数 MUST NOT 超过 4。

#### Scenario: 非 vision 会话可见 pdf_read

- **WHEN** 会话模型为 DeepSeek V4 Flash
- **THEN** 工具列表含 `pdf_read` 与 `pdf_render_pages`，不含 `image_read`

#### Scenario: vision 会话全套 PDF 工具

- **WHEN** 会话模型为 MiMo v2.5
- **THEN** 工具列表含 `pdf_read`、`pdf_render_pages` 与 `image_read`
