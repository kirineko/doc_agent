# data-analysis Specification (Delta)

## MODIFIED Requirements

### Requirement: SQL 数据整理
系统 SHALL 提供 `data_query` 工具，对沙箱内 CSV / xlsx / **xls** 数据源执行 SQL（polars-sql），支持过滤、聚合、排序、UNION 等整理操作，结果写回沙箱内 CSV 或直接返回（小结果集）。

xlsx / xls 数据源构建 DataFrame 时，表头 SHALL 经列名归一化（与 `excel_normalize` 同一套规则：去空白与换行、空名补 `column_N`、重复名追加 `_2`/`_3`），空/重复列名 MUST NOT 导致查询直接失败。

SQL 执行失败时，错误信息 SHALL 附带各已注册数据源的实际列名清单，以及「不规则 Excel 建议先经 excel_describe / excel_normalize 清洗」的提示，使 Agent 可凭单次报错自我修正。

#### Scenario: 聚合查询
- **WHEN** Agent 以某 CSV 为源执行 `SELECT category, SUM(amount) FROM t GROUP BY category`
- **THEN** 返回聚合结果，数值类型正确

#### Scenario: xlsx 作为数据源
- **WHEN** Agent 以某 `.xlsx` 的指定工作表为源执行查询
- **THEN** 系统经 calamine 读取并完成查询，无需先手工转 CSV

#### Scenario: xls 作为数据源
- **WHEN** Agent 以某 `.xls` 的指定工作表为源执行查询
- **THEN** 系统读取该旧格式表格并完成查询，无需 Agent 事先手动另存为 xlsx

#### Scenario: 空/重复表头的 xlsx 不再失败
- **WHEN** Agent 以表头含空单元格与重复列名的 xlsx 为源执行 `SELECT *`
- **THEN** 查询成功，列名按归一化规则呈现（`column_N`、`完成人_2` 等），不报 duplicate column 错误

#### Scenario: SQL 报错携带 schema 自愈信息
- **WHEN** Agent 执行的 SQL 引用了不存在的列名
- **THEN** 错误信息包含「可用表结构」与各数据源真实列名清单，及预处理工具的使用提示
