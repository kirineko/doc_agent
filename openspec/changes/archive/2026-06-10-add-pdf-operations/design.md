# 设计：PDF 页面操作工具链

## Context

PDF 能力现状（来自 `add-document-skills-runtime`）：

| 维度 | 现有 | 引擎 |
|---|---|---|
| 读取文本 | `office_read_to_markdown` | pdfium-render |
| 表格提取 | `pdf_extract_table` | pdfsink-rs |
| 创建 | `skill_run` + pdf-lib | boa JS 运行时 |
| **页面结构操作** | **缺** | — |

本变更补齐页面级结构操作（合并 / 拆分 / 旋转 / 删除），用纯 Rust 的 `lopdf`。`lopdf 0.38.0` 已被 `pdfsink-rs` 间接引入并锁定，显式声明零体积增量，且与 docx 的「Rust 原生工具链」风格一致（确定性、易单测）。

## Goals / Non-Goals

**Goals**：`pdf_merge` / `pdf_split` / `pdf_rotate` / `pdf_delete_pages` 四工具；沙箱化 I/O；明确错误（越界 / 加密 / 删空）；pdf skill 附属文档。

**Non-Goals**：表单填值、加密 / 解密、水印 / 叠加、内嵌图片提取、OCR；复杂 PDF（AcroForm / Outlines）合并后的完整保真（MVP 仅保证页内容与顺序）。

## 模块划分

```
src-tauri/src/tools/pdf_ops/
  mod.rs      # 4 个 ToolSpec + handler（复用 required_str_arg / ensure_parent_dir）
  merge.rs    # lopdf 官方 merge 范式
  pages.rs    # split / rotate / delete（get_pages / delete_pages / set Rotate）
```

- 现有 `tools/pdf.rs`（pdfium 文本桥）**保持不动**——职责不同（pdfium = 渲染 / 文本，lopdf = 结构）。
- `registry.rs` 追加注册 4 工具；依赖方向不变（`ipc → agent → tools`）。

## Decisions

### D1：合并采用 lopdf 官方 merge 范式
逐文档 `renumber_objects_with(max_id)` 重编号 → 收集各页对象 → 合并唯一 Catalog / Pages 字典 → 重建 `Kids` / `Count` → `renumber_objects()` + `adjust_zero_pages()` + `compress()` → `save()`。Outlines / 书签 MVP 不保留（合并 PDF 常见取舍）。

### D2：拆分 = 反向删除
- **范围模式**（`ranges: "1-3,5"`）：解析为页集合 → `load` → `delete_pages(全集 − 选定)` → `save` 单文件。
- **burst 模式**：对每页 `i`，重新 `load` 原文件 → `delete_pages(除 i 外)` → `save` `out_dir/page_{i}.pdf`。每页独立 load 避免 `Document` 深拷贝复杂度。

### D3：旋转写 `/Rotate`
lopdf 无 rotate 方法。经 `get_pages()` 定位目标 page 的 ObjectId，`get_object_mut` 取 dict，`dict.set("Rotate", angle)`。`mode=absolute` 覆盖；`mode=relative` 读旧值累加后 `% 360`。仅接受 90 的倍数。

### D4：页码 1-based + 错误优先
所有页码 1-based（对齐 `get_pages`）。越界 / 删空 / 空输入 / 加密 / 解析失败一律 `ToolError::Execution`（或 `InvalidArgs`），带文件名 / 页码上下文；不产出半成品文件。lopdf 的 `Result` 全部 `map_err` 包装，禁止 `unwrap` panic。

### D5：附属文档自写（非收录原文）
`forms.md` / `reference.md` 原文随 `doc_skills/` 删除且从未入 git，不可恢复。改为**面向本系统自写**：`reference.md` 覆盖 `pdf_*` 工具用法 + 页码约定 + 与 extract/create 的分工；`forms.md` 说明当前无表单填值工具，降级建议（`office_read_to_markdown` 读取 + 人工/外部处理）。`core/skills.rs` 的 `PDF_DOCS` 追加这两个 `SkillDoc`，主 `SKILL.md` 引用处与实际文档名对齐。

## 工具 I/O 契约

| 工具 | 入参 | 出参 |
|---|---|---|
| `pdf_merge` | `inputs: string[]`, `out_path` | `{ path, pages }` |
| `pdf_split` | `path`, `ranges?`, `mode?("burst")`, `out_path?`, `out_dir?` | `{ files: [] }` 或 `{ path, pages }` |
| `pdf_rotate` | `path`, `rotation(90\|180\|270)`, `pages?`, `mode?`, `out_path` | `{ path, rotated }` |
| `pdf_delete_pages` | `path`, `pages: number[]`, `out_path` | `{ path, pages }` |

## 测试策略

用 `skill_run` + pdf-lib 生成多页 PDF 作 fixture（管道已在上一变更验证可行），再调用各工具，最后用 `lopdf::Document::load` 校验：合并后总页数、拆分子集页数、`/Rotate` 值、删除后页数。加密 / 越界 / 删空走错误分支断言。

## Risks / Open Questions

- **复杂 PDF 合并保真**：带 AcroForm / Outlines / 命名目标的 PDF 合并后可能丢失非页面结构 → MVP 接受，`reference.md` 注明；必要时后续引入 hipdf（lopdf 高层封装）。
- **加密 PDF**：lopdf 对加密文档 load 行为依版本而定 → 统一捕获为明确错误，不在本变更内做解密。
- **lopdf `delete_pages` 对共享资源的处理**：删除页后孤立对象由 `renumber_objects` / save 流程处理，体积非最优但不影响有效性。
