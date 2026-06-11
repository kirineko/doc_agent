# docx-style-lint Spec Delta

## ADDED Requirements

### Requirement: docx 产物样式检查
系统 SHALL 在 `skill_run` 执行结束后，对本次写出的每个 `.docx` 文件做确定性样式检查（OOXML 文本层规则），并将告警以 `style_warnings`（按文件路径分组的中文消息数组）并入工具响应返回给 Agent。检查失败（IO / 解析错误）MUST 静默跳过，不得影响工具调用的成功结果。

#### Scenario: 无标题的长文档触发告警
- **WHEN** `skill_run` 写出一个正文超过 600 字且不含任何 `Heading` 段落样式的 `.docx`
- **THEN** 工具响应包含 `style_warnings`，其中有「缺少标题分层」的中文告警与修复指引

#### Scenario: 含中文但未配置 eastAsia 字体触发告警
- **WHEN** 写出的 `.docx` 正文含 CJK 字符，且 `styles.xml` 与 `document.xml` 中均无 `w:eastAsia` 字体声明
- **THEN** `style_warnings` 包含「未配置中文字体，将发生字体回退」的告警

#### Scenario: 合格文档不产生告警
- **WHEN** 写出的 `.docx` 配置了 eastAsia 字体、使用 Heading 样式分层、无超长段落、列表使用 numbering、表格设置了宽度
- **THEN** 工具响应不包含 `style_warnings` 字段

#### Scenario: lint 自身异常不阻断工具结果
- **WHEN** 写出的 `.docx` 无法被 lint 模块解析（如脚本故意产出非常规 zip）
- **THEN** 工具响应正常返回脚本结果，不包含 lint 错误，也不报工具失败

### Requirement: 样式检查规则集
首版规则集 SHALL 覆盖以下五类问题，阈值以常量定义便于调整：W1 正文超过 600 字且无 Heading 样式；W2 含 CJK 字符但无 eastAsia 字体声明；W3 单段落文本超过 500 字；W4 段落文本以手打项目符号（`•` `·` `●` 或「数字.」）开头且无 `numPr`；W5 `<w:tbl>` 缺少 `tblW` 或 `gridCol` 宽度定义。

#### Scenario: 手打 bullet 触发告警
- **WHEN** 某段落文本以 `•` 开头且该段无 `<w:numPr>` 配置
- **THEN** `style_warnings` 包含「手工项目符号应改用 numbering 配置」的告警

#### Scenario: 表格缺宽度触发告警
- **WHEN** 文档中存在未设置 `tblW` 或 `gridCol` 的表格
- **THEN** `style_warnings` 包含「表格未设置宽度，跨平台渲染会变形」的告警
