# 提案：PDF 页面操作工具链（add-pdf-operations）

## Why

`add-document-skills-runtime` 已交付 PDF 读取（`office_read_to_markdown` / pdfium）、表格提取（`pdf_extract_table` / pdfsink）、创建（`skill_run` + pdf-lib），但缺少**页面级结构操作**：合并、拆分、旋转、删除页——这是日常 PDF 处理的高频刚需，目前只能让模型现写 pdf-lib JS，稳定性与可测性不足。

## What Changes

- 新增 4 个 Rust 原生工具（基于 `lopdf`）：`pdf_merge`、`pdf_split`、`pdf_rotate`、`pdf_delete_pages`，注册到 `ToolRegistry`。
- 显式声明依赖 `lopdf = "0.38"`：该 crate 已被 `pdfsink-rs` 间接引入并锁定于 0.38.0，显式化**不新增重型依赖、零体积增量**。
- 编写面向本系统的 pdf skill 附属文档 `reference.md`（页面操作工具用法与页码约定）与 `forms.md`（表单处理现状与降级说明），纳入 `assets/skills/pdf/` 并可经 `skill_read` 渐进披露。
- 不改变现有架构（`ToolRegistry` / `Sandbox` / Agent Loop / ipc / 前端均不动）。
- **排除**：表单填值、加密 / 解密、水印 / 叠加、内嵌图片提取、扫描件 OCR（沿用上一变更的排除范围）。

## Capabilities

### New Capabilities

- `pdf-operations`: PDF 页面级合并、拆分、旋转、删除工具，及 PDF skill 的附属知识文档（reference / forms）。

### Modified Capabilities

<!-- openspec/specs/ 暂为空（既有 change 未归档），无可作 base 的已归档 spec；本变更不含 MODIFIED delta。 -->

## Impact

- **代码**：新增 `src-tauri/src/tools/pdf_ops/`（merge / pages 子模块）；`registry.rs` 注册 4 工具；`src-tauri/src/tools/tests.rs` 增测试。前端零改动（工具卡片自动展示）。
- **依赖**：`src-tauri/Cargo.toml` 显式 `lopdf = "0.38"`（已在 Cargo.lock，版本与 pdfsink-rs 一致）。
- **资源**：`src-tauri/assets/skills/pdf/reference.md`、`forms.md` 新增。
- **风险**：① lopdf 合并需正确处理对象重编号与 Pages/Catalog 树（采用官方 merge 范式）；② 加密或损坏 PDF 加载会失败，须明确报错而非 panic；③ `forms.md` / `reference.md` 原文随 `doc_skills/` 删除且从未入 git，已不可恢复，故改为「基于本系统工具能力自写精简版」，而非收录上游原文。
