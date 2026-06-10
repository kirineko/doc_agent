# PDF 页面操作能力

页码约定：全部工具使用 **1-based** 页码（与 lopdf `get_pages()` 一致）；越界页码 MUST 返回明确错误而非静默忽略。输入 / 输出路径均经现有 `Sandbox` 解析，禁止逃逸。

## ADDED Requirements

### Requirement: PDF 合并
系统 SHALL 提供 `pdf_merge` 工具，按给定顺序将多个 PDF 合并为单个 PDF，正确处理对象 ID 重编号与 Pages / Catalog 树合并，输出写入沙箱内指定路径。加密或无法解析的输入 MUST 返回明确错误。

#### Scenario: 顺序合并多个 PDF
- **WHEN** Agent 调用 `pdf_merge {"inputs": ["a.pdf", "b.pdf"], "out_path": "merged.pdf"}`
- **THEN** 生成的 `merged.pdf` 总页数等于各输入页数之和，且页面顺序与 inputs 顺序一致

#### Scenario: 输入为空或单个
- **WHEN** Agent 调用 `pdf_merge` 时 `inputs` 为空数组
- **THEN** 返回「至少需要一个输入 PDF」类错误

#### Scenario: 损坏 / 加密输入报错
- **WHEN** 某个输入 PDF 已加密或无法解析
- **THEN** 返回包含该文件名的明确错误，不产出半成品文件

### Requirement: PDF 拆分
系统 SHALL 提供 `pdf_split` 工具，支持两种模式：①按页范围（如 `"1-3,5"`）导出为单个子 PDF；②`burst` 模式将每页导出为独立 PDF 到指定目录。导出文件 MUST 仅含选定页，且页内容与原文一致。

#### Scenario: 按范围导出子集
- **WHEN** Agent 对 10 页 PDF 调用 `pdf_split {"path": "in.pdf", "ranges": "1-3,5", "out_path": "subset.pdf"}`
- **THEN** `subset.pdf` 含 4 页（原第 1、2、3、5 页），顺序与范围一致

#### Scenario: burst 每页一个文件
- **WHEN** Agent 对 3 页 PDF 调用 `pdf_split {"path": "in.pdf", "mode": "burst", "out_dir": "pages"}`
- **THEN** `pages/` 目录下生成 3 个单页 PDF 文件

#### Scenario: 范围越界报错
- **WHEN** `ranges` 引用了超过总页数的页码
- **THEN** 返回包含越界页码与总页数的明确错误

### Requirement: PDF 页面旋转
系统 SHALL 提供 `pdf_rotate` 工具，对全部或指定页设置旋转角度（90 / 180 / 270，须为 90 的倍数）。支持绝对设置（覆盖 `/Rotate`）与相对累加两种模式，输出写入沙箱内指定路径。

#### Scenario: 旋转全部页
- **WHEN** Agent 调用 `pdf_rotate {"path": "in.pdf", "rotation": 90, "out_path": "rotated.pdf"}`
- **THEN** `rotated.pdf` 每页的 `/Rotate` 为 90

#### Scenario: 旋转指定页
- **WHEN** Agent 调用 `pdf_rotate {"path": "in.pdf", "pages": [2], "rotation": 180, "out_path": "out.pdf"}`
- **THEN** 仅第 2 页被旋转，其余页保持原角度

#### Scenario: 非法角度报错
- **WHEN** `rotation` 不是 90 的倍数（如 45）
- **THEN** 返回「旋转角度必须为 90 的倍数」类错误

### Requirement: PDF 删除页
系统 SHALL 提供 `pdf_delete_pages` 工具，删除指定页并保留其余页，输出写入沙箱内指定路径。删除后若结果为零页 MUST 返回错误而非产出无效 PDF。

#### Scenario: 删除指定页
- **WHEN** Agent 对 5 页 PDF 调用 `pdf_delete_pages {"path": "in.pdf", "pages": [2, 4], "out_path": "out.pdf"}`
- **THEN** `out.pdf` 含 3 页（原第 1、3、5 页）

#### Scenario: 删空报错
- **WHEN** 删除操作会移除全部页
- **THEN** 返回「不能删除所有页」类错误，不产出文件

### Requirement: PDF skill 附属文档
系统 SHALL 在 `assets/skills/pdf/` 提供 `reference.md`（页面操作工具用法、页码约定、与 `pdf_extract_table` / `office_read_to_markdown` / `skill_run`+pdf-lib 的分工）与 `forms.md`（表单处理现状与降级说明），内容面向本系统工具（不含不可执行的外部命令），并可经 `skill_read` 读取。

#### Scenario: 读取附属文档
- **WHEN** Agent 调用 `skill_read {"skill": "pdf", "doc": "reference.md"}`
- **THEN** 返回该文档全文，且其中描述的是本系统 `pdf_*` 工具而非 `pypdf` / `qpdf` 等外部命令

#### Scenario: 文档枚举可见
- **WHEN** Agent 调用 `skill_read {"skill": "pdf"}` 读取主文档
- **THEN** 主文档中引用的附属文档名（reference.md、forms.md）与实际可读取的文档一致
