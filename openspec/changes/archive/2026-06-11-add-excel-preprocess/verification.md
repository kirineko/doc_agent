# 手工验收备注

## 真实问题文件（任务 5.3）

仓库内未找到 `报告/3.软件工程专业评估方案指标点-1017-确定版.xlsx`，无法在本 change 中完成端到端手工验收。

## 替代验证

已通过 umya-spreadsheet 构造的「不规则 xlsx」集成测试覆盖同等场景：

- `excel_describe_messy_xlsx`：合并区域、空/重复表头警告、表头行推测
- `excel_normalize_messy_xlsx`：合并填充、列名去重、CSV 产出
- `data_query_messy_xlsx_no_dup_error`：直查 xlsx 不再 duplicate column
- `normalize_csv_sum_query`：清洗后 CSV 可被 SQL 聚合

建议在项目沙箱中放入真实文件后，手动执行：

```text
excel_describe → excel_normalize(header_row=推测值) → data_query
```
