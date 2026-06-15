# PDF 页面操作参考（doc-agent）

本系统通过 Rust 原生工具处理 PDF 页面结构。页码一律 **1-based**（第 1 页 = 1），与 `lopdf` 一致。

## 工具分工

| 任务 | 工具 | 说明 |
|---|---|---|
| **读取 PDF（推荐，所有模型）** | `pdf_read` | 仅 path；vision 模型先 PDFium 再 Judge/vision |
| 仅 PDFium 纯文本（跳过 Judge） | `office_read_to_markdown` | PDFium 文本；公式可能失真 |
| 手动渲染页图（高级） | `pdf_render_pages` | PNG 写入 `.cache/pdf/`；`pdf_read` 内部会自动渲染 |
| 理解 1–4 张图 | `image_read` | `paths` 数组；可读 `.cache/pdf/` 页图 |
| 合并 PDF | `pdf_merge` | 按顺序拼接多个文件 |
| 拆分 PDF | `pdf_split` | 按范围或 burst 每页一个文件 |
| 旋转页面 | `pdf_rotate` | 90 / 180 / 270 度 |
| 删除页面 | `pdf_delete_pages` | 删除指定页 |
| 创建新 PDF | `skill_run` + pdf-lib | 从零绘制或生成 |
| SQL 整理 | `data_query` | 对提取出的 CSV 聚合 |

## pdf_merge

```json
{
  "inputs": ["part1.pdf", "part2.pdf"],
  "out_path": "merged.pdf"
}
```

返回 `{ "path", "pages" }`。`inputs` 至少一项；加密或损坏文件会返回带文件名的错误。

## pdf_split

**按范围**（保留 `1-3,5` 等页到单文件）：

```json
{
  "path": "in.pdf",
  "ranges": "1-3,5",
  "out_path": "subset.pdf"
}
```

**burst**（每页一个文件）：

```json
{
  "path": "in.pdf",
  "mode": "burst",
  "out_dir": "pages"
}
```

生成 `pages/page_1.pdf`、`page_2.pdf` … 返回 `{ "files": [...] }`。越界页码返回明确错误。

## pdf_rotate

```json
{
  "path": "in.pdf",
  "rotation": 90,
  "pages": [2],
  "mode": "absolute",
  "out_path": "rotated.pdf"
}
```

- `rotation` 必须为 90 的倍数。
- 省略 `pages` 时旋转全部页。
- `mode`: `absolute`（覆盖）或 `relative`（在现有角度上累加）。

## pdf_delete_pages

```json
{
  "path": "in.pdf",
  "pages": [2, 4],
  "out_path": "out.pdf"
}
```

不能删除全部页；删空会报错。

## pdf_read

```json
{ "path": "exam.pdf" }
```

**所有模型**：仅 `path`（可选 `pages`、`dpi`），勿传 `mode` 等已移除参数。

**vision 模型（Kimi K2.6、MiMo v2.5）**：先 PDFium 按页提取。**≤4 页**直接全量 vision（`page_count_short`）；**超过 20 页**直接返回文本并附带 `note`（`page_count_threshold`）；**5–20 页**经硬规则或 Judge 决定。

**非 vision 模型**：返回 PDFium 全文；扫描件（无文本层）报错并提示切换 vision 模型。

仅当明确不要 Judge、只要 PDFium 时用 `office_read_to_markdown`。

可选：`pages`（如 `"1-4"` 或 `[1,3]`）、`dpi`（默认 150，72–300）。

返回含 `resolved`（`text` | `vision`）、`judge` 元数据；超过 20 页纯文本时附带固定 `note`（提示可 `pages` 分段 vision）。

## pdf_render_pages

```json
{ "path": "exam.pdf", "pages": "all", "dpi": 150 }
```

通常无需手动调用——`pdf_read` 在需要 vision 时会自动渲染。用于高级流程：先渲染再 `image_read` 指定页。

返回 `cache_hit`、`pages`（PNG 相对路径）、`manifest_path`。相同源文件与参数再次调用时 `cache_hit: true`，跳过渲染。

## image_read（多图）

```json
{
  "paths": [".cache/pdf/<key>/page_001.png", ".cache/pdf/<key>/page_002.png"],
  "prompt": "按顺序提取文字与公式"
}
```

`paths` 长度 1–4，无单张 `path` 参数。

## 典型流程

1. **读取内容（默认）**：`pdf_read` 仅 `path`
2. **明确只要 PDFium**：`office_read_to_markdown`
3. `pdf_split` / `pdf_delete_pages` / `pdf_rotate` 调整结构
4. `pdf_merge` 合并交付物
5. 需表格数据时：先用 `office_read_to_markdown` 提取文本，再 `data_query` 或 `skill_run`+exceljs
6. 需新建或重绘：`skill_run`（pdf-lib 已内置）

## 限制

- 合并不保证保留书签、表单、注释等复杂结构，仅保证页面内容与顺序。
- 加密 PDF 当前不支持解密后操作。
- 扫描件：用 vision 模型 `pdf_read`，勿仅用 `office_read_to_markdown`
