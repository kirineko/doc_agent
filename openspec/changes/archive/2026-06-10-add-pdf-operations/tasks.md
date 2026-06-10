# 实施任务：PDF 页面操作工具链

每组附「代码参考」，为骨架而非成品；实现与 `design.md` 冲突时先更新 artifact，再改代码。

## 1. 依赖与模块骨架

- [x] 1.1 `src-tauri/Cargo.toml` 显式 `lopdf = "0.38"`（版本须与 Cargo.lock 现有 0.38.0 一致，确认 `cargo tree -i lopdf` 无版本分裂）
- [x] 1.2 新建 `src-tauri/src/tools/pdf_ops/mod.rs`：声明模块 + 4 个 `ToolSpec`，handler 复用 `tools::required_str_arg` / `ensure_parent_dir`
- [x] 1.3 `tools/mod.rs` 挂载 `pub mod pdf_ops;`

## 2. 合并 pdf_merge（specs/pdf-operations）

- [x] 2.1 `pdf_ops/merge.rs`：按 lopdf 官方 merge 范式实现（`renumber_objects_with` → 收集 Catalog/Pages/Page → 重建 `Kids`/`Count` → `renumber_objects` + `adjust_zero_pages` + `compress` → `save`）
- [x] 2.2 入参校验：`inputs` 空数组 → 明确错误；逐文件 `Document::load` 失败（损坏/加密）→ 带文件名错误，不产出文件
- [x] 2.3 返回 `{ path, pages }`（合并后总页数）

## 3. 拆分 pdf_split（specs/pdf-operations）

- [x] 3.1 `pdf_ops/pages.rs`：范围解析 `"1-3,5"` → 有序去重页集合；越界 → 带页码/总数错误
- [x] 3.2 范围模式：`load` → `delete_pages(全集 − 选定)` → `save out_path`
- [x] 3.3 burst 模式：逐页重新 `load` → `delete_pages(除 i 外)` → `save out_dir/page_{i}.pdf`；返回 `{ files: [] }`

## 4. 旋转与删除（specs/pdf-operations）

- [x] 4.1 `pdf_rotate`：`rotation` 必须为 90 倍数否则报错；`get_pages` 定位目标页 → `get_object_mut` → `dict.set("Rotate", angle)`；`mode=absolute|relative`（relative 读旧值累加 `% 360`）
- [x] 4.2 `pdf_delete_pages`：`delete_pages(pages)`；删空（结果 0 页）→ 明确错误不产出；返回剩余页数
- [x] 4.3 统一错误：所有 lopdf `Result` 经 `map_err` 包装为 `ToolError`，禁止 `unwrap` panic

## 5. 注册与 skill 文档（specs/pdf-operations）

- [x] 5.1 `tools/registry.rs`：注册 `pdf_merge`/`pdf_split`/`pdf_rotate`/`pdf_delete_pages`
- [x] 5.2 自写 `src-tauri/assets/skills/pdf/reference.md`（`pdf_*` 工具用法 + 1-based 页码约定 + 与 `pdf_extract_table`/`office_read_to_markdown`/`skill_run`+pdf-lib 分工，**不含** pypdf/qpdf 外部命令）
- [x] 5.3 自写 `src-tauri/assets/skills/pdf/forms.md`（无表单填值工具的现状 + 降级建议）
- [x] 5.4 `core/skills.rs` 的 PDF 文档清单追加 `reference.md`/`forms.md`；校对主 `SKILL.md` 引用名与实际可读文档一致

## 6. 测试与验收

- [x] 6.1 fixture：`skill_run`+pdf-lib 生成多页 PDF（复用上一变更管道）
- [x] 6.2 合并测试：merge 后 `Document::load` 校验总页数 = 各输入之和、顺序正确
- [x] 6.3 拆分测试：范围子集页数正确；burst 生成 N 个单页文件
- [x] 6.4 旋转测试：load 后断言目标页 `/Rotate` 值；非法角度走错误分支
- [x] 6.5 删除测试：剩余页数正确；删空报错
- [x] 6.6 `skill_read {"skill":"pdf","doc":"reference.md"}` 返回自写文档
- [x] 6.7 `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test` 全绿
