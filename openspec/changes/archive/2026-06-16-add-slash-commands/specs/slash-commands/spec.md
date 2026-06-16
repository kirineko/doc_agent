## ADDED Requirements

### Requirement: 静态斜杠命令注册表

系统 SHALL 在前端内置静态斜杠命令注册表，每条命令 MUST 包含：`id`、`category`（`general` | `word` | `ppt` | `excel` | `pdf` | `web`）、`label`、`description`、`keywords`、`prompt`（20–100 个字符，含标点与占位符）。

`id` 命名规则：**`category` 为 `general` 时 `id` MUST NOT 带 `general:` 前缀**（如 `read`、`clarify`）；其余分类 MUST 使用 `{category}:{action}`（如 `word:create`）。**MUST NOT** 在 `id` 内使用 `/`，以免与触发符 `/` 冲突。

注册表 MUST 包含且仅包含 design.md 定义的 **22 条**命令；**阅读分析**能力 MUST 仅通过 id 为 `read` 的命令提供，不得为 word:ppt/pdf/excel 单独提供 read 类命令。

默认分类展示顺序 MUST 为：**general → word → ppt → excel → pdf → web**；同一分类内顺序 MUST 与 registry 源数组一致。

#### Scenario: 注册表条目数量与 read 唯一性

- **WHEN** 应用加载斜杠命令 registry
- **THEN** 共 22 条命令
- **AND** 唯一 id 为 `read` 的阅读类命令（category 为 general）
- **AND** 不存在 `word:read`、`ppt:read`、`pdf:read`、`excel:read`、`general/read`

#### Scenario: 分类默认顺序

- **WHEN** 用户在空 query 下打开斜杠菜单
- **THEN** 分组标题按 general、word、ppt、excel、pdf、web 顺序出现

#### Scenario: prompt 长度约束

- **WHEN** 校验 registry 中每条命令的 `prompt`
- **THEN** 字符数（含标点）在 20–100 之间

#### Scenario: 通用类 id 无前缀

- **WHEN** 用户输入 `/read` 或 `/clarify`
- **THEN** 候选匹配 id 为 `read` 或 `clarify` 的命令
- **AND** 不存在 id 为 `general/read` 或 `general/clarify` 的条目

#### Scenario: read 覆盖 Excel 阅读分析

- **WHEN** 用户选择 `read` 并在 prompt 中将 `@文件名` 换为 `@报表.xlsx` 后发送
- **THEN** Agent 可按 xlsx 选择 `excel_read` 或 `office_read_to_markdown` 等工具
- **AND** 不存在独立的 `excel:read` 斜杠命令

### Requirement: 斜杠命令模糊搜索

系统 SHALL 对斜杠命令支持 fzf 式子序列模糊匹配，搜索字段 MUST 包含 `id`、`label`、`description`、`keywords` 与 `category` 显示名。

匹配结果 MUST 保持分类分组；分类顺序不变，组内按匹配得分降序。无匹配时 MUST 展示空状态提示。

#### Scenario: 按类别过滤

- **WHEN** 用户输入 `/word`
- **THEN** 候选优先展示 category 为 word 的命令（如新建 Word、精准修改 Word）

#### Scenario: 关键词跨类命中

- **WHEN** 用户输入 `/批注`
- **THEN** 候选包含 `word:comment`（添加批注）

#### Scenario: 无匹配

- **WHEN** 用户输入 `/zzzznotfound`
- **THEN** 弹层展示无匹配提示，输入框内容保持不变

### Requirement: 命令与 Agent 能力映射

每条斜杠命令的 `prompt` MUST 引导 Agent 使用系统已有工具链，且 MUST NOT 指示跳过 `skill_read` 或 `clarify` 等 system prompt 强制流程。

| 命令 id | 主要 Agent 路径 |
|---------|----------------|
| `read` | `office_read_to_markdown` / `pdf_read` / `excel_read` 等（按文件类型） |
| `clarify` | clarify skill + `clarify_ask` |
| `search` | `fs_search` |
| `convert` | `office_convert` |
| `web-search` | `web_search` / `web_extract` |
| `word:create` | skill_read docx → skill_run |
| `word:edit` | ooxml_unpack → XML 编辑 → ooxml_pack |
| `word:comment` | ooxml_unpack → `docx_comment` → document.xml 锚点 → pack |
| `word:clean-revisions` | `docx_accept_changes` |
| `word:extract-table` | `docx_extract_table` |
| `ppt:create` | skill_read pptx → skill_run |
| `ppt:edit` | ooxml 或 skill 路径 |
| `excel:create` | skill_read xlsx → skill_run / excel_write |
| `excel:clean` | `excel_describe` → `excel_normalize` |
| `excel:check-formula` | `xlsx_recalc`（报告错误，不自动修公式） |
| `excel:analyze` | `excel_normalize` → `data_query` |
| `pdf:create` | typst guide → list → 场景模板 → 写 `.typ` → `typst_to_pdf` |
| `pdf:edit-typst` | `fs_read`/`fs_patch` 改 `.typ` → `typst_to_pdf`（重编译；大改可再走模板链） |
| `pdf:ops` | `pdf_merge` / `pdf_split` / `pdf_rotate` / `pdf_delete_pages` |
| `pdf:forms` | skill_read pdf → skill_run |
| `web:report` | skill_read html-report → `fs_write` |
| `web:save-pdf` | `html_to_pdf` |

#### Scenario: pdf:create 基于 Typst 新建

- **WHEN** 用户选择 `pdf:create`
- **THEN** 填入的 prompt 明确提及「基于 Typst」与「写 .typ 再编译」
- **AND** label 或 description MUST 含 Typst 字样

#### Scenario: pdf:edit-typst 修订已有 Typst 源稿

- **WHEN** 用户选择 `pdf:edit-typst` 并将 `@文件名.typ` 换为项目内已有 `.typ` 后发送
- **THEN** Agent 优先修改该 `.typ` 并调用 `typst_to_pdf` 重新输出 PDF
- **AND** MUST NOT 将需求默认路由为 `html_to_pdf` 或直接改二进制 PDF

#### Scenario: excel:check-formula 不承诺自动修复

- **WHEN** 用户选择 `excel:check-formula`
- **THEN** prompt 仅要求检查并列出公式错误位置
