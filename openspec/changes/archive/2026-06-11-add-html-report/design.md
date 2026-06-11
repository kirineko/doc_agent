# 设计：HTML 报告与导出

## Context

- 用户项目目录即沙箱根；`fs_write` / `fs_patch` 经 `Sandbox::resolve_for_write` 持久落盘。
- `.skill-run/` 仅用于 `skill_run` 脚本与错误现场，turn 结束清理，**不得**作为报告产物目录。
- 已有 `office_read_to_markdown`（读 PDF）、`pdf_*`（改 PDF）、`skill_run`+pdf-lib（画 PDF），但无 HTML→PDF。
- 产品约束（`openspec/project.md`）：体积尽量小，不捆绑 LibreOffice / Chromium；发布目标 macOS / Windows。
- 探索结论：表格+文字+CSS 报告适合**系统 WebView 打印**；预览用现有文件浏览 + `tauri_plugin_opener::open_path`。

## Goals / Non-Goals

**Goals：**

- 内置 `html-report` skill，规范 Agent 用 `fs_write` 等在项目内生成静态网页项目。
- `html_to_pdf` 工具：对项目内已有 HTML 导出 PDF；与生成解耦。
- 产物持久在用户项目目录；文件浏览区可见；双击 HTML 用系统浏览器预览。
- 测试：sample HTML → PDF 冒烟（文件存在、页数 > 0）。

**Non-Goals：**

- 应用内 iframe/WebView 预览 UI。
- `report_generate` 等生成+导出一体化工具。
- npm/Vite/React 等构建链；远程 URL 转 PDF。
- Word/PPT 转 PDF；Linux 安装包。
- 复杂 Canvas 图表「等渲染完成」逻辑（MVP 仅固定短延迟可选）。

## Decisions

### D1：生成走 `fs_write`，不走临时目录

- **选择**：`html-report` skill 规定主路径为 `fs_write` / `fs_patch` 写入 `reports/<name>/`（或用户指定项目内路径）。
- **理由**：与 Office 文档一致，产物持久、可 `@` 引用、浏览区可见。
- **禁止**：将 `index.html` 写入 `.skill-run/`；不得以 `skill_run` 作为主生成手段（`skill_run` 仅保留给 base64 小资源等边缘场景，且目标路径仍须在项目内）。
- **备选**：专用 `html_write` 工具 — 与 `fs_write` 重复，否决。

### D2：生成与导出解耦

- **选择**：`html-report`（skill）与 `html_to_pdf`（工具）无调用依赖；skill 文案不得要求「生成后必须导出」。
- **理由**：用户明确两种场景独立（只生成 / 只导出 / 串联由 Agent 自行组合）。
- **备选**：单一 `report_export` 包装两步 — 耦合过高，否决。

### D3：PDF 渲染用隐藏系统 WebView 的 PDF 输出能力

- **选择**：创建不可见 `WebviewWindow`，加载沙箱内 HTML 的 `file://` URL，load 完成后调用平台 PDF 输出能力（macOS `WKWebView.createPDF`，Windows `WebView2.PrintToPdf`）。
- **理由**：零 Chromium 捆绑，CSS 表格排版保真可接受。
- **备选**：捆绑 headless Chrome（体积）、wkhtmltopdf（外部依赖）、pdf-lib 手画（无法渲 HTML）— 均否决。
- **注意**：macOS `createPDF` 是 WebView capture API，不是完整打印管线；`page_size` / `margin_mm` 在 macOS 作为注入的打印 CSS 提示，实际分页可能与 Windows WebView2 PrintToPdf 存在差异。

### D4：`html_to_pdf` 走 async + `AppHandle`

- **选择**：在 `ToolRegistry::execute` 与 `loop_runner` 中为 `html_to_pdf` 增加 async 分支，传入 `AppHandle`（同 `web_search` 模式）；`ToolContext` 不扩展。
- **理由**：WebView 创建属 Tauri 主进程，同步 handler 无法完成。
- **实现要点**：`block_on` 或 `async` handler；超时（默认 30s）；完成后销毁隐藏窗口。

### D5：输入路径解析

- `path` 为 `.html` 文件 → 直接加载。
- `path` 为目录 → 查找该目录下 `index.html`；缺失则明确错误。
- `out_path` 必须为项目内 `.pdf` 路径；父目录不存在则创建。
- 不要求路径在 `reports/` 下。

### D6：预览零开发

- 复用 `list_project_dir` + IPC `open_project_file`；skill 说明「双击 HTML → 系统默认浏览器」。
- 不修改 `workspace-ui` spec。

### D7：模块划分

```
src-tauri/src/tools/html_export/
  mod.rs       # ToolSpec + async handler 入口
  print.rs     # WebviewWindow 生命周期、file URL、平台打印
src-tauri/assets/skills/html-report/
  SKILL.md     # 目录约定、模板、打印 CSS、工具分工
src-tauri/src/core/skills.rs   # 注册 html-report
```

- `registry.rs` 注册 `html_to_pdf`；`changed_paths.rs` 记录 `out_path`。
- `loop_runner.rs`：`execute` 传入 `&app` 给 html 导出分支。

### D8：`html_to_pdf` 参数

| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `path` | string | 必填 | 项目内 `.html` 或含 `index.html` 的目录 |
| `out_path` | string | 必填 | 输出 `.pdf` |
| `page_size` | enum | `A4` | `A4` / `Letter`（打印 CSS / 平台打印提示） |
| `landscape` | bool | false | 横向 |
| `margin_mm` | number | 15 | 页边距 |

返回 `{ "path": "...", "pages": N }`（页数经 pdfium 或 lopdf 读取验证，实现时择一）。

## Risks / Trade-offs

- [WebView `file://` 相对资源路径失败] → 以 HTML 所在目录为 base URL；skill 强制相对路径；测试含 `styles.css` 外链。
- [打印 CSS 未设导致分页丑] → skill 提供 `@page` / `@media print` 模板与 checklist。
- [Windows/macOS 打印 API 差异] → 分平台实现；自动化覆盖参数/路径错误分支，真实 PDF 渲染需桌面应用手验。
- [同步工具线程阻塞 WebView] → async 专用路径 + 超时销毁窗口。
- [用户 HTML 含外网 CDN] → skill 建议离线资源；失败时错误提示检查网络资源。

## Migration Plan

- 纯增量：新 skill + 新工具，无数据迁移。
- 回滚：移除 registry 注册与 skill 条目即可。

## Open Questions

- ~~Tauri 2 启用 `WebviewWindow` 所需 feature 清单~~ — **已确认**：默认 `tauri` feature 即可创建 `WebviewWindow`；单元测试用 `tauri` 的 `test` feature + `mock_app()`。
- ~~Windows WebView2 `PrintToPdf`~~ — **已实现**：`ICoreWebView2_7::PrintToPdf` + `PrintToPdfCompletedHandler`（`webview2-com 0.38`）。
- macOS：`WKWebView::createPDFWithConfiguration_completionHandler`（`objc2-web-kit 0.3`）。
- 自动化端到端 PDF 冒烟在 `mock_app` 下会超时；当前仅保留 sample fixture 与错误分支单测，真实导出需在桌面应用中手验。
