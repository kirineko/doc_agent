# document-skills Specification

## Purpose
TBD - created by archiving change add-document-skills-runtime. Update Purpose after archive.
## Requirements
### Requirement: 内置 Skill 仓库
系统 SHALL 内置 docx / pdf / pptx / xlsx 四个 skill 的文档（含 SKILL.md 及附属文档 editing.md、forms.md、reference.md、pptxgenjs.md），内容遵循 `doc_skills/` 原文，仅将命令行执行段落改写为本系统工具调用说明。文档 MUST 经编译期内置（不依赖运行时外部路径）。

#### Scenario: 应用启动即可枚举 skill
- **WHEN** 应用启动且未做任何额外安装
- **THEN** 系统能枚举出 docx / pdf / pptx / xlsx 四个 skill 的 name 与 description

#### Scenario: 原文知识保留
- **WHEN** 读取 docx skill 的全文
- **THEN** 其中表格双宽度规则、DXA 单位换算、tracked changes XML 模式等知识性内容与原文一致，且不包含 `python scripts/...`、`npm install` 等不可执行的原始命令

### Requirement: Skill 索引注入 system prompt
系统 SHALL 在每轮对话的 system prompt 中注入 skill 索引（每个 skill 的 name + description 摘要），并指示模型在处理对应文档任务前先调用 `skill_read` 获取全文（渐进披露）。

#### Scenario: 索引可见
- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含四个 skill 的名称与触发场景描述，但不包含 skill 全文

### Requirement: skill_read 工具
系统 SHALL 提供 `skill_read` 工具，按 skill 名称（及可选的附属文档名）返回内置文档全文。

#### Scenario: 读取主文档
- **WHEN** Agent 调用 `skill_read {"skill": "docx"}`
- **THEN** 返回 docx skill 的 SKILL.md 全文（已做命令映射改写的版本）

#### Scenario: 读取附属文档
- **WHEN** Agent 调用 `skill_read {"skill": "pptx", "doc": "pptxgenjs.md"}`
- **THEN** 返回该附属文档全文

#### Scenario: 未知 skill
- **WHEN** Agent 调用 `skill_read {"skill": "unknown"}`
- **THEN** 返回包含可用 skill 列表的错误信息

