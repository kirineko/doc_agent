# html-export Specification

## Purpose
TBD - created by archiving change add-html-report. Update Purpose after archive.
## Requirements
### Requirement: html_to_pdf 工具

系统 SHALL 提供 `html_to_pdf` 工具，将项目沙箱内**已存在**的 HTML 文档导出为 PDF。工具 MUST 通过隐藏系统 WebView 加载本地 HTML 并执行系统 WebView 的 PDF 输出能力（macOS 使用 WKWebView PDF capture，Windows 使用 WebView2 PrintToPdf），不得依赖用户机器上的 LibreOffice、Chromium 可执行文件或外网服务。

输入 `path` 为项目相对路径：若为 `.html` 文件则直接加载；若为目录则 MUST 加载该目录下的 `index.html`（不存在则返回明确错误）。输出 `out_path` MUST 为项目内 `.pdf` 路径。

本工具与 html-report skill **无强制依赖**：不得要求 HTML 由本系统或本 skill 生成；不得要求路径位于 `reports/` 下。

#### Scenario: 导出单个 HTML 文件

- **WHEN** Agent 对项目内 `docs/summary.html` 调用 `html_to_pdf`，指定 `out_path` 为 `docs/summary.pdf`
- **THEN** 沙箱内生成有效 PDF 文件，且返回包含输出路径与页数的信息

#### Scenario: 导出目录入口

- **WHEN** Agent 对项目内目录 `reports/q1/`（含 `index.html`）调用 `html_to_pdf`
- **THEN** 系统加载 `index.html` 并导出 PDF，且相对路径引用的 `styles.css` 等同目录资源可被正确加载

#### Scenario: 目录无 index 报错

- **WHEN** `path` 指向不含 `index.html` 的目录
- **THEN** 返回明确错误，且不产生空 PDF 文件

#### Scenario: 独立于报告生成

- **WHEN** 用户仅要求将已有 HTML 转为 PDF
- **THEN** Agent 可只调用 `html_to_pdf`，无需调用 `skill_read` 或 `fs_write`

#### Scenario: 越界路径拒绝

- **WHEN** `path` 或 `out_path` 解析后越出项目沙箱
- **THEN** 返回 sandbox 错误，不创建 PDF

### Requirement: 导出参数与超时

`html_to_pdf` SHALL 支持可选参数：`page_size`（默认 `A4`，含 `Letter`）、`landscape`（默认 false）、`margin_mm`（默认 15）。这些参数 MUST 作为打印 CSS / 平台打印配置提示注入；具体分页细节 MAY 因系统 WebView 平台实现存在差异。执行 MUST 设有超时（默认 30 秒）；超时或 WebView 加载失败时 MUST 返回明确错误并清理隐藏窗口，不得阻塞后续工具调用。

#### Scenario: 默认导出参数

- **WHEN** Agent 仅提供 `path` 与 `out_path`
- **THEN** 系统以 A4、纵向、15mm 边距作为导出提示执行，并生成有效 PDF

#### Scenario: 无效参数

- **WHEN** `page_size` 不是 `A4` / `Letter`，或 `margin_mm` 超出允许范围
- **THEN** 返回参数错误，不创建 PDF

#### Scenario: 加载失败

- **WHEN** `path` 指向不存在或不可读的文件
- **THEN** 返回包含路径上下文的错误，且不写入 `out_path`

### Requirement: 平台范围

`html_to_pdf` SHALL 在 macOS（aarch64）与 Windows（x86_64）发布目标上可用。Linux 不在本能力范围内。

#### Scenario: 非支持平台

- **WHEN** 在未支持的平台上调用 `html_to_pdf`
- **THEN** 返回说明平台不支持的错误，而非 panic

