# HTML Report Skill

用 LLM 在用户**项目目录**内生成静态网页报告（表格、文字、CSS）。与 PDF 导出**解耦**——本 skill 只规范如何**生成** HTML；需要 PDF 时另行调用 `html_to_pdf`。

## 工具分工

| 任务 | 工具 | 说明 |
|------|------|------|
| 生成 HTML/CSS/JS | `fs_write` / `fs_patch` | **主路径**；产物持久落在项目沙箱 |
| 数据分析填表 | `data_query` | 可选；先汇总再写入 HTML |
| 导出 PDF | `html_to_pdf` | **可选**；输入为已存在的 HTML，与是否刚生成本报告无关 |
| 预览 | 文件浏览区双击 | 系统默认浏览器打开，无应用内预览 |

## 落盘规则（必须遵守）

- 所有报告文件 MUST 写在用户选定的**项目根目录**内（如 `reports/q1-sales/`）。
- **禁止**写入 `.skill-run/` 或任何 turn 内临时目录。
- **禁止**使用 React、Vue、npm、Vite 等框架或构建链。
- 资源只用**相对路径**（`./styles.css`、`./assets/logo.svg`）；避免外网 CDN，保证离线打开与 PDF 导出时样式完整。

## 推荐目录结构

```
reports/<报告名>/
├── index.html      # 入口（必须）
├── styles.css      # 样式（推荐）
├── script.js       # 可选，仅简单逻辑
└── assets/         # 可选，SVG/小图
```

路径不必强制 `reports/`，但推荐该前缀便于用户识别。

## 工作流示例

### 仅生成 HTML 报告

```json
{"tool": "skill_read", "args": {"skill": "html-report"}}
```

然后：

```json
{"tool": "fs_write", "args": {"path": "reports/q1-sales/styles.css", "content": "..."}}
{"tool": "fs_write", "args": {"path": "reports/q1-sales/index.html", "content": "..."}}
```

**不要**在此流程末尾自动调用 `html_to_pdf`，除非用户明确要求导出 PDF。

### 可选：数据驱动

```json
{"tool": "data_query", "args": {"sources": [...], "sql": "...", "out_path": "reports/q1-sales/data.csv"}}
```

将查询结果内联进 HTML 表格，或让 `script.js` 读取同目录 CSV（保持简单，勿引入打包工具）。

### 单独导出 PDF（与生成无关）

```json
{"tool": "html_to_pdf", "args": {"path": "reports/q1-sales/index.html", "out_path": "reports/q1-sales/report.pdf"}}
```

`path` 也可以是目录（自动找 `index.html`）。

## HTML 模板要点

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <link rel="stylesheet" href="./styles.css" />
</head>
<body>
  <h1>报告标题</h1>
  <table>...</table>
</body>
</html>
```

`styles.css` 须含打印样式：

```css
@page { size: A4; margin: 15mm; }
@media print {
  body { font-family: "PingFang SC", "Microsoft YaHei", sans-serif; }
  table { page-break-inside: avoid; }
  h1, h2, h3 { page-break-after: avoid; }
}
```

## 交付前检查

- [ ] 文件均在项目目录内，**不在** `.skill-run/`
- [ ] `index.html` 可用相对路径加载 CSS
- [ ] 中文使用系统字体
- [ ] 含 `@page` / `@media print`
- [ ] 未使用 npm/框架
- [ ] 用户未要求 PDF 时，未调用 `html_to_pdf`
