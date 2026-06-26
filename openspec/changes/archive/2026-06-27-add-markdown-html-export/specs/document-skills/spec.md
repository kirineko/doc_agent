## MODIFIED Requirements

### Requirement: 内置 Skill 仓库

系统 SHALL 内置 docx / pdf / pptx / xlsx / **html-report** / **markdown** 六个 skill 的文档（docx / pdf / pptx / xlsx 含 SKILL.md 及附属文档 editing.md、forms.md、reference.md、pptxgenjs.md 等；html-report 含 SKILL.md；markdown 含 SKILL.md，说明 slide / report / resume 三类 profile 的写法、frontmatter schema、与 html-report / pptx 的选型分工），内容遵循各 skill 既定来源，仅将命令行执行段落改写为本系统工具调用说明。文档 MUST 经编译期内置（不依赖运行时外部路径）。

#### Scenario: 应用启动即可枚举 skill

- **WHEN** 应用启动且未做任何额外安装
- **THEN** 系统能枚举出 docx / pdf / pptx / xlsx / html-report / markdown 六个 skill 的 name 与 description

#### Scenario: 原文知识保留

- **WHEN** 读取 docx skill 的全文
- **THEN** 其中表格双宽度规则、DXA 单位换算、tracked changes XML 模式等知识性内容与原文一致，且不包含 `python scripts/...`、`npm install` 等不可执行的原始命令

#### Scenario: markdown skill 可读取

- **WHEN** Agent 调用 `skill_read {"skill": "markdown"}`
- **THEN** 返回 markdown skill 的 SKILL.md 全文，含 slide / report / resume 选型说明与 frontmatter 示例，且优先推荐本地资源（不含 jsdelivr 等 CDN 引用建议）

### Requirement: Skill 索引注入 system prompt

系统 SHALL 在每轮对话的 system prompt 中注入 skill 索引（每个 skill 的 name + description 摘要），并以强制性措辞指示模型：生成 `.docx` / `.pptx` / `.xlsx` **或静态 HTML 报告交付物**前 MUST 先调用 `skill_read` 获取对应 skill 全文（渐进披露），不得凭记忆直接编写生成代码。索引 MUST 包含 `markdown` skill；当任务为从 Markdown 生成 slide / report / resume 网页时，system prompt MUST 指示优先 `skill_read markdown` 并使用 `markdown_to_html` 工具，不得凭记忆手写完整 HTML。`skill_run` 工具的 description MUST 包含对 Office 交付物的同等强制性提示（HTML 报告以 `fs_write` 为主，不强制经 `skill_run`）。

#### Scenario: 索引可见

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含六个 skill 的名称与触发场景描述，但不包含 skill 全文

#### Scenario: 强制 skill_read 指示可见

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含「生成 Office 或 HTML 报告交付物前 MUST 先 skill_read」的强制性指示
- **AND** `skill_run` 的工具 description 包含对 Office 交付物的同等提示

#### Scenario: markdown 网页交付指示可见

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含「从 Markdown 生成 slide / report / resume 网页时优先 skill_read markdown 并用 markdown_to_html」的指示
