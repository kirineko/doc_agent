## Why

当前系统对 Office / PDF / Typst 的生成与读取能力很强，但对 **Markdown 作为交付物**支持薄弱：`.md` 仅能 `fs_write` 落盘或被读为上下文，无法转换为美观、可直接分享的 HTML。用户需要从 Markdown 一键产出**幻灯片（slide）、报告（report）、简历（resume）**三类精巧实用的网页；工具内置 JS / CSS 库走本地 vendor，**离线优先**，正文外链按需保留。

## What Changes

- 新增 `markdown_to_html` 工具：读取项目沙箱内 `.md`，按 `profile`（slide / report / resume）+ `template` 转换为静态 HTML，产物落项目目录。
- 新增 `markdown_list_templates` / `markdown_read_template`：枚举与读取内置模板（含示例 frontmatter），对标 `typst_list_templates` 模式。
- 转换引擎走现有 **boa 嵌入式 JS 运行时**：slide 用 `@marp-team/marp-core`；report / resume 用 `marked` + GFM + `gray-matter` + `highlight.js` 子集（转换路径的 bundle 加载细节见 design D4）。
- 对输入 Markdown 设规模上限，超限拒绝并提示拆分，防止 boa 无 heap 上限下的 OOM。
- 增强能力全做：YAML frontmatter → 封面/元数据、自动 TOC（report 默认）、代码高亮、KaTeX 数学、Mermaid 图、`@media print` 打印样式。
- 内置成熟模板：slide ≥6（Marp 3 官方 + 3 定制）、report ≥6（含 github-markdown-css）、resume ≥5（借鉴 JSON Resume 美学）。
- **离线 vendor 资产（优先本地）**：KaTeX、Mermaid（tiny）、highlight 着色 CSS、主题 CSS 等大体积资源由 **Rust 编译期内置并在转换时拷贝到产物 `assets/` 目录**，工具注入的脚本与样式仅以**相对路径**引用本地文件；Markdown 正文外链不拦截。Mermaid 等按需拷贝（md 无对应语法块则不写入）。
- 新增内置 `markdown` skill（写法、frontmatter schema、profile 选型、与 html-report/pptx 分工），并注入 skill 索引。
- clarify 交付格式新增「Markdown 网页（幻灯片 / 报告 / 简历）」选项。

非目标（本期排除）：Markdown → PDF（含 slide PDF）、Python 运行时、用户上传自定义主题、PPTX 输出。

## Capabilities

### New Capabilities
- `markdown-html-export`: Markdown → 静态 HTML 的工具集、三类 profile（slide/report/resume）、内置模板与离线 vendor 资产（KaTeX / Mermaid / highlight / 主题 CSS 全本地、离线优先）、frontmatter / TOC / 代码高亮 / 数学 / Mermaid / 打印增强、输入规模上限。

### Modified Capabilities
- `document-skills`: 内置 skill 仓库与 system prompt skill 索引新增 `markdown` skill（name + description + 强制 skill_read 指引）。
- `clarify-skill`: 交付格式选项新增「Markdown 网页（slide / report / resume）」，并为该交付类型补充排版/样式澄清覆盖。

## Impact

- **后端（Rust）**：新增 `tools/markdown_html/`（tool spec + 转换 + 模板 list/read）；`tools/registry.rs` 注册三个工具；`tools/runtime/` 新增 markdown / marp-core bundle（独立加载路径，不并入 `bundles_for_code` 现有启发式）。
- **资产**：`assets/js/marked.bundle.js`、`assets/js/marp-core.bundle.js`；`assets/markdown-vendor/`（katex、mermaid.tiny、highlight CSS）；`assets/markdown-templates/{slide,report,resume,samples}/`；`assets/skills/markdown/SKILL.md`。
- **构建**：`scripts/bundle-js-libs.mjs` 增加 marked / marp-core 打包；新增依赖 `marked`、`@marp-team/marp-core`、`gray-matter`、`highlight.js`（devDependencies，仅用于 bundle，须在 design.md 说明）；vendor 静态文件入库。
- **核心**：`core/skills.rs` 注册 markdown skill 文档与索引。
- **体积**：安装包增量约 3–5MB（进 boa eval 的转换引擎约 0.4–0.7MB，其余为产物 vendor 与模板 CSS）。
- **关联 follow-up**：BL-012（boa heap 上限）——大文档转换的 OOM 防护，本期以输入规模上限缓解。
