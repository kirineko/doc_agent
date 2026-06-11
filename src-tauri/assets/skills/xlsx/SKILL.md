---
name: xlsx
description: "Use this skill any time a spreadsheet file is the primary input or output. This means any task where the user wants to: open, read, edit, or fix an existing .xlsx, .xlsm, .csv, or .tsv file (e.g., adding columns, computing formulas, formatting, charting, cleaning messy data); create a new spreadsheet from scratch or from other data sources; or convert between tabular file formats. Trigger especially when the user references a spreadsheet file by name or path — even casually (like \"the xlsx in my downloads\") — and wants something done to it or produced from it. Also trigger for cleaning or restructuring messy tabular data files (malformed rows, misplaced headers, junk data) into proper spreadsheets. The deliverable must be a spreadsheet file. Do NOT trigger when the primary deliverable is a Word document, HTML report, standalone Python script, database pipeline, or Google Sheets API integration, even if tabular data is involved."
license: Proprietary. LICENSE.txt has complete terms
---

# XLSX creation, editing, and analysis

> 本系统无 shell/Python/Node 环境，外部表格处理库一律不可用。所有操作通过内置工具完成。

## Quick Reference

| Task | Tool |
|------|------|
| 读取单元格 | `excel_read {"path": "f.xlsx", "sheet": "Sheet1"}`（仅 `.xlsx`） |
| SQL 分析/聚合（CSV/xlsx/**xls**） | `data_query {"sources": [{"name": "t", "path": "f.xls"}], "sql": "SELECT ..."}` — **直接读 `.xls`，不新建文件** |
| 旧格式 `.xls` 阅读为文本 | `office_read_to_markdown {"path": "f.xls"}` — 不新建文件 |
| **仅必要时**转为 `.xlsx` | `office_convert {"path": "f.xls"}` → `f-converted.xlsx`（会丢格式；仅当需要 `excel_write`/`xlsx_recalc`/样式化输出时） |
| 简单写入几个格 | `excel_write {"path": "f.xlsx", "cells": [{"cell": "A1", "value": 100}]}` |
| **生成样式化表格（首选）** | `skill_run` + `ExcelJS`（见下方模板） |
| 公式重算与错误检查（必做） | `xlsx_recalc {"path": "f.xlsx"}` |

## skill_run 生成样式化 Excel（首选）

复杂表格、多 sheet、样式、冻结窗格 → 用 `skill_run`，**不要**用 `excel_write` 逐格写。

```javascript
// ✅ 正确模板（可直接复制）
async function main() {
  const wb = new ExcelJS.Workbook();  // 全局 ExcelJS，require('exceljs') 也可
  const ws = wb.addWorksheet("Sheet1", { views: [{ state: "frozen", ySplit: 1 }] });
  ws.columns = [
    { header: "项目", key: "name", width: 24 },
    { header: "金额", key: "amount", width: 14 },
  ];
  ws.addRow({ name: "营收", amount: 1000 });
  ws.addRow({ name: "合计", amount: { formula: "SUM(B2:B2)" } });  // 公式而非硬编码

  // 样式示例
  ws.getRow(1).font = { bold: true, color: { argb: "FFFFFFFF" } };
  ws.getRow(1).fill = { type: "pattern", pattern: "solid", fgColor: { argb: "FF4472C4" } };
  ws.getCell("B2").numFmt = "$#,##0";

  await wb.xlsx.writeFile("输出.xlsx");  // 已接入沙箱
  return { path: "输出.xlsx" };
}
// ❌ 不要在末尾写 main();
```

**编辑已有文件**：`await wb.xlsx.readFile()` 不可用（无 Node fs stream）。改为：

```javascript
async function main() {
  const buf = fs.readFileSync("existing.xlsx");   // 不带 encoding = 字节
  const wb = new ExcelJS.Workbook();
  await wb.xlsx.load(buf.buffer);                 // load 接受 ArrayBuffer
  const ws = wb.getWorksheet(1);
  ws.getCell("A1").value = "新值";
  await wb.xlsx.writeFile("modified.xlsx");
  return { ok: true };
}
```

生成/修改后**必须**调用 `xlsx_recalc` 校验公式（见下）。

## CRITICAL: Use Formulas, Not Hardcoded Values

**Always use Excel formulas instead of pre-computing values.** This keeps the spreadsheet dynamic.

```javascript
// ❌ WRONG - 把 JS 算好的数硬编码进单元格
ws.getCell("B10").value = rows.reduce((s, r) => s + r.amount, 0);

// ✅ CORRECT - 让 Excel 计算
ws.getCell("B10").value = { formula: "SUM(B2:B9)" };
ws.getCell("C5").value  = { formula: "(C4-C2)/C2" };
ws.getCell("D20").value = { formula: "AVERAGE(D2:D19)" };
```

This applies to ALL calculations — totals, percentages, ratios, differences, etc.

## 公式重算与校验（必做）

```json
xlsx_recalc {"path": "输出.xlsx"}
```

返回 `{ "errors": [...], "warnings": [...] }`：

- `errors` 非空 → 定位 `#REF!`/`#DIV/0!`/`#VALUE!`/`#NAME?` 等并修复后重跑
- 引擎为 IronCalc：**不支持数组公式**，避免生成（`SUMPRODUCT` 单值用法可以）

### Formula Verification Checklist

- [ ] **Test 2-3 sample references** before building the full model
- [ ] **Row offset**: ExcelJS rows are 1-indexed; header 占第 1 行时数据从第 2 行起
- [ ] **Division by zero**: Check denominators (`#DIV/0!`)
- [ ] **Cross-sheet references**: Use `Sheet1!A1` format
- [ ] **Consistent formulas** across all projection periods

## SQL 数据分析

```json
data_query {
  "sources": [{ "name": "sales", "path": "data.xlsx", "sheet": "Q1" }],
  "sql": "SELECT region, SUM(amount) AS total FROM sales GROUP BY region ORDER BY total DESC",
  "out_path": "summary.csv"
}
```

- 引擎 polars-sql；`sources` 可同时挂多个 CSV/xlsx 做 join
- 大数据集聚合优先用它，不要在 JS 里手写循环

---

# Requirements for Outputs

## All Excel files

### Professional Font
- Use a consistent, professional font (e.g., Arial, Times New Roman) for all deliverables unless otherwise instructed by the user

### Zero Formula Errors
- Every Excel model MUST be delivered with ZERO formula errors (#REF!, #DIV/0!, #VALUE!, #N/A, #NAME?)
- 用 `xlsx_recalc` 验证，不要凭感觉

### Preserve Existing Templates (when updating templates)
- Study and EXACTLY match existing format, style, and conventions when modifying files
- Never impose standardized formatting on files with established patterns
- Existing template conventions ALWAYS override these guidelines

## Financial models

### Color Coding Standards
Unless otherwise stated by the user or existing template

#### Industry-Standard Color Conventions
- **Blue text (RGB: 0,0,255)**: Hardcoded inputs, and numbers users will change for scenarios
- **Black text (RGB: 0,0,0)**: ALL formulas and calculations
- **Green text (RGB: 0,128,0)**: Links pulling from other worksheets within same workbook
- **Red text (RGB: 255,0,0)**: External links to other files
- **Yellow background (RGB: 255,255,0)**: Key assumptions needing attention or cells that need to be updated

### Number Formatting Standards

#### Required Format Rules
- **Years**: Format as text strings (e.g., "2024" not "2,024")
- **Currency**: Use $#,##0 format; ALWAYS specify units in headers ("Revenue ($mm)")
- **Zeros**: Use number formatting to make all zeros "-", including percentages (e.g., "$#,##0;($#,##0);-")
- **Percentages**: Default to 0.0% format (one decimal)
- **Multiples**: Format as 0.0x for valuation multiples (EV/EBITDA, P/E)
- **Negative numbers**: Use parentheses (123) not minus -123

### Formula Construction Rules

#### Assumptions Placement
- Place ALL assumptions (growth rates, margins, multiples, etc.) in separate assumption cells
- Use cell references instead of hardcoded values in formulas
- Example: Use `=B5*(1+$B$6)` instead of `=B5*1.05`

#### Documentation Requirements for Hardcodes
- Comment or in cells beside (if end of table). Format: "Source: [System/Document], [Date], [Specific Reference], [URL if applicable]"

---

## doc-agent 系统约束

- **公式重算**：`xlsx_recalc`（IronCalc），不支持数组公式。
- **exceljs dataBar**：条件格式 dataBar 需带完整 cfvo/color 默认值，否则 Excel 可能报错。
- **保存**：只用 `await wb.xlsx.writeFile(path)`（已 shim 接入沙箱）或 `doc_write_bytes(path, await wb.xlsx.writeBuffer())`。
- **CSV**：`await wb.csv.writeFile(path)` 同样可用；读 CSV 优先 `data_query` 或 `fs_read`。
