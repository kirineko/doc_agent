# PDF 表单（AcroForm）现状

## 当前能力

doc-agent **尚未提供** AcroForm 自动填值工具（无 `pdf_fill_form` 等）。下列能力**不在**当前版本：

- 读取可填写字段列表
- 批量写入表单域
- 扁平化（flatten）表单
- 基于 bounding box 的注释式填写

## 降级建议

1. **只读了解**：用 `office_read_to_markdown` 查看 PDF 可见文本；复杂表单结构可能不完整。
2. **人工填写**：将 PDF 交给用户在外部工具（Acrobat、Preview 等）填写后，再继续合并 / 拆分等页面操作。
3. **生成替代物**：若目标是交付带数据的文档，可用 `skill_run` + pdf-lib **新建** PDF 并直接绘制文本，而非填写既有表单域。
4. **表格数据**：用 `office_read_to_markdown` 读取可见文本后手工整理。

## 后续

表单填值列入后续增量（可能结合 pdf-lib 或专用 Rust crate）。在此之前，请勿假设存在任何外部 PDF 命令行或 Python 库。
