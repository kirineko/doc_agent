## ADDED Requirements

### Requirement: PPT OOXML precise edit slash template

The registry SHALL include a template entry `ppt:edit-ooxml` in category `ppt`, placed immediately after `ppt:edit`.

- `label` MUST describe precise OOXML editing (e.g.「精准修改 PPT」).
- `prompt` MUST instruct the Agent to use `ooxml_unpack` → edit `ppt/slides/slide{N}.xml` → `ooxml_pack`, preserve layout, and MUST NOT use `skill_run` / PptxGenJS for this task.
- `keywords` MUST include terms such as `ooxml`, `xml`, `精准`.

#### Scenario: ppt:edit-ooxml prompt locks OOXML path

- **WHEN** the user selects `ppt:edit-ooxml` and sends after filling placeholders
- **THEN** the sent prompt explicitly requires OOXML unpack/edit/pack
- **AND** explicitly forbids JS script / PptxGenJS for this edit

#### Scenario: ppt:edit-ooxml fuzzy search

- **WHEN** the user searches slash commands with query `精准` or `ooxml` under ppt
- **THEN** `ppt:edit-ooxml` appears in candidates

### Requirement: Compact slash command registration

The registry SHALL include a command entry for manual context compaction in category `command`, alongside `init`.

#### Scenario: Compact command metadata

- **WHEN** the slash menu is loaded
- **THEN** an entry with id `compact`, `kind: "command"`, label describing manual context compaction, and keywords including `compact` and `压缩` SHALL be present

#### Scenario: Compact fills composer without tail

- **WHEN** the user picks `compact` from the slash menu
- **THEN** the composer SHALL be filled with `/compact` only (no trailing space required)

### Requirement: Compact slash command execution

When the user submits composer text that trim-equals `/compact`, the application SHALL invoke `compact_session` IPC with the active session id instead of `send_message`.

The application MUST NOT add a user message containing `/compact` to chat history.

#### Scenario: Compact invokes IPC not send_message

- **WHEN** the user submits `/compact` from the composer
- **THEN** the application SHALL call `compact_session`
- **AND** SHALL NOT call `send_message`
- **AND** SHALL NOT append an optimistic user message for `/compact`

#### Scenario: Compact blocked while clarify pending

- **WHEN** the active session has a pending clarify question
- **THEN** picking `compact` from the slash menu or submitting `/compact` SHALL be blocked with user-visible error (same rationale as `/init`)

#### Scenario: Compact blocked while turn running

- **WHEN** the active session has a turn in `running` or `stopping` state
- **THEN** submitting `/compact` SHALL be blocked with user-visible error

## MODIFIED Requirements

### Requirement: 静态斜杠命令注册表

系统 SHALL 在前端内置静态斜杠命令注册表，每条命令 MUST 包含：`id`、`kind`（`template` | `command`）、`category`（`command` | `general` | `word` | `ppt` | `excel` | `pdf` | `web`）、`label`、`description`、`keywords`、`prompt`（20–100 个字符，含标点与占位符；`kind=command` 时 prompt 可为空或占位，实际行为见各 command 要求）。

`kind=template` 时选中后仅填入 prompt、不自动发送。

`kind=command` 时：`init` 在 Enter 提交 MUST 将 composer 全文作为 user message 调用 `send_message`；`compact` MUST 按 Compact slash command execution 调用 `compact_session` 而非 `send_message`。

`id` 命名规则：**`category` 为 `general` 时 `id` MUST NOT 带 `general:` 前缀**（如 `read`、`clarify`）；其余分类 MUST 使用 `{category}:{action}`（如 `word:create`）。**MUST NOT** 在 `id` 内使用 `/`，以免与触发符 `/` 冲突。

注册表 MUST 包含 **23 条** `kind=template` 命令（含新增 `ppt:edit-ooxml`）及 **2 条** `kind=command`（`init`、`compact`）；**阅读分析**能力 MUST 仅通过 id 为 `read` 的命令提供，不得为 word/ppt/pdf/excel 单独提供 read 类命令。

默认分类展示顺序 MUST 为：**command → general → word → ppt → excel → pdf → web**；同一分类内顺序 MUST 与 registry 源数组一致。

模板占位符 MUST 使用 `{{提示文字}}` 格式；文件路径占位符 MUST NOT 在模板内预置 `@`（用户自行输入 `@` 引用项目文件）。

#### Scenario: 注册表模板条目数量与 read 唯一性

- **WHEN** 应用加载斜杠命令 registry
- **THEN** 共 23 条 `kind=template` 命令
- **AND** 共 2 条 `kind=command` 命令（`init`、`compact`）
- **AND** 唯一 id 为 `read` 的阅读类命令（category 为 general）
- **AND** 不存在 `word:read`、`ppt:read`、`pdf:read`、`excel:read`

#### Scenario: 分类默认顺序

- **WHEN** 用户在空 query 下打开斜杠菜单
- **THEN** 分组标题按 command、general、word、ppt、excel、pdf、web 顺序出现

#### Scenario: prompt 长度约束

- **WHEN** 校验 registry 中每条 `kind=template` 命令的 `prompt`
- **THEN** 字符数（含标点）在 20–100 之间

#### Scenario: 通用类 id 无前缀

- **WHEN** 用户输入 `/read` 或 `/clarify`
- **THEN** 候选匹配 id 为 `read` 或 `clarify` 的命令

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
| `web:report` | skill_read html-report → `fs_write` |
| `web:save-pdf` | `html_to_pdf` |

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
