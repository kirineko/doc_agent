# OOXML 工具链能力

## ADDED Requirements

### Requirement: OOXML 解包
系统 SHALL 提供 `ooxml_unpack` 工具，将项目内 docx / pptx / xlsx 解包为目录：XML 部件 pretty-print 便于编辑、合并相邻同格式 run（可关闭）、智能引号转 XML 实体以保证编辑往返安全。语义对齐原 skill `unpack.py`。

#### Scenario: 解包 docx
- **WHEN** Agent 对 `report.docx` 调用 `ooxml_unpack`
- **THEN** 目标目录出现 `word/document.xml` 等部件，XML 已格式化且相邻同格式 run 已合并

#### Scenario: 解包目录可直接用 fs 工具编辑
- **WHEN** Agent 用现有 `fs_read` / `fs_write` 修改解包目录中的 XML
- **THEN** 修改后的目录仍可被 `ooxml_pack` 正确回包

### Requirement: OOXML 回包与校验
系统 SHALL 提供 `ooxml_pack` 工具，将解包目录回包为文档：XML 压缩回写、自动修复常见问题（durableId/paraId 越界、`w:t` 缺失 `xml:space="preserve"`）、执行结构校验（XSD 或等效规则集 + roundtrip 重新解析）。校验失败 MUST 返回包含部件路径与具体错误位置的信息，且不产出损坏文件。语义对齐原 skill `pack.py` + `validate.py`。

#### Scenario: 正常回包
- **WHEN** Agent 对合法修改后的解包目录调用 `ooxml_pack`
- **THEN** 产出可被 Office 打开的文档，且 zip 内含 `[Content_Types].xml`

#### Scenario: 自动修复
- **WHEN** 解包目录中存在 `<w:t> 文本 </w:t>`（带前后空格但缺 `xml:space`）
- **THEN** 回包产物中该元素带有 `xml:space="preserve"`

#### Scenario: 校验失败给出可修复信息
- **WHEN** Agent 写入了嵌套错误的 `w:tbl` 结构后调用 `ooxml_pack`
- **THEN** 工具返回错误，含出错部件（如 `word/document.xml`）与错误描述，Agent 可据此修正 XML 后重试

#### Scenario: 表格 XML 生成稳定性
- **WHEN** Agent 按 skill 指引在 document.xml 中插入含合并单元格与列宽定义的 `w:tbl` 并回包
- **THEN** 校验通过，产物在 Office 中表格结构与列宽正确

### Requirement: Word 批注注入
系统 SHALL 提供 `docx_comment` 工具，向解包目录注入批注（含回复），自动维护 comments.xml 及关联部件与 `[Content_Types].xml` 注册，作者默认 "Claude" 可定制。语义对齐原 skill `comment.py`。

#### Scenario: 添加批注
- **WHEN** Agent 调用 `docx_comment` 注入 id=0 的批注，并在 document.xml 加入对应 range 标记后回包
- **THEN** 产物在 Office 中显示该批注

### Requirement: 接受全部修订
系统 SHALL 提供 `docx_accept_changes` 工具，以纯 XML 变换接受文档全部修订（应用 `w:ins` 内容、移除 `w:del` 及段落删除标记），不依赖 LibreOffice。

#### Scenario: 接受修订产出干净文档
- **WHEN** Agent 对含插入与删除修订的 docx 调用 `docx_accept_changes`
- **THEN** 产物不再含 `w:ins` / `w:del` 元素，文本为修订接受后的结果

### Requirement: 旧格式编辑降级
系统 SHALL 在对 `.doc` / `.ppt` / `.xls` 旧格式调用解包或编辑类工具时返回明确错误，提示用户先将文件另存为新格式（读取能力不受影响）。

#### Scenario: 旧格式提示
- **WHEN** Agent 对 `legacy.doc` 调用 `ooxml_unpack`
- **THEN** 返回错误信息，说明需先转换为 `.docx`
