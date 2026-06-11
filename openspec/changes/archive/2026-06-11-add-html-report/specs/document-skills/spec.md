# document-skills Specification (Delta)

## MODIFIED Requirements

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
