## Context

- 已有：`pdf_read` + `mode`、`.cache/pdf` 渲染缓存、`vision_subcall`、按批 4 页全量 vision
- 问题：vision 模型 `auto` 对纯文本 PDF 仍全量 vision；`mode` 易被 Agent 误用
- 约束：准确性优先；Judge 允许增加 1 页渲染 + 1 次 vision 子调用，但避免全文档 vision
- `office_read_to_markdown` 保留为「明确不要 Judge/vision」的快速纯文本路径

## Goals / Non-Goals

**Goals:**

- 取消 `mode`；`pdf_read({ path })` 为唯一推荐读 PDF 入口
- vision 模型：硬规则 + **代表页图文对比 Judge** 决定是否全量 vision
- 非 vision 模型：PDFium 全文，扫描件明确报错
- 返回 `resolved`（`text` | `vision`）与 `judge` 元数据（样本页、理由、verdict）
- 代表页选取：suspicion 优先；第 1 页过空时用中间页或字符最多页

**Non-Goals:**

- 多页抽样 Judge（MVP 仅 1 代表页）
- vision 结果缓存（仍只缓存渲染）
- 修改 `pdf_render_pages` / `image_read` 对外 API
- 大学数学 skill

## Decisions

### D1. 无 `mode` 的统一状态机

```
pdf_read(path, pages?, dpi?)
  → extract_text_pages()  // 按页
  → 全文拼接 + 逐页统计

非 vision:
  有文本 → return resolved=text
  无文本 → error（扫描件）

vision + 全文无文本:
  → 全量 vision（不 Judge）

vision + 有文本:
  全文硬规则命中 → 全量 vision
  否则:
    pick_sample_page()
    render 1 页 (pages=N)
    judge_page_compare(image, page_text)
      TEXT_OK → return resolved=text（全文 PDFium）
      NEED_VISION / 解析失败 → 全量 vision
```

### D2. 代表页选取 `pick_sample_page`

输入：`Vec<PageTextStats { index, char_count, suspicion }>`，`page_count`

优先级：

1. `page_count == 1` → 第 1 页
2. 存在 `suspicion >= SUSPICION_THRESHOLD` → 取 suspicion 最高页（并列取较小页码）
3. 第 1 页 `char_count < MIN_CHARS`（默认 80）且 `page_count > 1` → 取 `middle_index = (page_count + 1) / 2`；若 middle 仍过空则取 `char_count` 最大页
4. 默认 → 第 1 页

`suspicion` 由 `pdf_text_quality` 按页计算：`(cid:)` 密度、数学符号、替换符、过短碎片等。

### D3. 全文硬规则（跳过 Judge，直接 vision）

全文拼接文本命中任一则 `need_vision=true`：

- 全文 trim 为空（已在无文本分支处理）
- `(cid:\d+)` 计数 ≥ 3 或密度超阈值
- 替换符 `` `□` `` 占比超阈值
- 全文长度 &lt; 50 且 `page_count >= 1`（可疑扫描+OCR 残留）

硬规则偏保守（准确性优先）。

### D4. Judge：代表页图文对比（1 次 vision subcall）

- 渲染：复用 `render_pages_cached(..., pages_spec=Some("{N}"))`，单页 PNG
- 输入：1 张 `image_url` + prompt 内嵌该页 PDFium 文本（≤4k 字符）
- Prompt：对比图文是否一致；公式/版式/漏字 → `NEED_VISION`；普通叙述一致 → `TEXT_OK`；不确定 → `NEED_VISION`
- 解析：仅接受 `TEXT_OK` / `NEED_VISION`；其他 → `NEED_VISION`
- usage 不计入 session（与现有 vision_subcall 一致）

优于纯文本 Judge：能看见「积分号变 R」等失真。

### D5. 全量 vision 路径

与现实现一致：`render_pages_cached`（`pages` 参数若传入则限制范围）→ `chunks(4)` → `vision_subcall`。

### D6. 工具 schema 与分工

`pdf_read` 参数：`path`（required）、`pages`、`dpi`。描述：「只传 path；系统自动判断是否 vision」。

`office_read_to_markdown`：明确「只要 PDFium、不要 Judge」。

系统提示一行：读 PDF 用 `pdf_read({path})`，勿传 mode。

### D7. 返回 JSON

```json
{
  "resolved": "text" | "vision",
  "markdown": "...",
  "page_count": 12,
  "judge": {
    "skipped": false,
    "method": "page_compare" | "hard_rule" | "no_text_layer",
    "sample_page": 3,
    "sample_reason": "max_suspicion",
    "verdict": "TEXT_OK" | "NEED_VISION"
  },
  "cache_hit": true,
  "cache_key": "...",
  "text_layer": "..."
}
```

`judge.skipped=true` 且 `method=hard_rule|no_text_layer` 时表示未调用 Judge。

## Risks / Trade-offs

- **[Risk] 代表页 OK 但其他页含复杂公式** → 硬规则扫全文 + suspicion 选最高嫌疑页；MVP 后可选 top-2 页 Judge
- **[Risk] Judge 偶发误判** → 不确定时 `NEED_VISION`；返回 `text_layer` 供 Agent 发现后再让用户重试
- **[Risk] 单页渲染 + Judge 增加延迟** → 仍远小于全量 vision；纯文本 PDF 省掉全量 vision
- **[Risk] 与已提交 `mode` 实现不兼容** → **BREAKING**；更新 skill/tests/spec

## Migration Plan

1. 实现 `extract_text_pages`、`pdf_text_quality`、`judge_page_compare`
2. 重写 `pdf_read` handler，删除 `mode` / `resolve_mode` / `parameters_for_model` 分支
3. 更新 registry 工具定义、skill、系统提示、测试
4. `add-pdf-vision-read` 归档时合并能力 spec，再归档本 change 覆盖 `pdf_read` 需求

## Open Questions

- `pages` 参数传入时：Judge 与 vision 仅针对子集；文本返回是否也只返回子集页文本？**建议：是，与渲染范围一致。**
