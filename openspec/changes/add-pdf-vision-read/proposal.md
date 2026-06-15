## Why

`add-multimodal-support` 已交付 `image_read` 与 vision 门控，但 PDF 仍仅能通过 `office_read_to_markdown`（PDFium 纯文本）读取。文字版 PDF 的公式与版式常严重失真，扫描件则完全无文本层。需要将 PDF 页渲染为图片并经 vision 分批理解，同时在非 vision 模型上保留显式文本回退；渲染耗时明显，须用 `.cache/pdf` 缓存命中跳过重复渲染。

## What Changes

- 新增 `pdf_render_pages`：PDFium 将 PDF 页渲染为 PNG，写入 `.cache/pdf/<cache_key>/`，支持 manifest 缓存命中
- 新增 `pdf_read`：统一 PDF 读取入口；默认 `mode=auto`（未传等价）：先 PDFium 文本，vision 模型再渲染+分批 vision；`mode=text`/`mode=vision` 为显式单路径
- **BREAKING**：`image_read` 参数改为仅 `paths`（1–4 张），移除单张 `path`
- 抽取共享 `vision_subcall` helper，供 `image_read` 与 `pdf_read` 复用（多图子调用、usage 不计入 session）
- 更新 pdf skill 文档与 `toolLabels`；`office_read_to_markdown` 对 PDF 可委托 `pdf_read(mode=text)` 或保留文本路径并文档化分工
- 测试：`reference/pdf/` 样例（text vs vision、缓存命中、4 页单批、非 vision 拒绝默认 vision）

## Capabilities

### New Capabilities

- `pdf-vision-read`：`pdf_render_pages`、`pdf_read`、`.cache/pdf` 渲染缓存、vision 分批策略（每批 ≤4 页）

### Modified Capabilities

- `image-read-tool`：`image_read` 仅接受 `paths`（1–4），多图单次 vision 子调用
- `office-tools`：PDF 读取以 `pdf_read` 为主入口；默认 auto 兼容文本与 vision 模型
- `agent-loop`：注册 `pdf_render_pages` / `pdf_read`；vision 分批通过扩展后的 `image_read` 或内部 helper

## Impact

- Rust：`tools/pdf.rs`（+渲染）、`tools/pdf_read.rs`（新）、`tools/image_read.rs`（breaking API）、`tools/office.rs`、`registry.rs`、`agent/loop_support.rs`（若抽 helper）
- 资源：`assets/skills/pdf/SKILL.md`、`reference.md`
- 前端：无必须改动（工具链自动展示）；`toolLabels` / 测试同步
- 依赖：复用已有 `pdfium-render`，不新增 crate
- 排除：大学数学 skill、组卷、OCR 引擎、PDF 内嵌图提取、`.skill-run/` 目录迁移
