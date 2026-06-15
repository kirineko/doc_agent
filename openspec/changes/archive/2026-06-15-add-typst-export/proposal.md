## Why

doc-agent 已有 `html_to_pdf`（WebView 打印 HTML），但公式密集、版式严谨的 PDF（试卷、论文、讲义）需要可编程排版引擎。Typst 为 Rust 原生、可离线嵌入，适合作为与 HTML 并列的**通用 PDF 生成能力**。

## What Changes

- 新增 `typst_to_pdf`：编译沙箱内 `.typ`（或含 `main.typ` 的目录）为 PDF
- 新增 `typst_list_templates` / `typst_read_template`：暴露内置中英模板（报告、试卷、论文、讲义）
- 内置 `assets/typst-templates/`：公共字体与页面模块 + 8 套场景模板
- 字体策略：优先 Windows/macOS 系统字体（微软雅黑、宋体、黑体、Times New Roman 等），回退 Typst 内嵌字体与 Noto CJK
- 注册工具、更新 `toolLabels`；系统提示补充 Typst vs HTML 分工
- **不做** math-skill / exam-skill（后续变更）

## Capabilities

### New Capabilities

- `typst-export`：`typst_to_pdf`、内置模板、字体栈、与 `html_to_pdf` 并列

## Impact

- Rust：`tools/typst_export/`、`registry.rs`、`loop_support.rs`（一行提示）
- 资源：`assets/typst-templates/**`
- 依赖：`typst-as-lib`（`typst-kit-embed-fonts`）、`typst-pdf`
- 安装包体积增加（Typst 引擎 + 内嵌字体）；无运行时网络依赖

## Non-Goals

- 大学数学 / 组卷专用 skill
- Typst 包仓库在线下载（`packages` feature 关闭）
- Linux 安装包支持
