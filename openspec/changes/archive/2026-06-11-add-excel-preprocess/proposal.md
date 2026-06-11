# Proposal: add-excel-preprocess

## Why

`data_query`（polars-sql）当前假设 Excel 第 0 行就是干净表头，但真实业务文件大量存在合并单元格、空/重复列名、表头不在首行等不规则结构，导致 DataFrame 构建失败（duplicate column）且报错不含任何结构信息，模型只能盲猜反复试探，浪费多轮工具调用仍无法完成分析（实际案例：`软件工程专业评估方案指标点.xlsx` 因合并表头产生多个空列名，SQL 全部失败）。

## What Changes

- 新增 `excel_describe` 工具：结构侦察——返回 sheet 列表、维度、合并单元格区域、前 N 行原始预览、推测表头行与结构警告（空列名/重复列名），供模型判断文件结构
- 新增 `excel_normalize` 工具：确定性清洗——按模型指定的参数（表头行、合并单元格填充、列名去重）将不规则 xlsx/xls 转为干净 CSV 写入沙箱，供 `data_query` 复用
- 加固 `data_query` 的 xlsx 加载层：空/重复列名自动改名（不再直接失败）；SQL 执行失败时报错附带各数据源的实际列名，让模型一轮自我修正
- 工具描述（tool description）引导模型遵循「describe → normalize → query」管线处理不规则文件

不纳入本期（后续按需另立 change）：多行表头拼接、单 sheet 多表区域（range）提取、小计/合计行清除（模型可用 SQL `WHERE` 自行排除）。

## Capabilities

### New Capabilities

- `excel-preprocess`: Excel 不规则数据预处理——结构侦察（`excel_describe`）与确定性清洗（`excel_normalize`），产出可被 SQL 直接消费的干净 CSV

### Modified Capabilities

- `data-analysis`: 「SQL 数据整理」需求变更——xlsx 数据源遇空/重复列名 SHALL 自动改名而非失败；SQL 失败时 SHALL 返回含数据源 schema 的可自愈错误信息

## Impact

- 新增代码：`src-tauri/src/tools/data/describe.rs`、`src-tauri/src/tools/data/normalize.rs`
- 修改代码：`src-tauri/src/tools/data/mod.rs`（注册 2 个新工具）、`src-tauri/src/tools/data/query.rs`（复用列名归一化、增强报错）、`src-tauri/src/tools/registry.rs`（注册）
- 依赖：无新增 crate（calamine 0.35 已支持 `load_merged_regions()`，umya-spreadsheet 仅测试用例构造合并单元格时使用，均为现有依赖）
- 不影响前端与 IPC 层；Agent 通过现有工具调用协议使用新工具
