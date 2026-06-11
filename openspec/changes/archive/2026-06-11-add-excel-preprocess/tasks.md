# Tasks: add-excel-preprocess

## 1. 共享预处理基础（preprocess.rs）

- [x] 1.1 新建 `src-tauri/src/tools/data/preprocess.rs`，实现 `load_grid` / `load_xlsx_grid`（含 .xls 经 `convert_legacy` 转换）；首先以编译器核实 calamine 0.35 的 `load_merged_regions()` / `merged_regions_by_sheet()` 返回结构与 `range.start()` 偏移换算，若与 design.md 参考实现不符，先更新 design.md 再写代码
- [x] 1.2 实现 `normalize_headers`（trim/去换行、空名补 `column_N`、重复名 `_2`/`_3`、撞名递增、返回 renamed 映射）+ 单元测试（空名、重复、trim、后缀撞名四组用例）
- [x] 1.3 实现 `fill_merged`（整矩形锚点填充、空锚点跳过）+ 单元测试（纵向/横向/矩形/空锚点）
- [x] 1.4 实现 `suggest_header_row` 启发式 + 单元测试（首行标题行→建议第 1 行；规整表→第 0 行）

## 2. excel_normalize 工具

- [x] 2.1 在 preprocess.rs 实现 `run_normalize`（合并填充→表头提取→列名归一化→数据行补齐/截断→写 CSV→返回 path/rows/columns/renamed；非法 header_row 返回含总行数的 InvalidArgs）
- [x] 2.2 在 `data/mod.rs` 添加 `normalize_tool()` ToolSpec（schema 按 design.md「工具接口定义」）与 `normalize_handler`，并在 `registry.rs::default_tools()` 注册
- [x] 2.3 集成测试：umya-spreadsheet 构造含合并单元格（`add_merge_cells`）、标题行、空/重复表头的 xlsx → normalize → 断言 CSV 表头无空/重复名、合并值已填充、行数正确
- [x] 2.4 集成测试：normalize 产出的 CSV 作为 `data_query` 数据源执行 `SUM` 聚合 → 数值类型推断正确、查询成功

## 3. excel_describe 工具

- [x] 3.1 新建 `src-tauri/src/tools/data/describe.rs`，实现 `run_describe`（sheets/维度/merged_regions(A1 格式+锚点)/preview(行列与单元格截断)/suggested_header_row/warnings）
- [x] 3.2 在 `data/mod.rs` 添加 `describe_tool()` ToolSpec 与 `describe_handler`（preview_rows 默认 15、clamp 1..50），并在 `registry.rs` 注册
- [x] 3.3 集成测试：对 2.3 构造的不规则 xlsx 调用 describe → 断言 merged_regions 含预期 range、warnings 提示空表头/重复列名/合并单元格/表头不在首行、suggested_header_row 正确

## 4. data_query 加固

- [x] 4.1 `query.rs::xlsx_to_dataframe` 表头改走 `preprocess::normalize_headers`；集成测试：空/重复表头 xlsx 直接 `SELECT *` 不再报 duplicate column
- [x] 4.2 `run_query` 记录各数据源 schema，SQL 执行失败时错误信息附「可用表结构」清单与预处理工具提示；集成测试：SQL 引用不存在的列 → 错误含真实列名
- [x] 4.3 更新 `data_query` 工具 description，加入「不规则 Excel 先 excel_describe / excel_normalize」引导

## 5. 收尾

- [x] 5.1 自检文件体量（preprocess.rs / describe.rs ≤ 300 行，超出则拆分）
- [x] 5.2 本地门禁：`cargo fmt --check && cargo clippy -- -D warnings && cargo test`（src-tauri）
- [x] 5.3 用真实问题文件（`报告/3.软件工程专业评估方案指标点-1017-确定版.xlsx`，如可获得）走一遍 describe → normalize → query 管线做手工验收，记录结果到 change 备注（见 `verification.md`：仓库内无该文件，已由集成测试替代验证）
