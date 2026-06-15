# pdf-vision-read Specification

## Purpose

PDF 页渲染缓存、`pdf_render_pages` 工具，以及 `pdf_read` 全量 vision 路径（分批理解、缓存命中）的行为约束。

## Requirements

### Requirement: pdf_render_pages 工具

系统 SHALL 提供 `pdf_render_pages` 工具，将项目沙箱内 PDF 指定页渲染为 PNG 并写入 `.cache/pdf/<cache_key>/`。参数 MUST 包含 `path`（项目相对 PDF 路径）；可选 `pages`（1-based 范围，如 `1-4` 或 `1,3,5`，默认全部页）、`dpi`（默认 150）。

渲染前 MUST 根据 `source_path`、`source_size`、`source_mtime_secs`、`dpi`、`pages_spec` 计算 `cache_key`。若对应目录下 `manifest.json` 存在且字段一致、且全部 `page_NNN.png` 文件存在，MUST 跳过 PDFium 渲染并返回 `cache_hit: true`；否则重渲并写入新 manifest。

#### Scenario: 首次渲染写入缓存

- **WHEN** Agent 对 `exam.pdf`（4 页）调用 `pdf_render_pages` 且缓存不存在
- **THEN** 系统生成 `.cache/pdf/<key>/page_001.png` … `page_004.png` 与 `manifest.json`，返回 `cache_hit: false` 与页图路径列表

#### Scenario: 相同文件与参数命中缓存

- **WHEN** 源 PDF 未修改且 `dpi` 与 `pages` 与上次相同，再次调用 `pdf_render_pages`
- **THEN** 返回 `cache_hit: true`，不调用 PDFium 渲染，页图路径与 manifest 一致

#### Scenario: 源文件修改后缓存失效

- **WHEN** 源 PDF 被覆盖导致 `size` 或 `mtime` 变化
- **THEN** 系统使用新 `cache_key` 重新渲染，不返回旧页图

### Requirement: 缓存目录布局

PDF 渲染产物 MUST 存放在项目相对路径 `.cache/pdf/<cache_key>/`。`manifest.json` MUST 记录源文件元数据、渲染参数、页图相对路径与 `page_count`。该目录 MUST NOT 出现在用户文件浏览与 `@` 候选列表（沿用点目录隐藏规则）。

#### Scenario: 页图路径可供 image_read 读取

- **WHEN** `pdf_render_pages` 成功
- **THEN** 返回的 `page_NNN.png` 路径位于 `.cache/pdf/` 下且可被 `image_read` 的 `paths` 引用

### Requirement: vision 批大小上限

`pdf_read` 全量 vision 路径每批送入 vision 的图片数量 MUST NOT 超过 4，与 `MAX_ATTACHMENTS_PER_MESSAGE` 及 `image_read` 的 `paths` 上限一致。

#### Scenario: 4 页单批

- **WHEN** 全量 vision 路径处理恰好 4 页
- **THEN** 仅发起 1 次多图 vision 子调用

#### Scenario: 9 页分三批

- **WHEN** 全量 vision 路径处理 9 页 PDF
- **THEN** 发起 3 次 vision 子调用（4+4+1 页），合并为单一文本结果
