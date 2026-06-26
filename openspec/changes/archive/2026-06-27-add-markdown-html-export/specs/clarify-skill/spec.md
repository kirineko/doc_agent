## ADDED Requirements

### Requirement: clarify skill 交付格式覆盖 Markdown 网页

clarify skill 的 SKILL.md SHALL 在交付格式选项中包含「Markdown 网页（幻灯片 slide / 报告 report / 简历 resume）」，作为与 Word / PPT / Excel / HTML 报告 / Typst PDF 并列的可选交付格式。当用户选择该交付格式或意图为生成 Markdown 网页时，clarify 流程 SHALL 至少澄清 profile（slide / report / resume）与风格/模板倾向（除非用户已明确或 AGENTS.md 已规定）。

#### Scenario: 交付格式包含 Markdown 网页

- **WHEN** Agent 读取 clarify skill 全文并在交付格式不明时发起交付格式选择
- **THEN** 候选交付格式包含「Markdown 网页（slide / report / resume）」

#### Scenario: Markdown 网页澄清 profile 与风格

- **WHEN** 用户选择 Markdown 网页交付且未说明用途与风格
- **THEN** Agent 在澄清过程中至少询问 profile（slide / report / resume）与样式/模板倾向
