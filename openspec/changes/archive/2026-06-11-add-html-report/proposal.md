# 提案：HTML 报告与导出（add-html-report）

## Why

办公场景常见需求是：用 LLM 把数据整理成**可交付的网页报告**（表格、文字、CSS），并在需要时导出 PDF。当前系统擅长 Office 文档与数据分析，但缺少「静态 HTML 项目」的生成规范与「HTML → PDF」的原生能力；Word/PPT 转 PDF 又因体积约束刻意未做。HTML 报告路径轻量、与 `data_query` 天然衔接，且可用系统 WebView 打印实现 PDF 导出，无需捆绑 LibreOffice / Chromium。

## What Changes

- 新增内置 **`html-report` skill**：指导 Agent 用 `fs_write` / `fs_patch` 在用户**项目目录**内生成静态网页项目（`index.html`、`styles.css`、可选 `script.js` 与 `assets/`），禁止复杂框架与构建链；产物**持久落盘**，不走 `.skill-run/` 等临时目录。
- 新增 Rust 原生工具 **`html_to_pdf`**：对项目内**已存在**的 HTML 文件（或含 `index.html` 的目录）导出 PDF，经隐藏系统 WebView 打印；与报告生成**解耦**，可单独调用。
- **预览**：仅依赖现有文件浏览区「系统默认应用打开」（双击 HTML → 浏览器），不新增应用内预览 UI。
- 更新 `document-skills`：内置 skill 列表与 `skill_read` 枚举增加 `html-report`。
- **排除**：React/Vue/npm 脚手架；应用内 iframe 预览；生成+导出一体化工具；远程 URL 转 PDF；Word/PPT/HTML 以外的格式转 PDF；Linux 安装包支持。

## Capabilities

### New Capabilities

- `html-report`：静态 HTML 报告生成规范、目录约定、skill 文档与落盘约束（必须写入项目沙箱、禁止临时目录）。
- `html-export`：`html_to_pdf` 工具行为、WebView 打印实现约束、与生成能力无强制依赖。

### Modified Capabilities

- `document-skills`：内置 skill 知识库扩展为含 `html-report`（name、description、`skill_read` 枚举与编译期内置文档）。

## Impact

- **代码**：`src-tauri/src/tools/html_export/`（或同级模块）、`registry.rs` 注册 `html_to_pdf`；`loop_runner` / `registry.execute` 为 WebView 工具传入 `AppHandle`（async 特殊路径，同 `web_search`）；`changed_paths.rs`、`toolLabels.ts`；`src-tauri/src/core/skills.rs` 与 `assets/skills/html-report/SKILL.md`。
- **依赖**：Tauri `WebviewWindow`（复用系统 WebView，**不新增** Chromium/LibreOffice）；`Cargo.toml` 可能需启用 Tauri webview 相关 feature。
- **前端**：零必改项（工具卡片自动展示）；可选更新推荐问文案。
- **体积**：预期增量极小（无新重型运行时）。
- **平台**：`html_to_pdf` 面向已发布目标 macOS / Windows；与现有 release 流水线一致。
