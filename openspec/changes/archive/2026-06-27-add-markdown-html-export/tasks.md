## 1. 依赖与离线 vendor 资产

- [x] 1.1 `package.json` 增加 devDependencies：`marked`、`@marp-team/marp-core`、`gray-matter`、`highlight.js`（仅用于 bundle）
- [x] 1.2 `scripts/bundle-js-libs.mjs` 新增两个 IIFE bundle：`markdown`（marked + GFM + gray-matter + highlight.js 子集，global 暴露）与 `marp-core`（global `Marpit`）
- [x] 1.3 确定 highlight.js 语言子集清单（python/js/ts/rust/bash/sql/json/yaml/html/css/markdown 等），并在 bundle 脚本中按子集打包
- [x] 1.4 内置 vendor 静态资源到 `src-tauri/assets/markdown-vendor/`：`katex.min.js` + `katex.min.css`、`mermaid.tiny.min.js`、`highlight.css`（均来自本地，无 CDN）
- [x] 1.5 运行 `npm run bundle:js` 产出 `assets/js/markdown.bundle.js`、`assets/js/marp-core.bundle.js`，确认体积进 boa 部分 ≤ ~0.7MB

## 2. 内置模板

- [x] 2.1 slide 模板 ≥6：Marp 官方 default/gaia/uncover + 定制 corporate/academic/minimal-dark（`@theme` + `@import`），放 `assets/markdown-templates/slide/`
- [x] 2.2 report 模板 ≥6：含 github-markdown-css light/dark + academic/business/narrow/data，放 `assets/markdown-templates/report/`
- [x] 2.3 resume 模板 ≥5：classic/even/two-col/compact/modern，放 `assets/markdown-templates/resume/`
- [x] 2.4 公共 CSS：封面/TOC/`@media print` 共享样式，中文系统字体栈（PingFang SC / Microsoft YaHei）
- [x] 2.5 各 profile 示例 Markdown（含 frontmatter）放 `assets/markdown-templates/samples/`

## 3. 转换引擎（boa）

- [x] 3.1 `src-tauri/src/tools/markdown_html/convert.rs`：独立 boa 转换路径，单 profile 仅加载一个 bundle，**不复用** `bundles_for_code`，不与 office bundle 同次 eval
- [x] 3.2 frontmatter 解析（gray-matter）：slide 映射 Marp directives；report/resume 注入模板 shell 封面/头部
- [x] 3.3 slide：调用 `Marp().render()` 得 `{html, css}`，组装单文件 `deck.html`（inline helper，离线）
- [x] 3.4 report/resume：marked(GFM) → 片段 → 模板 shell（含 TOC 扫描 h2/h3、highlight class 标注）
- [x] 3.5 增强：KaTeX / Mermaid 改为浏览器端渲染（页脚本地脚本 init）；`@media print` 注入
- [x] 3.6 输入规模上限校验（字节/页数），超限返回明确错误，不写产物

## 4. vendor 拷贝与产物落盘

- [x] 4.1 转换时按需将 vendor 资源拷贝到产物 `assets/`（report/resume 目录形态；slide 单文件按需）
- [x] 4.2 Mermaid/KaTeX 按需拷贝：Markdown 无对应语法块时不写入对应资源
- [x] 4.3 工具注入的 JS/CSS 以相对路径引用 `./assets/...`；不运行时拦截 Markdown 正文外链
- [x] 4.4 遵循 html-report 落盘规则：写项目根目录内，禁写 `.cache/skill-run/`；经 Sandbox 校验路径

## 5. 工具注册与 schema

- [x] 5.1 `markdown_html/mod.rs`：`markdown_to_html` ToolSpec（path/out_path/profile/template?/options?）+ handler
- [x] 5.2 `markdown_html/templates.rs`：`markdown_list_templates`（id/profile/title/description）与 `markdown_read_template`（返回示例 md）
- [x] 5.3 `tools/registry.rs` 注册三个工具
- [x] 5.4 参数校验：profile 必填且枚举校验；未知 template 报错并列可用 id；越界路径拒绝

## 6. Skill 与 system prompt

- [x] 6.1 新增 `assets/skills/markdown/SKILL.md`：slide/report/resume 写法、frontmatter schema、profile 选型、与 html-report/pptx 分工、离线优先说明、`markdown_to_html` 工作流
- [x] 6.2 `core/skills.rs` 注册 markdown skill 文档与索引（name + description）
- [x] 6.3 system prompt skill 索引含 markdown；新增「Markdown 网页交付优先 skill_read markdown + markdown_to_html」强制指示
- [x] 6.4 clarify skill（`assets/skills/clarify/SKILL.md`）交付格式新增「Markdown 网页（slide/report/resume）」，并补 profile 与样式澄清

## 7. 测试

- [x] 7.1 转换单测：report GFM 表格 → `<table>`；slide 多 `---` → 多页；默认模板生效
- [x] 7.2 frontmatter → 封面/TOC 单测；关闭 `options.mermaid` 不写 mermaid 资源
- [x] 7.3 内置资源断言：工具注入的 script/link 与 `assets/` CSS 走相对路径；正文外链允许保留
- [x] 7.4 工具 handler：profile 缺失/非法、未知 template、越界路径均报错且不写文件
- [x] 7.5 `markdown_list_templates` / `markdown_read_template` 契约测试（含未知 id 错误列出可用项）
- [x] 7.6 输入规模上限测试：超限返回错误且不写产物
- [x] 7.7 skill 测试：`skill_read markdown` 返回全文且不含 jsdelivr 等 CDN 引用建议；索引枚举含 markdown

## 8. 文档与收尾

- [x] 8.1 `README.md` 文档与工具能力表新增 Markdown 网页（slide/report/resume）一行
- [x] 8.2 `CHANGELOG.md` 追加 `[Unreleased]` 条目
- [x] 8.3 本地自检：`npm run bundle:js`、`cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test`、`npm run typecheck`、`npm test`、`npm run build`

## 9. Review 优化（post-apply）

- [x] 9.1 resume 封面：`name`/`title`/联系方式字段映射；sample 去掉重复 `# 姓名`
- [x] 9.2 resume 五套 CSS 对齐真实 HTML（`.resume-main` / `.resume-header .role|.contact`）；双栏用 `column-count`
- [x] 9.3 `base.css` 通用排版（行内 code、链接、hr、图注、`.table-wrap`）；`slide/default.css` 增强
- [x] 9.4 JS：`wrapTables()` 宽表横向滚动；`injectFigureCaptionClasses`；Mermaid `language-mermaid` + `mermaid.run`
- [x] 9.5 外链策略：正文 `<a href>` 与外链图片允许；工具内置资源优先本地，不运行时拦截
- [x] 9.11 review 跟进：移除 `contains_external_resource_url` 校验；`path` 返回解析后的 HTML 文件路径（目录型 `out_path` → `…/index.html`）；同步 OpenSpec / SKILL / README
- [x] 9.6 `written_paths` + `changed_paths` 注册，构建产物面板收录 HTML 项目
- [x] 9.7 `markdown/SKILL.md` 模板速选、内容增强写法、resume 封面说明；`clarify` 双栏问法修正
- [x] 9.8 `npm run bundle:js` 重打包 `markdown.bundle.js`
- [x] 9.9 slide viewer 对齐 Marp CLI Bespoke 模板（`bespoke.js` + OSC + `#:$p` 根节点），移除自研 `slide-stage`
- [x] 9.10 review 修复：`@marp-team/marp-cli` vendor 链、`gray-matter` frontmatter、深色 slide 表格样式
