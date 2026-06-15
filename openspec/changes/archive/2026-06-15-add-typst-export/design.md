## Context

- 已有 `html_to_pdf`（WebView 打印），适合图文报告；公式、编号、交叉引用需 Typst。
- 依赖已加入：`typst-as-lib`（`typst-kit-embed-fonts`）、`typst-pdf`。
- 不启用 `packages` feature，模板通过虚拟路径 `/doc-agent/typst/**` 静态挂载。

## Goals

- `typst_to_pdf`：编译沙箱内 `.typ` 或 `main.typ` 目录 → PDF
- 内置 8 套中英场景模板 + 公共 `fonts.typ` / `page.typ`
- 字体：优先系统（微软雅黑、宋体、黑体、Times New Roman 等），回退 Typst 内嵌与 Noto CJK

## Non-Goals

- math/exam skill
- 在线 Typst 包仓库
- Linux 安装包

## Decisions

### 引擎与解析器

```text
TypstEngine::builder()
  .search_fonts_with(TypstKitFontOptions::default())  // 系统 + embed
  .with_static_source_file_resolver(bundled templates)
  .with_file_system_resolver(sandbox.root())
```

用户主文件用相对 vpath（如 `docs/exam.typ`）；`#import "/doc-agent/typst/..."` 走静态解析器。

### 工具 API

| 工具 | 说明 |
|------|------|
| `typst_to_pdf` | `path`, `out_path`；60s 超时；`spawn_blocking` |
| `typst_list_templates` | 返回 id / category / lang / title / import_path |
| `typst_read_template` | `template` id → 源码，供 `fs_write` 复制 |

### 字体栈（`common/fonts.typ`）

- 中文正文：Times New Roman → SimSun → Songti SC → Noto Serif CJK SC
- 中文无衬线：Microsoft YaHei → PingFang SC → SimHei → Noto Sans CJK SC
- 英文：Times New Roman → Libertinus Serif；无衬线 Arial → Noto Sans
- 数学：`New Computer Modern Math`（Typst 内嵌）

不捆绑微软字体；Windows 上自动命中系统字体。

### 与 html_to_pdf 分工

- 公式密集、试卷、论文、讲义 → `typst_to_pdf`
- 快速图文 HTML 报告 → `html_to_pdf`

## Risks

- 首次编译含字体搜索，可能较慢 → 60s 超时 + blocking 线程
- CI（Ubuntu）无中文字体 → 回退内嵌字体，模板仍可编译

## Verification

- `cargo test`：列表/读取模板、沙箱内最小 `.typ` 编译
- 本地：各内置模板 `typst_to_pdf` 冒烟（可选）
