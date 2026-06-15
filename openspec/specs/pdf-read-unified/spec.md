# pdf-read-unified Specification

## Purpose

`pdf_read` 无 `mode` 统一入口：按页 PDFium 提取、硬规则快判、代表页图文 Judge，以及非 vision / vision 模型分支行为。

## Requirements

### Requirement: pdf_read 无 mode 统一入口

系统 SHALL 提供 `pdf_read` 工具，参数 MUST 仅包含 `path`（required）与可选 `pages`、`dpi`。**不得**再接受 `mode` 参数。

所有模型 MUST 走同一调用形态：`pdf_read({"path": "doc.pdf"})`。

#### Scenario: 工具 schema 无 mode

- **WHEN** Agent 获取 `pdf_read` 工具定义
- **THEN** `parameters` 不含 `mode` 字段

### Requirement: 按页 PDFium 提取

`pdf_read` MUST 在决策前对 PDF 执行按页 PDFium 文本提取，得到每页 `char_count` 与文本内容，并拼接为全文 `text_layer`（受 `pages` 范围约束时仅提取对应页）。

#### Scenario: 多页 PDF 按页统计

- **WHEN** 对 4 页 PDF 调用 `pdf_read`
- **THEN** 内部获得 4 条页级文本记录且可用于代表页选取

### Requirement: 非 vision 模型行为

系统 SHALL 在非 vision 会话中仅使用 PDFium 全文结果，且 MUST NOT 触发渲染或 vision 子调用。当 `supports_vision=false` 时：若提取全文非空，MUST 返回 `resolved=text` 与 PDFium 全文 `markdown`；若提取全文为空，MUST 返回错误并提示切换 vision 模型。

#### Scenario: DeepSeek 读取有文本层 PDF

- **WHEN** DeepSeek V4 Flash 调用 `pdf_read({"path": "doc.pdf"})` 且 PDF 有文本
- **THEN** 返回 `resolved=text`，无 `cache_hit` 字段

#### Scenario: DeepSeek 读取扫描件

- **WHEN** 非 vision 模型调用 `pdf_read` 且 PDFium 全文为空
- **THEN** 返回明确错误

### Requirement: vision 模型小页数直接 vision

当会话模型 `supports_vision=true` 且 PDF 页数 **不大于 4** 时，系统 MUST 跳过 Judge 与全文硬规则，直接全量渲染并分批 vision，返回 `resolved=vision`，`judge.method` 为 `page_count_short` 且 `judge.skipped=true`。若有 PDFium 文本 MAY 附带 `text_layer`。

#### Scenario: 4 页 PDF 直接 vision

- **WHEN** Kimi K2.6 对 4 页 PDF 调用 `pdf_read`
- **THEN** 返回 `resolved=vision`，`judge.method` 为 `page_count_short`，不触发代表页 Judge

#### Scenario: 5 页 PDF 仍走 Judge

- **WHEN** Kimi K2.6 对 5 页有文本层 PDF 调用 `pdf_read` 且 Judge 判定 TEXT_OK
- **THEN** `judge.method` 为 `page_compare`，非 `page_count_short`

### Requirement: vision 模型大页数仅文本

当会话模型 `supports_vision=true` 且 PDF 页数 **大于 20** 时，系统 MUST 跳过 Judge 与全量 vision，直接返回 PDFium 全文，`resolved=text`，`judge.method` 为 `page_count_threshold` 且 `judge.skipped=true`，并 MUST 附带 `note` 字段提示可通过 `pages` 分段做 vision。若此时 PDFium 全文为空，MUST 返回明确错误并提示通过 `pages` 分段读取，MUST NOT 触发全量 vision。

#### Scenario: 21 页文本书跳过 vision

- **WHEN** Kimi K2.6 对 21 页有文本层 PDF 调用 `pdf_read`
- **THEN** 返回 `resolved=text` 与 PDFium 全文，`judge.method` 为 `page_count_threshold`，含固定 `note` 提示可 `pages` 分段，不触发渲染或 vision 子调用

#### Scenario: 大页数扫描件报错

- **WHEN** vision 模型对超过 20 页且无文本层 PDF 调用 `pdf_read`
- **THEN** 返回明确错误，提示 `pages` 分段读取，不触发全量 vision

### Requirement: vision 模型无文本层

当会话模型 `supports_vision=true`、PDF 页数 **在 5 至 20 之间（含）** 且 PDFium 全文为空时，MUST 跳过 Judge，直接全量渲染（可命中缓存）并分批 vision，返回 `resolved=vision`。

#### Scenario: 扫描 PDF 直接 vision

- **WHEN** Kimi K2.6 对无文本层 PDF 调用 `pdf_read`
- **THEN** `judge.method` 为 `no_text_layer` 且 `judge.skipped=true`，返回 `resolved=vision`

### Requirement: 全文硬规则快判

当 vision 模型、PDF 页数 **在 5 至 20 之间（含）** 且全文非空时，系统 MUST 对全文执行硬规则检测（如 `(cid:)` 密度、替换符占比、极短全文等）。命中时 MUST 跳过 Judge，直接全量 vision。

#### Scenario: cid-glyphs 触发硬规则

- **WHEN** 全文含大量 `(cid:*)` 模式
- **THEN** `judge.method` 为 `hard_rule` 且走全量 vision

### Requirement: 代表页选取

未命中全文硬规则且页数 **在 5 至 20 之间（含）** 时，系统 MUST 按以下优先级选取 Judge 样本页（1-based）：

1. 仅 1 页 → 第 1 页
2. 存在 suspicion 达阈值的页 → suspicion 最高页
3. 第 1 页字符数低于 `MIN_CHARS` 且总页数 > 1 → 中间页；若仍过空则字符最多页
4. 否则 → 第 1 页

#### Scenario: 封面空白选中间页

- **WHEN** 第 1 页文本极少、第 3 页正文丰富
- **THEN** `judge.sample_page` 不为 1 且 `sample_reason` 反映非首页策略

### Requirement: 代表页图文对比 Judge

系统 MUST 仅渲染代表页为 PNG，并发起 **1 次** vision 子调用：输入该页图片与同页 PDFium 文本，判断图文是否一致。

- 输出 `TEXT_OK` → 返回全文 PDFium，`resolved=text`，不触发全量 vision
- 输出 `NEED_VISION` 或无法解析 → 全量 vision，`resolved=vision`
- Judge 子调用失败 → 保守全量 vision（`JUDGE_FAILED`）
- 不确定时 MUST 视为 `NEED_VISION`（准确性优先）

Judge 子调用 usage MUST NOT 计入 session token 计数。

#### Scenario: 纯文本 PDF Judge 通过

- **WHEN** 代表页图文一致且为普通叙述
- **THEN** 返回 `resolved=text`，`judge.verdict` 为 `TEXT_OK`，`judge.method` 为 `page_compare`

#### Scenario: 公式页 Judge 要求 vision

- **WHEN** 代表页可见公式但 PDFium 文本明显失真
- **THEN** `judge.verdict` 为 `NEED_VISION` 且最终 `resolved=vision`

### Requirement: 全量 vision 路径

当走全量 vision 时，行为 MUST 与 `pdf-vision-read` 一致：渲染（可缓存）→ 每批最多 4 页 vision → 合并 `markdown`；可附带 `text_layer` 与 `cache_hit`。

#### Scenario: Judge 判定后全量 vision

- **WHEN** Judge 判定 `NEED_VISION` 且 PDF 共 9 页
- **THEN** 发起 3 次 vision 子调用（4+4+1）

### Requirement: 返回 judge 元数据

`pdf_read` 返回 JSON MUST 包含 `resolved`（`text` | `vision`）与 `judge` 对象（含 `skipped`、`method`、`sample_page`、`sample_reason`、`verdict`、`followed_by_full_vision` 等适用字段），供 Agent 与验收使用。

#### Scenario: 硬规则跳过 Judge 的元数据

- **WHEN** 全文硬规则命中
- **THEN** `judge.skipped=true` 且 `judge.method` 为 `hard_rule`
