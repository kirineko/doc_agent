## ADDED Requirements

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
