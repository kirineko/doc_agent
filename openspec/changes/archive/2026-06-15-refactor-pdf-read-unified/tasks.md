## 1. 提取与质量评估基础

- [x] 1.1 `pdf.rs` 新增 `extract_text_pages`（按页文本 + 页码）
- [x] 1.2 新增 `pdf_text_quality.rs`：按页/全文 `suspicion`、硬规则、`pick_sample_page`
- [x] 1.3 单元测试：代表页选取（封面空白、max suspicion、单页）

## 2. Judge 子调用

- [x] 2.1 实现 `judge_page_compare`（1 图 + 同页文本，vision subcall）
- [x] 2.2 解析 `TEXT_OK` / `NEED_VISION`；无法解析时保守 `NEED_VISION`
- [x] 2.3 Mock 测试 Judge 两种 verdict

## 3. pdf_read 重写

- [x] 3.1 **BREAKING** 移除 `mode` 及相关 `resolve_mode` / `parameters_for_model` 分支
- [x] 3.2 实现统一状态机（非 vision / 无文本 / 硬规则 / Judge / 全量 vision）
- [x] 3.3 返回 `resolved` + `judge` 元数据；`pages` 范围约束提取与渲染
- [x] 3.4 更新 `registry` 工具 schema 与描述

## 4. 文档与提示

- [x] 4.1 更新 `SKILL.md`、`reference.md`（无 mode）
- [x] 4.2 更新 `loop_support` 系统提示
- [x] 4.3 更新 `office_read_to_markdown` 描述分工

## 5. 集成测试

- [x] 5.1 非 vision：有文本 / 扫描件报错
- [x] 5.2 vision：无文本层直接全量 vision
- [x] 5.3 vision：纯文本样例 Judge → `resolved=text`（mock）
- [x] 5.4 vision：公式样例硬规则或 Judge → `resolved=vision`（mock）
- [x] 5.5 删除/更新旧 `mode` 相关测试

## 6. 验收（手动）

- [x] 6.1 Kimi：纯文本书 PDF → 快速返回文本，工具链无全量 vision
- [x] 6.2 Kimi：高数公式 PDF → 全量 vision，公式可读
- [x] 6.3 Kimi：封面+正文 PDF → `judge.sample_page` 非空白封面页
