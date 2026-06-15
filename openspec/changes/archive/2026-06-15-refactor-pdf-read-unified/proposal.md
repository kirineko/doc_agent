## Why

`add-pdf-vision-read` 已落地 `pdf_read` 与 `mode=auto|text|vision`，但 vision 模型在 `auto` 下对**所有** PDF 一律走全量渲染+vision，纯文本 PDF 浪费时间与 API 成本；同时 `mode` 增加 Agent 误用面（如误传 `text`）。需要在**准确性优先**前提下简化 API：取消 `mode`，统一一条智能流程，并由 vision 模型通过**代表页图文对比 Judge** 决定是否全量 vision。

## What Changes

- **BREAKING**：`pdf_read` 移除 `mode` 参数；仅保留 `path`、可选 `pages`、`dpi`
- 统一流程：PDFium 按页提取 → 硬规则快判 →（vision 模型）代表页渲染 + 图文对比 Judge → 返回文本或全量 vision
- 新增按页文本提取、代表页选取、硬规则评分、Judge 子调用模块
- 非 vision 模型：有文本层返回 PDFium 全文；无文本层报错
- vision 模型：无文本层直接全量 vision；有文本层经 Judge 分支
- 更新 pdf skill、系统提示、`office_read_to_markdown` 分工说明
- 测试：纯文本 / 公式 PDF / 扫描件 / 封面空白选页 / Judge mock 分支

## Capabilities

### New Capabilities

- `pdf-read-unified`：`pdf_read` 无 mode 统一流程、按页提取、硬规则、代表页选取、图文 Judge、返回 `resolved` 与 `judge` 元数据

### Modified Capabilities

- `office-tools`：`pdf_read` 与 `office_read_to_markdown` 分工（智能读 vs 纯 PDFium）
- `document-skills`：pdf skill 文档移除 `mode` 说明，只传 `path`

## Impact

- Rust：`pdf_read.rs`（重写门控）、`pdf.rs`（`extract_text_pages`）、`pdf_text_quality.rs`（新）、`llm_subcall.rs` 或扩展 `vision_subcall`（Judge）、`registry.rs`（工具 schema）
- 资源：`assets/skills/pdf/SKILL.md`、`reference.md`
- Agent：`loop_support.rs` 系统提示
- 与 `add-pdf-vision-read` 关系：本变更在其基础上重构 `pdf_read` 语义；归档顺序建议先合并 vision-read 能力 spec，再应用本 change
