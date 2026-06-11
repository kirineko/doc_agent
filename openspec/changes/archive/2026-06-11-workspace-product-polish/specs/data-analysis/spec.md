## MODIFIED Requirements

### Requirement: SQL 数据整理
系统 SHALL 提供 `data_query` 工具，对沙箱内 CSV / xlsx / **xls** 数据源执行 SQL（polars-sql），支持过滤、聚合、排序、UNION 等整理操作，结果写回沙箱内 CSV 或直接返回（小结果集）。

#### Scenario: 聚合查询
- **WHEN** Agent 以某 CSV 为源执行 `SELECT category, SUM(amount) FROM t GROUP BY category`
- **THEN** 返回聚合结果，数值类型正确

#### Scenario: xlsx 作为数据源
- **WHEN** Agent 以某 `.xlsx` 的指定工作表为源执行查询
- **THEN** 系统经 calamine 读取并完成查询，无需先手工转 CSV

#### Scenario: xls 作为数据源
- **WHEN** Agent 以某 `.xls` 的指定工作表为源执行查询
- **THEN** 系统读取该旧格式表格并完成查询，无需 Agent 事先手动另存为 xlsx
