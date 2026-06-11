# excel-preprocess Specification

## Purpose
Excel 不规则数据预处理——结构侦察（`excel_describe`）与确定性清洗（`excel_normalize`），产出可被 SQL 直接消费的干净 CSV。

## Requirements

### Requirement: Excel 结构侦察
系统 SHALL 提供 `excel_describe` 工具，对沙箱内 xlsx / xls 指定工作表返回结构信息：sheet 列表、行列维度、合并单元格区域（A1 格式 range 与锚点值）、前 N 行原始预览（默认 15 行，单元格值截断至 80 字符、列数截断至 30）、推测表头行索引（0-based）、结构警告（空表头、重复列名、合并单元格存在、表头不在首行）。

#### Scenario: 侦察含合并单元格的不规则表
- **WHEN** Agent 对含合并单元格 `A2:A4` 且首行为标题行的 xlsx 调用 `excel_describe`
- **THEN** 返回 `merged_regions` 含 `A2:A4` 及其锚点值，`suggested_header_row` 指向真实表头行，`warnings` 提示合并单元格与表头位置

#### Scenario: 侦察空/重复表头
- **WHEN** 推测表头行存在空单元格或重复列名
- **THEN** `warnings` 中明确列出空表头数量与位置、重复列名及自动改名规则提示

#### Scenario: xls 旧格式支持
- **WHEN** Agent 对 `.xls` 文件调用 `excel_describe`
- **THEN** 系统经 legacy 转换后完成侦察，无需 Agent 手工另存为 xlsx

### Requirement: Excel 确定性清洗
系统 SHALL 提供 `excel_normalize` 工具，按显式参数将不规则 xlsx / xls 工作表清洗为沙箱内干净 CSV：合并单元格按区域左上角锚点值整矩形填充（`fill_merged`，默认 true）；以 `header_row`（0-based，缺省采用与 `excel_describe` 一致的推测启发式）所在行为表头，之前的行丢弃；列名归一化（去首尾空白与换行符、空名补 `column_{1-based列号}`、重复名追加 `_2`/`_3` 且追加后撞名继续递增）；数据行按表头列数补齐/截断保证 CSV 矩形。返回输出路径、数据行数、最终列名及原名→新名重命名映射。

#### Scenario: 合并单元格填充
- **WHEN** Agent 对含纵向合并区域（值仅在左上角）的 xlsx 调用 `excel_normalize`（`fill_merged=true`）
- **THEN** 输出 CSV 中该区域所有行均填充锚点值

#### Scenario: 表头不在首行
- **WHEN** Agent 指定 `header_row=2` 对前两行为标题/说明的 xlsx 调用 `excel_normalize`
- **THEN** 输出 CSV 以第 2 行（0-based）为表头，第 0、1 行不出现在结果中

#### Scenario: 列名去重与补名
- **WHEN** 表头行含空单元格与两个「完成人」
- **THEN** 输出 CSV 表头为 `column_N` 与 `完成人`、`完成人_2`，且返回的 `renamed` 映射记录了所有改名

#### Scenario: 清洗产物可被 SQL 直接消费
- **WHEN** Agent 将 `excel_normalize` 产出的 CSV 作为 `data_query` 数据源执行聚合查询
- **THEN** 查询成功且数值列经 polars 类型推断可正确参与 `SUM` / `AVG`

#### Scenario: 非法 header_row 明确报错
- **WHEN** Agent 传入超出数据行数的 `header_row`
- **THEN** 返回含实际总行数的明确参数错误，而非 panic 或空文件
