## MODIFIED Requirements

### Requirement: Word 批注注入
系统 SHALL 提供 `docx_comment` 工具，向解包目录注入批注（含回复）。工具**自行**完成全部部件装配，不得把任何锚点/注册工作转嫁给调用方：
- 在 `word/comments.xml` 写入以给定 `id` 为 `w:id` 的 `<w:comment>`，含 `w:author`（缺省 "Claude"）、`w:date`、`w:initials`，正文段落承载批注文本；
- 在 `word/document.xml` 目标段落内插入 `<w:commentRangeStart w:id="X"/>` / `<w:commentRangeEnd w:id="X"/>` 及含 `<w:commentReference w:id="X"/>` 的 run（`X` = 入参 `id`），使批注附着到正文；
- 目标段落由入参 `paragraph_index`（0-based，对 `document.xml` 顶层 `<w:p>` 计数）指定，可选 `text_hint` 对该段落纯文本做断言式校验；
- 当 `parent` 提供时，在 `word/commentsExtended.xml` 写入 `<w15:commentEx>` 建立回复链；
- 自动维护 `people.xml` 及 `[Content_Types].xml` / 关系文件的注册（缺则建、有则去重）。

对 `word/comments.xml` 为自闭合空壳（`<w:comments/>`）的常见形态也必须正确写入——不得因闭合标签缺失而静默丢弃条目。

#### Scenario: 添加批注（含正文锚点）
- **WHEN** Agent 调用 `docx_comment`（id=1, paragraph_index=N, text="建议明确付款方式", author="审阅人"）注入批注并回包
- **THEN** 打包后 `word/comments.xml` 含 `<w:comment w:id="1">`，且 `word/document.xml` 第 N 段内含 `commentRangeStart w:id="1"` 与 `commentReference w:id="1"`，产物在 Office 中显示该批注

#### Scenario: 自闭合空壳 comments.xml 也能写入
- **WHEN** 文档由 docx-js 生成、`word/comments.xml` 为 `<w:comments .../>`，调用 `docx_comment`（id=1）
- **THEN** 写入后 `comments.xml` 不再是自闭合空壳，含 `<w:comment w:id="1">` 条目

#### Scenario: 段落定位未命中报错
- **WHEN** `paragraph_index` 越界，或 `text_hint` 与目标段落纯文本不符
- **THEN** 工具返回明确错误，不得在任意位置插入锚点

#### Scenario: 回复链归属
- **WHEN** 已有 id=1 批注，调用 `docx_comment`（id=2, parent=1, text="同意"）
- **THEN** `commentsExtended.xml` 含一条 `commentEx`，其 `paraIdParent` 指向 id=1 批注的 paraId

## ADDED Requirements

### Requirement: 批注三件套一致性校验
`ooxml_pack` 的验证阶段 SHALL 检测批注"断链"：`word/comments.xml` 中每个 `w:comment/@w:id` 在 `word/document.xml` 必须有同 id 的 `commentReference`；反之 `document.xml` 每个 `commentReference/@w:id` 必须能在 `comments.xml` 找到对应 `w:comment`。任一方向不满足即产生违规并阻止打包。

#### Scenario: 有批注条目但无锚点
- **WHEN** `comments.xml` 含 `w:id="1"` 的 `<w:comment>`，但 `document.xml` 无 `commentReference w:id="1"`
- **THEN** 验证器报告一致性违规，`ooxml_pack` 失败并指出失配的 id

#### Scenario: 有锚点但无批注条目
- **WHEN** `document.xml` 含 `commentReference w:id="2"`，但 `comments.xml` 无 `w:id="2"` 的 `<w:comment>`
- **THEN** 验证器报告一致性违规，`ooxml_pack` 失败

#### Scenario: 三件套一致时放行
- **WHEN** comments.xml 的 `w:comment/@w:id` 集合与 document.xml 的 `commentReference/@w:id` 集合完全一致
- **THEN** 验证器不就该规则产生任何违规
