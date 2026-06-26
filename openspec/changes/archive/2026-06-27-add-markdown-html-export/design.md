## Context

系统已具备成熟的文档生成栈：boa 嵌入式 JS 运行时（`skill_run` + esbuild IIFE bundle）、`html_to_pdf`（WebView）、`typst_to_pdf`（Rust），以及 `html-report` 的离线静态 HTML 规范。Markdown 仅能 `fs_write` 或读为上下文，缺少「Markdown → 美观 HTML」的交付路径。

约束：
- 不引入 Python / 新运行时；复用现有 boa 运行时（`tools/runtime/`）。
- 离线优先：工具内置 JS / CSS 库走本地 vendor；Markdown 正文外链按需保留，不运行时拦截（与 `html-report` skill 的 SHOULD 建议一致，不同于 `image_download` 的入站 URL 校验）。
- 体积尽量小：本期容忍安装包增量 3–5MB；真正进 boa eval 的转换引擎约 0.4–0.7MB。
- boa 现状：`skill_run` 线程 32MB **栈**（解析大 bundle 递归深），**无 heap 上限**（backlog BL-012），exceljs ~874KB 已验证可解析。

## Goals / Non-Goals

**Goals:**
- 新增 `markdown_to_html` / `markdown_list_templates` / `markdown_read_template`，对标 `typst_*` 三件套。
- 三 profile：slide（Marp Core）、report / resume（marked 管线）。
- 增强全做：frontmatter 封面、TOC、代码高亮、KaTeX、Mermaid、`@media print`。
- 成熟模板：slide ≥6、report ≥6、resume ≥5；大体积 vendor 资产 Rust 拷贝到产物 `assets/`，HTML 仅相对路径引用。
- 转换路径与 `skill_run` 的 `bundles_for_code` 解耦，单 profile 单 bundle，不与 office bundle 同次 eval。

**Non-Goals:**
- Markdown → PDF（含 slide PDF）：本期不做（用户已明确不需要）。
- Python 沙箱 / 第三方运行时。
- 用户上传自定义主题 CSS、Markdown → PPTX。
- 应用内 HTML 预览（沿用 html-report：文件浏览区双击系统浏览器打开）。

## Decisions

### D1：转换分两条引擎，按 profile 互斥加载

- slide → `@marp-team/marp-core`（`Marp().render()` 返回 `{ html, css }`，`---` 分页 + 官方主题 + 自动缩放）。
- report / resume → `marked`（+ GFM）解析为 HTML 片段，再由模板 shell 包裹。
- **为何不用前端的 remark 生态**：remark/rehype 是 ESM 管道，boa 适配成本高；marked 易打成 IIFE，已与现有 docx/pptxgenjs bundle 同模式。
- **为何不纯 Rust（pulldown-cmark）**：slide 必须依赖 Marp 才能拿到主题生态，纯 Rust 会分裂两套实现；统一走 boa 转换层更一致。
- 备选：Slidev / reveal.js（重、需 Vite/Vue，违背无构建链约束）——否决。

### D2：vendor 资产 Rust 拷贝，离线优先

- 进 boa eval 的仅转换引擎：marp-core（~250KB）或 marked 管线（marked + GFM + gray-matter + highlight.js 子集，~150–350KB）。
- KaTeX、Mermaid（tiny ~1.1MB）、highlight 着色 CSS、主题 CSS **不进 boa**；编译期 `include_bytes!`/资源内置，转换时由 Rust `fs::write` 拷贝到产物 `assets/`。
- 工具注入的脚本与样式仅以相对路径引用 `./assets/...`；KaTeX / Mermaid 采用**浏览器端渲染**（页脚 `<script src="./assets/...">` + init），转换期不在 boa 内渲染数学/图表，降低 boa 负载。
- **为何浏览器端渲染数学/图表**：避免把 katex(~300KB)/mermaid(~1MB) 塞进 boa eval；内置增强脚本仍走本地 vendor。
- **正文外链**：不运行时校验或拒绝；需要离线可靠时由 skill 引导 `image_download` 或本地路径。
- 备选：转换期预渲染 KaTeX 为静态 HTML（无 JS 依赖但增 boa 负载）——本期不选，留作后续 option。

### D3：highlight 在 boa 内打 class，浏览器用本地 CSS 着色

- marked 管线内用 highlight.js 子集（约 15 常用语言：python/js/ts/rust/bash/sql/json/yaml/html/css/markdown 等）给 `<pre><code>` 加 class；着色由拷出的 `highlight.css` 完成。
- 控制语言子集以压低 bundle 体积与 parse 栈压力。

### D4：profile 互斥单 bundle，独立于 bundles_for_code

- `markdown_to_html` 走独立 `convert()`，不复用 `bundles_for_code` 启发式；单次仅 eval 一个 profile bundle。
- **理由**：避免脚本含 `.pptx`/`exceljs` 等关键字误加载 office bundle（已知成本，见 skill-run-runtime-ops），并保证 32MB 栈/heap 压力可控（marp ~250KB < exceljs ~874KB，不更高）。

### D5：frontmatter 解析位置

- 用 `gray-matter`（打进 marked bundle）在 boa 内解析 frontmatter → `{ data, content }`；slide 将 data 映射为 Marp directive 行注入 Markdown 头部，report/resume 将 data 注入模板 shell（封面/头部）。
- 备选：Rust 侧 YAML 解析后传入——可行但需双向序列化；统一在 boa 内解析更简单，本期选 boa 内解析。

### D6：输出形态

- slide：默认单文件 `deck.html`（Marp inline helper script，体积可控）；含 mermaid/math 时仍可走 `assets/`。
- report / resume：目录 + `assets/`（`index.html` + `assets/theme.css` + 按需 katex/mermaid/highlight 资源）。
- 遵循 html-report 落盘规则：产物写项目根目录内，禁写 `.cache/skill-run/`。

### D7：依赖与构建

- 新增 devDependencies（仅用于 `npm run bundle:js`，不进运行时 npm）：`marked`、`@marp-team/marp-core`、`gray-matter`、`highlight.js`。
- vendor 静态文件（katex、mermaid.tiny、highlight.css、主题 CSS）入库到 `assets/markdown-vendor/` 与 `assets/markdown-templates/`。
- `scripts/bundle-js-libs.mjs` 增加 `markdown`（marked+gfm+gray-matter+hljs 子集）与 `marp-core` 两个 IIFE bundle。
- `release:check` / CI 经 `npm run bundle:js` 产出，与现有四 bundle 一致。

### D8：输入规模上限

- 对输入 Markdown 设字节/页数上限（如 ≤512KB 或 slide ≤200 页），超限返回明确错误并提示拆分，缓解 BL-012（boa 无 heap 上限）下的 OOM 风险。

## Risks / Trade-offs

- [marp-core 在 boa 下渲染失败/不兼容] → 实现期先做最小 render spike（含多页 + 一套定制主题）；失败则降级为 Rust 拼 HTML shell + boa 仅做 Marpit 渲染。
- [大文档/多页 slide 触发 boa OOM] → 输入规模上限（D8）；关联 BL-012 后续做 heap 软上限。
- [Mermaid/KaTeX 在 file:// 下行为受限] → 采用本地 inline init 脚本，断网用例纳入验收。
- [模板过多导致 Agent 选错] → `markdown_list_templates` 带 description；clarify 增 profile 澄清；markdown skill 给决策树。
- [与 html-report 能力重叠] → markdown skill 写清分工：要自由 HTML/复杂交互 → html-report；有 Markdown 源、要 slide/report/resume → markdown_to_html。
- [bundle 体积增长] → 限定 highlight 语言子集；mermaid 用 tiny 且按需拷贝；总进 boa ≤~0.7MB。

## Migration Plan

- 纯新增能力，无破坏性变更；不改动现有 `skill_run`、office、typst、html_to_pdf 行为。
- 分阶段落地（report 基础 → 增强 → slide → resume → mermaid 按需），每阶段产物可用，不阻塞用户。
- 回滚：移除三个工具注册与 markdown skill 索引项即可；vendor/模板资产为静态文件，无运行时副作用。

## Open Questions

- highlight 语言子集的最终清单（覆盖度 vs 体积）——实现期定，记录于 design 附注或 tasks。
- KaTeX 是否需提供「转换期预渲染」开关（无 JS 依赖产物）——本期默认浏览器端渲染，预渲染留作后续 option。
