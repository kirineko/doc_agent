## ADDED Requirements

### Requirement: Markdown 斜杠命令分组

注册表 SHALL 新增分类 `markdown`，并包含以下 4 条 `kind=template` 命令，顺序 MUST 为 slide → report → resume → convert：

| id | label | 主要引导 |
|----|-------|---------|
| `markdown:slide` | Markdown 幻灯片 | profile=slide，选模板写 .md 再转 HTML |
| `markdown:report` | Markdown 报告 | profile=report，选模板写 .md 再转 HTML |
| `markdown:resume` | Markdown 简历 | profile=resume，frontmatter 再转 HTML |
| `markdown:convert` | 转 HTML | 已有 `.md` 按 profile 转换 |

每条命令的 `keywords` MUST 含 `markdown` 及对应 profile 相关词（如 slide、报告、简历、转换）。

#### Scenario: markdown 分类 Tab 可见

- **WHEN** 用户打开斜杠菜单并切换到 Markdown 分类
- **THEN** 展示上述 4 条命令，label 与 description 一行可见

#### Scenario: markdown:slide 引导模板化幻灯片

- **WHEN** 用户选择 `markdown:slide` 并发送（替换 `{{主题}}` 后）
- **THEN** 填入的 prompt 提及 slide 模板与「转 HTML」（MUST NOT 使用工具名 `markdown_to_html`）
- **AND** Agent 路径为 skill_read markdown → markdown_list_templates → markdown_read_template → fs_write → markdown_to_html

#### Scenario: markdown:convert 引导已有文件转换

- **WHEN** 用户选择 `markdown:convert` 并将占位符换为项目内 `.md` 与 profile
- **THEN** Agent 优先对指定 `.md` 调用 `markdown_to_html`，而非新建 html-report

### Requirement: 下载图片斜杠命令

注册表 SHALL 在 category `general` 新增 `kind=template` 命令 `download-images`：

- `label` MUST 为「下载图片」或等价表述
- `description` MUST 说明按主题搜索并下载图片到项目
- `prompt` MUST 使用 `{{主题}}` 占位符，**MUST NOT** 要求用户填写 URL 列表
- `prompt` MUST 引导 Agent 将图片落地到 `images/`（或等价默认目录）并返回本地路径

#### Scenario: download-images 占位符为主题

- **WHEN** 用户选择 `download-images`
- **THEN** 填入的 prompt 含 `{{主题}}` 占位符
- **AND** prompt 不含「URL 列表」类占位符

#### Scenario: download-images Agent 路径

- **WHEN** 用户选择 `download-images`、填入主题并发送
- **THEN** Agent SHOULD 通过 `web_search` 等获取图片 URL 后调用 `image_download`
- **AND** 返回本地相对路径供后续文档引用

## MODIFIED Requirements

### Requirement: 静态斜杠命令注册表

系统 SHALL 在前端内置静态斜杠命令注册表，每条命令 MUST 包含：`id`、`kind`（`template` | `command`）、`category`（`command` | `general` | `markdown` | `word` | `ppt` | `excel` | `pdf` | `web`）、`label`、`description`、`keywords`、`prompt`（20–100 个字符，含标点与占位符；`kind=command` 时 prompt 可为空或占位，实际行为见各 command 要求）。

`kind=template` 时选中后仅填入 prompt、不自动发送。

`kind=command` 时：`init` 在 Enter 提交 MUST 将 composer 全文作为 user message 调用 `send_message`；`compact` MUST 按 Compact slash command execution 调用 `compact_session` 而非 `send_message`。

`id` 命名规则：**`category` 为 `general` 时 `id` MUST NOT 带 `general:` 前缀**（如 `read`、`clarify`、`download-images`）；其余分类 MUST 使用 `{category}:{action}`（如 `word:create`、`markdown:slide`）。**MUST NOT** 在 `id` 内使用 `/`，以免与触发符 `/` 冲突。

注册表 MUST 包含 **28 条** `kind=template` 命令（含新增 4 条 markdown 与 1 条 `download-images`）及 **2 条** `kind=command`（`init`、`compact`）；**阅读分析**能力 MUST 仅通过 id 为 `read` 的命令提供，不得为 word/ppt/pdf/excel 单独提供 read 类命令。

默认分类展示顺序 MUST 为：**command → general → markdown → word → ppt → excel → pdf → web**；同一分类内顺序 MUST 与 registry 源数组一致。

模板占位符 MUST 使用 `{{提示文字}}` 格式；文件路径占位符 MUST NOT 在模板内预置 `@`（用户自行输入 `@` 引用项目文件）。

#### Scenario: 注册表模板条目数量与 read 唯一性

- **WHEN** 应用加载斜杠命令 registry
- **THEN** 共 28 条 `kind=template` 命令
- **AND** 共 2 条 `kind=command` 命令（`init`、`compact`）
- **AND** 唯一 id 为 `read` 的阅读类命令（category 为 general）
- **AND** 不存在 `word:read`、`ppt:read`、`pdf:read`、`excel:read`

#### Scenario: 分类默认顺序

- **WHEN** 用户在空 query 下打开斜杠菜单
- **THEN** 分组标题按 command、general、markdown、word、ppt、excel、pdf、web 顺序出现

#### Scenario: prompt 长度约束

- **WHEN** 校验 registry 中每条 `kind=template` 命令的 `prompt`
- **THEN** 字符数（含标点）在 20–100 之间

#### Scenario: 通用类 id 无前缀

- **WHEN** 用户输入 `/read` 或 `/download-images`
- **THEN** 候选匹配 id 为 `read` 或 `download-images` 的命令

#### Scenario: read 覆盖 Excel 阅读分析

- **WHEN** 用户选择 `read` 并在 prompt 中将占位符换为项目内 `@报表.xlsx` 后发送
- **THEN** Agent 可按 xlsx 选择 `excel_read` 或 `office_read_to_markdown` 等工具
- **AND** 不存在独立的 `excel:read` 斜杠命令

### Requirement: 命令与 Agent 能力映射

每条斜杠命令的 `prompt` MUST 引导 Agent 使用系统已有工具链，且 MUST NOT 指示跳过 `skill_read` 或 `clarify` 等 system prompt 强制流程。

| 命令 id | 主要 Agent 路径 |
|---------|----------------|
| `read` | `office_read_to_markdown` / `pdf_read` / `excel_read` 等（按文件类型） |
| `clarify` | clarify skill + `clarify_ask` |
| `search` | `fs_search` |
| `convert` | `office_convert` |
| `web-search` | `web_search` / `web_extract` |
| `download-images` | `web_search`（找 URL）→ `image_download` |
| `markdown:slide` | skill_read markdown → list/read template → fs_write → markdown_to_html（profile=slide） |
| `markdown:report` | skill_read markdown → list/read template → fs_write → markdown_to_html（profile=report） |
| `markdown:resume` | skill_read markdown → list/read template → fs_write → markdown_to_html（profile=resume） |
| `markdown:convert` | markdown_to_html（已有 .md） |
| `word:create` | skill_read docx → skill_run |
| `word:edit` | ooxml_unpack → XML 编辑 → ooxml_pack |
| `word:comment` | ooxml_unpack → `docx_comment` → document.xml 锚点 → pack |
| `word:clean-revisions` | `docx_accept_changes` |
| `word:extract-table` | `docx_extract_table` |
| `ppt:create` | skill_read pptx → skill_run |
| `ppt:edit` | skill_read pptx/pptxgenjs.md → skill_run |
| `ppt:edit-ooxml` | ooxml_unpack → slide XML 编辑 → ooxml_pack |
| `excel:create` | skill_read xlsx → skill_run / excel_write |
| `excel:clean` | `excel_describe` → `excel_normalize` |
| `excel:check-formula` | `xlsx_recalc`（报告错误，不自动修公式） |
| `excel:analyze` | `excel_normalize` → `data_query` |
| `pdf:create` | typst guide → list → 场景模板 → 写 `.typ` → `typst_to_pdf` |
| `pdf:edit-typst` | `fs_read`/`fs_patch` 改 `.typ` → `typst_to_pdf` |
| `pdf:ops` | `pdf_merge` / `pdf_split` / `pdf_rotate` / `pdf_delete_pages` |
| `pdf:forms` | skill_read pdf → skill_run |
| `web:report` | skill_read html-report → `fs_write`（自由 HTML，非 Markdown 模板） |
| `web:save-pdf` | `html_to_pdf` |

`web:report` 的 `description` MUST 明确为自由 HTML/CSS 静态报告，且 MUST NOT 暗示 Markdown 模板路径。

#### Scenario: web:report 与 markdown:report 分工

- **WHEN** 用户查看 `web:report` 与 `markdown:report` 的 description
- **THEN** `web:report` 强调自由 HTML/CSS、非 Markdown 模板
- **AND** `markdown:report` 强调 Markdown 模板化 report 网页

#### Scenario: ppt:edit 引导脚本路径

- **WHEN** 用户选择 `ppt:edit` 并发送
- **THEN** 填入的 prompt 明确要求通过 pptxgenjs / skill_run 脚本修改
- **AND** 不将 OOXML 作为该命令的首选路径

#### Scenario: ppt:edit-ooxml 引导 OOXML 路径

- **WHEN** 用户选择 `ppt:edit-ooxml` 并将占位符换为项目内 `.pptx` 后发送
- **THEN** Agent 优先 `ooxml_unpack` → 编辑 slide XML → `ooxml_pack`
- **AND** 不使用 PptxGenJS 脚本改写该文件

#### Scenario: pdf:create 基于 Typst 新建

- **WHEN** 用户选择 `pdf:create`
- **THEN** 填入的 prompt 明确提及「基于 Typst」与「写 .typ 再编译」

#### Scenario: pdf:edit-typst 修订已有 Typst 源稿

- **WHEN** 用户选择 `pdf:edit-typst` 并将占位符换为项目内 `.typ` 路径后发送
- **THEN** Agent 优先修改该 `.typ` 并调用 `typst_to_pdf` 重新输出 PDF

#### Scenario: excel:check-formula 不承诺自动修复

- **WHEN** 用户选择 `excel:check-formula`
- **THEN** prompt 仅要求检查并列出公式错误位置
