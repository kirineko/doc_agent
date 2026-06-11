# document-skills Specification

## Purpose
TBD - created by archiving change add-document-skills-runtime. Update Purpose after archive.
## Requirements
### Requirement: 内置 Skill 仓库

系统 SHALL 内置 docx / pdf / pptx / xlsx / **html-report** 五个 skill 的文档（docx / pdf / pptx / xlsx 含 SKILL.md 及附属文档 editing.md、forms.md、reference.md、pptxgenjs.md 等；html-report 含 SKILL.md），内容遵循各 skill 既定来源，仅将命令行执行段落改写为本系统工具调用说明。文档 MUST 经编译期内置（不依赖运行时外部路径）。

#### Scenario: 应用启动即可枚举 skill

- **WHEN** 应用启动且未做任何额外安装
- **THEN** 系统能枚举出 docx / pdf / pptx / xlsx / html-report 五个 skill 的 name 与 description

#### Scenario: 原文知识保留

- **WHEN** 读取 docx skill 的全文
- **THEN** 其中表格双宽度规则、DXA 单位换算、tracked changes XML 模式等知识性内容与原文一致，且不包含 `python scripts/...`、`npm install` 等不可执行的原始命令

### Requirement: Skill 索引注入 system prompt

系统 SHALL 在每轮对话的 system prompt 中注入 skill 索引（每个 skill 的 name + description 摘要），并以强制性措辞指示模型：生成 `.docx` / `.pptx` / `.xlsx` **或静态 HTML 报告交付物**前 MUST 先调用 `skill_read` 获取对应 skill 全文（渐进披露），不得凭记忆直接编写生成代码。`skill_run` 工具的 description MUST 包含对 Office 交付物的同等强制性提示（HTML 报告以 `fs_write` 为主，不强制经 `skill_run`）。

#### Scenario: 索引可见

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含五个 skill 的名称与触发场景描述，但不包含 skill 全文

#### Scenario: 强制 skill_read 指示可见

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含「生成 Office 或 HTML 报告交付物前 MUST 先 skill_read」的强制性指示
- **AND** `skill_run` 的工具 description 包含对 Office 交付物的同等提示

### Requirement: skill_read 工具

系统 SHALL 提供 `skill_read` 工具，按 skill 名称（及可选的附属文档名）返回内置文档全文。可用 skill 名称 MUST 包含 `html-report`。

#### Scenario: 读取主文档

- **WHEN** Agent 调用 `skill_read {"skill": "docx"}`
- **THEN** 返回 docx skill 的 SKILL.md 全文（已做命令映射改写的版本）

#### Scenario: 读取 html-report

- **WHEN** Agent 调用 `skill_read {"skill": "html-report"}`
- **THEN** 返回 html-report 的 SKILL.md 全文

#### Scenario: 读取附属文档

- **WHEN** Agent 调用 `skill_read {"skill": "pptx", "doc": "pptxgenjs.md"}`
- **THEN** 返回该附属文档全文

#### Scenario: 未知 skill

- **WHEN** Agent 调用 `skill_read {"skill": "unknown"}`
- **THEN** 返回包含可用 skill 列表（含 html-report）的错误信息

### Requirement: docx skill 中文排版指导
docx skill 的 SKILL.md SHALL 包含「中文排版硬规则」与「风格菜单」两个章节。硬规则 MUST 覆盖：eastAsia 字体配置（含可复制的 docx-js 配置片段）、Heading 样式分层强制、中文文档使用 A4 页面、正文首行缩进与行距设置、列表 numbering 强制。风格菜单 SHALL 提供至少四套风格（公文 / 商务报告 / 学术 / 现代简洁）的完整 `styles` 配置片段，并明确指示模型按文档内容选择和调整（颜色、字号、细节），不得每次套用同一风格。原文中 Arial 默认字体、US Letter 默认页面等美式建议 MUST 移除或改写为「西文文档适用」。

#### Scenario: 中文配置片段可直接复制使用
- **WHEN** Agent 通过 `skill_read {"skill":"docx"}` 读取全文并复制「中文排版硬规则」中的默认字体片段用于 `skill_run`
- **THEN** 生成的 `.docx` 中文以指定 eastAsia 字体（如微软雅黑）渲染，无字体回退

#### Scenario: 风格菜单鼓励变化
- **WHEN** 读取 docx skill 全文
- **THEN** 风格菜单章节包含至少四套风格的完整样式片段，且包含「按内容调整、避免千篇一律」的明确指示

### Requirement: pptx 与 xlsx skill 中文字体指引
pptx skill 的 SKILL.md SHALL 包含中文演示文稿字体指引（pptxgenjs `fontFace` 使用微软雅黑等中文字体）；xlsx skill 的 SKILL.md SHALL 包含中文表格字体与列宽估算指引（中文字符约占 2 个西文字符宽度）。

#### Scenario: pptx 中文指引可见
- **WHEN** Agent 读取 pptx skill 的 SKILL.md
- **THEN** 文档包含中文字体的 `fontFace` 配置说明

### Requirement: clarify skill 注册与枚举

系统现有的 `document-skills` Requirement「内置 Skill 仓库」SHALL 扩展：skill 仓库新增 `clarify` skill（名称 `clarify`，描述「文档创作前的需求澄清流程，帮助明确内容、结构与排版风格」）。`skill_read` 工具的 `skill` 参数枚举说明 MUST 包含 `clarify`。

#### Scenario: 应用启动可枚举 clarify skill

- **WHEN** 应用启动且未做任何额外安装
- **THEN** 系统能枚举出 `clarify` skill 的 name 与 description，与 docx / pptx 等并列

#### Scenario: skill_read 工具描述包含 clarify

- **WHEN** Agent 读取 `skill_read` 工具的描述文本
- **THEN** 描述中的可用 skill 枚举包含 `clarify`

---

### Requirement: 系统提示词包含 clarify 触发指示

系统 SHALL 在每轮对话的 system prompt 中注入 clarify 触发指示：收到全新文档创作请求且需求信息不完整（缺少主题/受众/结构/风格中 ≥ 2 项）时，MUST 先 `skill_read clarify` 并按流程执行澄清，不得直接进入 `skill_run`。

#### Scenario: 触发指示可见

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含「模糊文档创作请求前 MUST skill_read clarify」的指示，与已有的 Office deliverable `skill_read` 指示共存

#### Scenario: 触发指示不替换原有指示

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 同时包含「生成 Office 交付物前 MUST skill_read 对应格式 skill」和「模糊创作请求前 MUST skill_read clarify」两条指示

