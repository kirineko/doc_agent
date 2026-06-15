## ADDED Requirements

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

### Requirement: pdf_read 统一读取

系统 SHALL 提供 `pdf_read` 工具作为 PDF 内容理解的主入口。参数 MUST 包含 `path`；可选 `mode`（`text` | `vision` | `auto`）、`pages`、`dpi`（传递给渲染阶段）。

**默认行为**：`mode` 未传时 MUST 等价于 `mode=auto`。

**`mode=auto`（或未传）**：MUST 先使用 PDFium 提取文本；再根据会话模型 `supports_vision` 分支：
- `supports_vision=false`：若有文本层则返回 PDFium 文本（`resolved=text`）；若文本为空（扫描件）则返回错误，提示切换 vision 模型。
- `supports_vision=true`：在提取文本后 MUST 继续走渲染（可命中缓存）→ 按每批最多 4 页 vision 理解 → 合并为 Markdown 返回（`resolved=vision`）；若有非空文本层 MAY 在结果中附带 `text_layer` 字段。auto 路径 MUST NOT 因文本已足够而跳过 vision。

**`mode=text`**：MUST 仅使用 PDFium 文本提取（与 `office_read_to_markdown` PDF 分支等价），不触发渲染或 vision。

**`mode=vision`**：MUST 仅走渲染 + vision 路径（不先返回文本结果）；会话模型 `supports_vision=false` 时 MUST 返回明确错误。

向 vision 模型暴露的 `pdf_read` 工具 schema MUST NOT 包含 `mode=text`（避免 Agent 误选）；若仍传入 `mode=text`，实现 MUST 按 `auto` 处理。vision 模型纯文本需求应使用 `office_read_to_markdown`。

当 `mode=text` 且提取结果为空时，MUST 返回说明可能为扫描件、建议使用 vision 模型或 `mode=auto`/`mode=vision` 的错误。

#### Scenario: 默认与 auto 等价

- **WHEN** Agent 调用 `pdf_read({"path": "doc.pdf"})` 未传 `mode`
- **THEN** 行为与 `pdf_read({"path": "doc.pdf", "mode": "auto"})` 完全一致

#### Scenario: vision 模型 auto 读取

- **WHEN** 会话为 Kimi K2.6，Agent 调用 `pdf_read({"path": "doc.pdf"})` 或 `mode=auto`
- **THEN** 系统先 PDFium 提取文本，再渲染（或命中缓存）并分批 vision，返回 `mode=auto`、`resolved=vision` 与合并后的 Markdown

#### Scenario: 非 vision 模型 auto 返回文本

- **WHEN** 会话为 DeepSeek V4 Flash，Agent 调用 `pdf_read({"path": "doc.pdf"})` 未传 mode 且 PDF 有文本层
- **THEN** 返回 `mode=auto`、`resolved=text` 与 PDFium 文本，不触发渲染或 vision

#### Scenario: 非 vision 模型显式 text

- **WHEN** 会话为 DeepSeek V4 Flash，Agent 调用 `pdf_read({"path": "doc.pdf", "mode": "text"})` 且 PDF 有文本层
- **THEN** 返回 `mode=text` 与 PDFium 提取的文本内容

#### Scenario: 非 vision 模型显式 vision 拒绝

- **WHEN** 会话为 DeepSeek V4 Flash，Agent 调用 `pdf_read({"path": "doc.pdf", "mode": "vision"})`
- **THEN** 返回错误，提示需要 vision 模型或使用 `mode=auto`/`mode=text`

#### Scenario: 9 页 PDF 分三批 vision

- **WHEN** vision 模型对 9 页 PDF 调用 `pdf_read` 且 `mode=vision` 或 `mode=auto`
- **THEN** 系统发起 3 次 vision 理解（4+4+1 页），合并为单一文本结果

#### Scenario: 扫描件 text 模式报错

- **WHEN** `mode=text` 且 PDFium 提取为空字符串
- **THEN** 返回明确错误，说明无文本层并建议使用 vision 模型

#### Scenario: 扫描件 auto 非 vision 报错

- **WHEN** 非 vision 模型、`mode=auto` 且 PDFium 提取为空
- **THEN** 返回明确错误，提示切换 vision 模型后重试

### Requirement: 缓存目录布局

PDF 渲染产物 MUST 存放在项目相对路径 `.cache/pdf/<cache_key>/`。`manifest.json` MUST 记录源文件元数据、渲染参数、页图相对路径与 `page_count`。该目录 MUST NOT 出现在用户文件浏览与 `@` 候选列表（沿用点目录隐藏规则）。

#### Scenario: 页图路径可供 image_read 读取

- **WHEN** `pdf_render_pages` 成功
- **THEN** 返回的 `page_NNN.png` 路径位于 `.cache/pdf/` 下且可被 `image_read` 的 `paths` 引用

### Requirement: vision 批大小上限

`pdf_read` vision 路径每批送入 vision 的图片数量 MUST NOT 超过 4，与 `MAX_ATTACHMENTS_PER_MESSAGE` 及 `image_read` 的 `paths` 上限一致。

#### Scenario: 4 页单批

- **WHEN** vision 路径处理恰好 4 页
- **THEN** 仅发起 1 次多图 vision 子调用
