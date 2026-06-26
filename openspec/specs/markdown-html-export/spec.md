# markdown-html-export Specification

## Purpose

将项目沙箱内 Markdown 转为可直接分享的静态 HTML，支持 slide / report / resume 三类 profile；内置模板与 KaTeX / Mermaid / 代码高亮等 vendor 资源，离线优先。

## Requirements

### Requirement: markdown_to_html 工具

系统 SHALL 提供 `markdown_to_html` 工具，将项目沙箱内的 `.md` 文件转换为静态 HTML，产物写入项目目录。工具 MUST 接受 `path`（项目相对 `.md`）、`out_path`（`.html` 文件或目录；目录时写 `index.html` 并在同目录 `assets/` 下放置资源）、`profile`（`slide` | `report` | `resume`）。MUST 接受可选 `template`（模板 id，缺省按 profile 取默认模板）与可选 `options`（`toc` / `math` / `highlight` / `mermaid` / `lang`，缺省全开、`lang` 默认 `zh-CN`）。转换 MUST 在嵌入式 boa 运行时内完成，产物 HTML MUST 可被系统浏览器离线打开。

#### Scenario: 转换报告 Markdown

- **WHEN** Agent 对 `docs/q1.md` 调用 `markdown_to_html`，`profile` 为 `report`，`out_path` 为 `docs/q1/index.html`
- **THEN** 生成 `docs/q1/index.html` 及 `docs/q1/assets/`，HTML 含 Markdown 渲染内容，返回**解析后的 HTML 文件路径**（`docs/q1/index.html`）与所用 template

#### Scenario: 目录型 out_path 返回 index.html

- **WHEN** Agent 传入目录型 `out_path`（如 `docs/q1` 或 `docs/q1/`）
- **THEN** 写入 `docs/q1/index.html`，且工具响应的 `path` MUST 为 `docs/q1/index.html`（而非原始目录路径）

#### Scenario: profile 必填且校验

- **WHEN** Agent 调用 `markdown_to_html` 未传 `profile` 或传入非 `slide`/`report`/`resume` 的值
- **THEN** 工具返回 invalid arguments 错误，不写入任何文件

#### Scenario: 未知 template

- **WHEN** Agent 传入的 `template` 不属于该 `profile` 的内置模板
- **THEN** 工具返回错误并列出该 profile 可用模板 id，不写入文件

#### Scenario: 越界路径被拒

- **WHEN** `path` 或 `out_path` 解析后逃出项目根目录
- **THEN** 经 Sandbox 校验拒绝，返回错误，不写入文件

### Requirement: 三类 profile 与内置模板

系统 SHALL 内置三类 profile 的成熟模板：`slide` ≥ 6 套（含 Marp 官方 default/gaia/uncover 与定制扩展）、`report` ≥ 6 套（含基于 github-markdown-css 的样式）、`resume` ≥ 5 套。所有模板 MUST 编译期内置，MUST NOT 依赖运行时外部路径或网络。slide 转换 MUST 使用 `@marp-team/marp-core`，支持 `---` 分页与主题；report / resume 转换 MUST 使用 `marked`（含 GFM 表格、代码块、任务列表）。

#### Scenario: slide 多页分页

- **WHEN** Agent 对含多个 `---` 分隔的 `.md` 调用 `markdown_to_html` 且 `profile` 为 `slide`
- **THEN** 产物为分页幻灯片 HTML，每个 `---` 段对应一页，可键盘翻页

#### Scenario: report GFM 表格渲染

- **WHEN** report Markdown 含 GFM 表格
- **THEN** 产物 HTML 正确渲染为 `<table>`，并应用所选模板样式

#### Scenario: 默认模板

- **WHEN** Agent 不传 `template`
- **THEN** 按 `profile` 使用该类默认模板完成转换

### Requirement: frontmatter 与内容增强

系统 SHALL 解析 `.md` 头部的 YAML frontmatter，并据此生成元数据与封面/头部信息（report / resume 的 `title`/`author`/`date`/`name` 等；slide 映射 Marp directives 如 `theme`/`paginate`）。系统 SHALL 支持以下增强（默认开启，可经 `options` 关闭）：report 自动目录 TOC（扫描 h2/h3，页内锚点）、代码块语法高亮、KaTeX 数学（`$...$` / `$$...$$`）、Mermaid 图（` ```mermaid ` 代码块）、`@media print` 打印样式。

#### Scenario: frontmatter 驱动封面

- **WHEN** report Markdown 含 `title` 与 `author` frontmatter
- **THEN** 产物 HTML 头部含对应封面/标题区，正文不重复渲染 frontmatter 原文

#### Scenario: report 默认生成 TOC

- **WHEN** report Markdown 含多个 h2/h3 标题且未关闭 `toc`
- **THEN** 产物 HTML 含目录，目录项锚点指向对应标题

#### Scenario: 代码高亮

- **WHEN** Markdown 含带语言标注的代码块（如 ```python）
- **THEN** 产物 HTML 中该代码块带语法高亮标记，并引用本地高亮样式

#### Scenario: 关闭增强项

- **WHEN** Agent 传入 `options.mermaid=false` 且 Markdown 含 mermaid 代码块
- **THEN** 该代码块按普通代码块处理，产物不写入 mermaid 资源

### Requirement: 离线 vendor 资产（优先本地）

系统 SHALL 将转换所需的全部 JS / CSS 库（KaTeX、Mermaid、代码高亮样式、主题 CSS 等）编译期内置，并在转换时按需拷贝到产物 `assets/` 目录；工具生成的脚本与样式 MUST 仅以**相对路径**引用这些本地资源。Markdown 正文中的外链（如 `[文字](https://…)`、`![](https://…)`、原始 HTML 资源标签）MAY 保留，系统 MUST NOT 因含外网 URL 而拒绝转换；skill 与工具描述 SHOULD 建议优先使用本地资源以保证离线打开。当 Markdown 未使用某增强语法时（如无 mermaid 代码块），对应大体积资源 MUST NOT 写入产物目录。

#### Scenario: 工具内置资源走本地路径

- **WHEN** 任意 profile 转换完成且启用了 math / mermaid / highlight / theme 等增强
- **THEN** 工具注入的 `<script>` / `<link>` 与 `assets/` 内 CSS 均以相对路径引用本地文件，不依赖 CDN

#### Scenario: 断网可正常渲染

- **WHEN** 在无网络环境用浏览器打开产物 HTML（含数学/图表/高亮）
- **THEN** 数学公式、Mermaid 图、代码高亮、主题样式均正常显示

#### Scenario: Mermaid 按需拷贝

- **WHEN** Markdown 不含 mermaid 代码块
- **THEN** 产物 `assets/` 不包含 mermaid 运行时文件

### Requirement: markdown_list_templates 与 markdown_read_template

系统 SHALL 提供 `markdown_list_templates` 与 `markdown_read_template`。`markdown_list_templates` MUST 返回内置模板列表，每项含 `id`、`profile`、`title`、`description`。`markdown_read_template` MUST 按模板 id 返回该模板的示例 Markdown（含示例 frontmatter），供 Agent 复制改写。

#### Scenario: 列出模板

- **WHEN** Agent 调用 `markdown_list_templates`
- **THEN** 返回 slide / report / resume 各自的模板项，含 id 与 description

#### Scenario: 读取模板示例

- **WHEN** Agent 调用 `markdown_read_template` 并传入合法模板 id
- **THEN** 返回该模板的示例 Markdown 文本（含 frontmatter）

#### Scenario: 读取未知模板

- **WHEN** Agent 传入不存在的模板 id
- **THEN** 返回错误并列出可用模板 id

### Requirement: 输入规模上限

系统 SHALL 对输入 Markdown 设定规模上限（字节或页数），超限时返回明确错误并提示拆分，不写入产物。该上限用于在 boa 运行时无 heap 上限的前提下防止大文档转换拖垮进程。

#### Scenario: 超大输入被拒

- **WHEN** 输入 Markdown 超过规模上限
- **THEN** 工具返回超限错误并提示拆分，不写入产物
