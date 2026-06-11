# Design: add-excel-preprocess

## Context

`data_query` 的 xlsx 加载链路（`tools/data/query.rs::xlsx_to_dataframe`）无脑取 range 第 0 行做表头、全列读为 String。真实业务文件常见：

- 合并单元格（calamine 中仅左上角有值，其余为空串）→ 多个空列名 → polars `DataFrame::new` 报 duplicate column
- 标题/落款行占据首行，真实表头在第 2~3 行
- 同名列（如两个「完成人」）
- SQL 失败时报错为 polars 原始信息，不含实际列名，模型无法自我修正

项目已有相关设施：

- calamine 0.35（支持 `load_merged_regions()`）、umya-spreadsheet 3.0（测试构造合并单元格）
- `office::convert_legacy` 可将 .xls 转临时 xlsx
- `data_query` 原生支持 CSV 数据源（polars CsvReader，自带类型推断）
- `data/mod.rs::csv_row` CSV 行序列化 helper

## Goals / Non-Goals

**Goals:**

- 模型可侦察任意 xlsx/xls 的真实结构（合并区域、原始前 N 行、表头位置建议、结构警告）
- 模型可用确定性参数将不规则表清洗为干净 CSV，交给现有 `data_query` 消费
- `data_query` 直查 xlsx 遇空/重复列名不再失败；SQL 报错携带数据源 schema，可一轮自愈
- 清洗中间产物（CSV）落在沙箱内，模型与用户均可检查

**Non-Goals:**

- 多行表头拼接（`header_rows: [2,3]` → `2024年-指标`）——二期
- 单 sheet 多表的区域（range）提取——二期
- 小计/合计行自动剔除——模型用 SQL `WHERE` 自行排除
- 类型转换/单位换算——CSV 进 polars 后由类型推断与 SQL `CAST` 解决
- 不引入 Python 等新运行时

## Decisions

### D1: 模型当大脑、工具当手（vs 纯启发式自动清洗）

不规则 Excel 的「不规则」无穷尽，纯启发式必有误判且误判后模型仍无信息。因此：

- `excel_describe` **忠实呈现**原始结构（前 N 行原貌 + 合并区域坐标 + 警告），结构判断交给模型
- `excel_normalize` 按显式参数**确定性执行**，可单测、可复现
- 启发式（表头行推测）只作为 describe 的 `suggested_header_row` 建议与 normalize 缺省值，模型永远可覆盖

### D2: 中间产物为沙箱 CSV（vs 内存 DataFrame 直通）

- CSV 可被 `fs_read` 抽查、被用户打开核对，可调试性远好于内存一闪而过
- 清洗一次、N 次 SQL 复用
- `data_query` 的 CSV 路径走 polars 原生 reader，**自带类型推断**，顺带解决 xlsx 直读全列 String 导致 `SUM`/`AVG` 失效的问题
- `data_query` 一行不用改即可消费产物

### D3: 模块划分与代码复用

```
src-tauri/src/tools/data/
├── mod.rs          # +excel_describe / excel_normalize 的 ToolSpec 与 handler（注册入口）
├── preprocess.rs   # 新增：共享纯函数（载入网格+合并区域、填充、列名归一化）+ normalize 执行逻辑
├── describe.rs     # 新增：describe 执行逻辑（预览、表头推测、警告生成）
├── query.rs        # 修改:复用 preprocess::normalize_headers；SQL 报错附 schema
└── ...
```

- 列名归一化逻辑只写一份（`preprocess::normalize_headers`），`excel_normalize` 与 `data_query` 直读 xlsx 共用，避免两处行为漂移
- 注意与 `tools/runtime/normalize.rs`（JS 脚本归一化）无关，模块路径不同不冲突
- 每个文件控制在 300 行内（workspace 规则）

### D4: 合并单元格填充策略——整矩形锚点填充

对每个合并区域，将左上角（锚点）的值填充到区域内**所有**单元格（纵向、横向统一处理），在提取表头**之前**作用于整个网格：

- 纵向合并的数据列（如「材料提供人」跨 5 行）→ 向下填充，行行可查
- 横向合并的表头（如「指标」跨 3 列）→ 3 列同名，再经去重变 `指标`、`指标_2`、`指标_3`，SQL 可用；语义化拼接留给二期多行表头

### D5: 列名归一化规则（确定性、有序）

1. 去首尾空白；单元格内换行/制表符替换为空串（中文表头常见 `完成\n人`）
2. 空列名 → `column_{N}`（N 为 1-based 列号，贴近 Excel 习惯）
3. 重复列名 → 第 2 次起追加 `_2`、`_3`…（追加后若仍撞名继续递增）
4. 返回 `renamed` 映射（原名 → 新名）供工具结果回显，模型知道发生了什么

### D6: data_query 报错自愈信息

注册数据源时记录 `name → Vec<列名>`；SQL 执行失败时在错误信息后追加：

```
可用表结构: 指标: ["指标点", "材料提供人", "完成人", "column_4", ...]
```

模型拿到真实列名即可一轮改写 SQL，消灭你那次踩坑中「盲猜列名反复试探」的根因。

### D7: 工具引导（prompt 层）

`data_query` 描述追加一句：「Excel 结构不规则（合并单元格/表头不在首行/列名报错）时，先 `excel_describe` 侦察、`excel_normalize` 清洗为 CSV 再查询」。三件套形成管线：

```
脏 xlsx ──▶ excel_describe ──▶ excel_normalize ──▶ 干净 CSV ──▶ data_query(SQL)
            (结构侦察,模型读)    (模型定参,确定性执行)              (现有能力)
```

## 工具接口定义

### excel_describe

```json
{
  "name": "excel_describe",
  "description": "Inspect the raw structure of an xlsx/xls sheet: dimensions, merged regions, first rows preview, suggested header row, and structural warnings (empty/duplicate headers). Use BEFORE excel_normalize / data_query on irregular files.",
  "parameters": {
    "type": "object",
    "properties": {
      "path": { "type": "string" },
      "sheet": { "type": "string", "description": "Sheet name, default first sheet" },
      "preview_rows": { "type": "integer", "default": 15, "maximum": 50 }
    },
    "required": ["path"]
  }
}
```

返回示例：

```json
{
  "sheets": ["指标", "Sheet2"],
  "sheet": "指标",
  "rows": 122, "cols": 8,
  "merged_regions": [
    { "range": "A2:A6", "anchor": "1.1 毕业要求" },
    { "range": "C1:E1", "anchor": "支撑材料" }
  ],
  "preview": [["软件工程专业评估指标点", "", "", ""], ["指标点", "材料提供人", "完成人", ""]],
  "suggested_header_row": 1,
  "warnings": [
    "第1行存在 1 个空表头（第4列）",
    "存在合并单元格 2 处，建议 excel_normalize 填充"
  ]
}
```

实现要点：

- preview 单元格值截断至 80 字符、列数截断至 30，控制 token
- `suggested_header_row` 启发式：取前 10 行中「非空格数 × 唯一性」得分最高的行（见参考实现），仅为建议

### excel_normalize

```json
{
  "name": "excel_normalize",
  "description": "Clean an irregular xlsx/xls sheet into a tidy CSV in the sandbox: fill merged cells with anchor value, take header from header_row, dedupe/auto-name columns. Output CSV is directly consumable by data_query.",
  "parameters": {
    "type": "object",
    "properties": {
      "path": { "type": "string" },
      "sheet": { "type": "string" },
      "header_row": { "type": "integer", "description": "0-based header row index; default = describe suggestion heuristic" },
      "fill_merged": { "type": "boolean", "default": true },
      "out_path": { "type": "string", "description": "Output CSV path in sandbox, e.g. normalized/指标.csv" }
    },
    "required": ["path", "out_path"]
  }
}
```

返回示例：

```json
{
  "path": ".../normalized/指标.csv",
  "rows": 120,
  "columns": ["指标点", "材料提供人", "完成人", "column_4"],
  "renamed": { "": "column_4", "完成人(2)": "完成人_2" }
}
```

语义：`header_row` 之前的行全部丢弃，之后的行为数据行；合并填充先于表头提取作用于全网格。

## 参考实现

> 以下为实现基准；与实际 crate API 出入时（calamine `merged_regions` 签名需在实现时核实），以「先改本文档、再改代码」为准。

### preprocess.rs — 共享纯函数与 normalize

```rust
use crate::tools::{ensure_parent_dir, ToolContext, ToolError};
use calamine::{Reader, Xlsx};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

/// 原始网格 + 合并区域（行列均为相对 range 起点的 0-based 坐标）
pub struct RawGrid {
    pub cells: Vec<Vec<String>>,
    pub merged: Vec<MergedRegion>, // 已换算为 grid 坐标
    pub sheet: String,
    pub sheets: Vec<String>,
}

pub struct MergedRegion {
    pub start: (usize, usize), // (row, col)
    pub end: (usize, usize),
}

/// 打开 xlsx/xls（xls 经 convert_legacy 转临时 xlsx），读出网格与合并区域。
/// 注意 calamine 的 worksheet_range 有起点偏移（range.start()），
/// merged_regions 返回的是绝对坐标，须减去偏移换算成 grid 坐标。
pub fn load_grid(path: &Path, sheet: Option<&str>) -> Result<RawGrid, ToolError> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("xls") => {
            let temp = tempfile::Builder::new()
                .suffix(".xlsx")
                .tempfile()
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            crate::tools::office::convert_legacy(path, temp.path())?;
            load_xlsx_grid(temp.path(), sheet)
        }
        Some("xlsx") | Some("xlsm") => load_xlsx_grid(path, sheet),
        other => Err(ToolError::InvalidArgs(format!(
            "unsupported excel type: {other:?}"
        ))),
    }
}

fn load_xlsx_grid(path: &Path, sheet: Option<&str>) -> Result<RawGrid, ToolError> {
    let mut wb: Xlsx<_> = calamine::open_workbook(path)
        .map_err(|e| ToolError::Execution(format!("xlsx open: {e}")))?;
    wb.load_merged_regions()
        .map_err(|e| ToolError::Execution(format!("merged regions: {e}")))?;
    let sheets = wb.sheet_names().to_owned();
    let sheet_name = sheet
        .map(str::to_string)
        .or_else(|| sheets.first().cloned())
        .ok_or_else(|| ToolError::Execution("xlsx has no sheets".into()))?;
    let range = wb
        .worksheet_range(&sheet_name)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    let (off_r, off_c) = range.start().unwrap_or((0, 0));
    let cells: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(|c| c.to_string()).collect())
        .collect();
    let merged = wb
        .merged_regions_by_sheet(&sheet_name)
        .iter()
        .map(|(_, _, dim)| MergedRegion {
            start: (
                dim.start.0.saturating_sub(off_r) as usize,
                dim.start.1.saturating_sub(off_c) as usize,
            ),
            end: (
                dim.end.0.saturating_sub(off_r) as usize,
                dim.end.1.saturating_sub(off_c) as usize,
            ),
        })
        .collect();
    Ok(RawGrid { cells, merged, sheet: sheet_name, sheets })
}

/// 合并区域整矩形锚点填充（原地）。锚点 = 区域左上角的值。
pub fn fill_merged(cells: &mut [Vec<String>], merged: &[MergedRegion]) {
    for region in merged {
        let anchor = cells
            .get(region.start.0)
            .and_then(|r| r.get(region.start.1))
            .cloned()
            .unwrap_or_default();
        if anchor.is_empty() {
            continue;
        }
        for r in region.start.0..=region.end.0 {
            let Some(row) = cells.get_mut(r) else { continue };
            for c in region.start.1..=region.end.1 {
                if let Some(cell) = row.get_mut(c) {
                    if cell.is_empty() {
                        *cell = anchor.clone();
                    }
                }
            }
        }
    }
}

/// 列名归一化：trim + 去换行 → 空名补 column_N → 重复名加 _2/_3。
/// 返回 (归一化列名, 原名→新名 的重命名记录)。
pub fn normalize_headers(raw: &[String]) -> (Vec<String>, Vec<(String, String)>) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::with_capacity(raw.len());
    let mut renamed = Vec::new();
    for (i, original) in raw.iter().enumerate() {
        let cleaned = original
            .replace(['\n', '\r', '\t'], "")
            .trim()
            .to_string();
        let base = if cleaned.is_empty() {
            format!("column_{}", i + 1)
        } else {
            cleaned
        };
        let count = seen.entry(base.clone()).or_insert(0);
        *count += 1;
        let mut name = if *count == 1 {
            base.clone()
        } else {
            format!("{base}_{count}")
        };
        // 追加后缀后仍撞名（如原表就有「完成人_2」列）则继续递增
        while *count > 1 && seen.contains_key(&name) {
            *count += 1;
            name = format!("{base}_{count}");
        }
        if *count > 1 {
            seen.insert(name.clone(), 1);
        }
        if &name != original {
            renamed.push((original.clone(), name.clone()));
        }
        out.push(name);
    }
    (out, renamed)
}

/// 表头行启发式：前 10 行中「非空格数 + 唯一值数」得分最高者。
pub fn suggest_header_row(cells: &[Vec<String>]) -> usize {
    let max_cols = cells.iter().map(Vec::len).max().unwrap_or(0);
    let mut best = (0usize, 0usize); // (score, row)
    for (i, row) in cells.iter().take(10).enumerate() {
        let non_empty = row.iter().filter(|c| !c.trim().is_empty()).count();
        let unique = row
            .iter()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .collect::<std::collections::HashSet<_>>()
            .len();
        let score = non_empty + unique;
        // 满分行直接选：非空铺满且全唯一
        if non_empty == max_cols && unique == non_empty {
            return i;
        }
        if score > best.0 {
            best = (score, i);
        }
    }
    best.1
}

pub fn run_normalize(
    ctx: &ToolContext,
    path: &str,
    sheet: Option<&str>,
    header_row: Option<usize>,
    fill: bool,
    out_path: &str,
) -> Result<Value, ToolError> {
    let resolved = ctx.sandbox.resolve(path).map_err(ToolError::Sandbox)?;
    let mut grid = load_grid(&resolved, sheet)?;
    if fill {
        fill_merged(&mut grid.cells, &grid.merged);
    }
    let header_idx = header_row.unwrap_or_else(|| suggest_header_row(&grid.cells));
    if header_idx >= grid.cells.len() {
        return Err(ToolError::InvalidArgs(format!(
            "header_row {header_idx} 超出范围（共 {} 行）",
            grid.cells.len()
        )));
    }
    let (headers, renamed) = normalize_headers(&grid.cells[header_idx]);
    let out = ctx
        .sandbox
        .resolve_for_write(out_path)
        .map_err(ToolError::Sandbox)?;
    ensure_parent_dir(&out)?;
    let mut lines = Vec::with_capacity(grid.cells.len() - header_idx);
    lines.push(super::csv_row(headers.iter().map(String::as_str)));
    let mut data_rows = 0usize;
    for row in &grid.cells[header_idx + 1..] {
        // 补齐/截断到表头列数，保证 CSV 矩形
        let padded: Vec<&str> = (0..headers.len())
            .map(|i| row.get(i).map(String::as_str).unwrap_or(""))
            .collect();
        lines.push(super::csv_row(padded));
        data_rows += 1;
    }
    std::fs::write(&out, lines.join("\n"))
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({
        "path": out.display().to_string(),
        "rows": data_rows,
        "columns": headers,
        "renamed": renamed.into_iter().collect::<HashMap<_, _>>()
    }))
}
```

### describe.rs — 结构侦察

```rust
use super::preprocess::{load_grid, suggest_header_row, RawGrid};
use crate::tools::{ToolContext, ToolError};
use serde_json::{json, Value};

const CELL_TRUNC: usize = 80;
const COL_TRUNC: usize = 30;

pub fn run_describe(
    ctx: &ToolContext,
    path: &str,
    sheet: Option<&str>,
    preview_rows: usize,
) -> Result<Value, ToolError> {
    let resolved = ctx.sandbox.resolve(path).map_err(ToolError::Sandbox)?;
    let grid = load_grid(&resolved, sheet)?;
    let rows = grid.cells.len();
    let cols = grid.cells.iter().map(Vec::len).max().unwrap_or(0);
    let suggested = suggest_header_row(&grid.cells);
    let preview: Vec<Vec<String>> = grid
        .cells
        .iter()
        .take(preview_rows)
        .map(|row| {
            row.iter()
                .take(COL_TRUNC)
                .map(|c| truncate(c, CELL_TRUNC))
                .collect()
        })
        .collect();
    Ok(json!({
        "sheets": grid.sheets,
        "sheet": grid.sheet,
        "rows": rows,
        "cols": cols,
        "merged_regions": merged_json(&grid),
        "preview": preview,
        "suggested_header_row": suggested,
        "warnings": build_warnings(&grid, suggested),
    }))
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

fn merged_json(grid: &RawGrid) -> Vec<Value> {
    grid.merged
        .iter()
        .map(|m| {
            let anchor = grid
                .cells
                .get(m.start.0)
                .and_then(|r| r.get(m.start.1))
                .cloned()
                .unwrap_or_default();
            json!({
                "range": format!("{}:{}", a1(m.start), a1(m.end)),
                "anchor": truncate(&anchor, CELL_TRUNC),
            })
        })
        .collect()
}

fn a1((row, col): (usize, usize)) -> String {
    let mut name = String::new();
    let mut c = col;
    loop {
        name.insert(0, (b'A' + (c % 26) as u8) as char);
        if c < 26 { break; }
        c = c / 26 - 1;
    }
    format!("{name}{}", row + 1)
}

fn build_warnings(grid: &RawGrid, header_row: usize) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(header) = grid.cells.get(header_row) {
        let empty: Vec<usize> = header
            .iter()
            .enumerate()
            .filter(|(_, c)| c.trim().is_empty())
            .map(|(i, _)| i + 1)
            .collect();
        if !empty.is_empty() {
            warnings.push(format!(
                "第{header_row}行（推测表头）存在 {} 个空表头（第{:?}列），直接 SQL 查询会失败，建议先 excel_normalize",
                empty.len(), empty
            ));
        }
        let mut seen = std::collections::HashMap::new();
        for c in header {
            let t = c.trim();
            if !t.is_empty() {
                *seen.entry(t).or_insert(0) += 1;
            }
        }
        for (name, n) in seen.iter().filter(|(_, n)| **n > 1) {
            warnings.push(format!("列名「{name}」重复 {n} 次，将被自动改名为 {name}_2 等"));
        }
    }
    if !grid.merged.is_empty() {
        warnings.push(format!(
            "存在合并单元格 {} 处，建议 excel_normalize（fill_merged=true）填充后再分析",
            grid.merged.len()
        ));
    }
    if header_row > 0 {
        warnings.push(format!(
            "推测表头不在首行（第{header_row}行），data_query 直查会把第0行当表头"
        ));
    }
    warnings
}
```

### query.rs 修改点

1. `xlsx_to_dataframe` 中表头经 `preprocess::normalize_headers` 归一化（消灭 duplicate column 失败）：

```rust
// xlsx_to_dataframe 内，i == 0 分支替换：
let (normalized, _renamed) = super::preprocess::normalize_headers(&values);
headers = normalized;
```

2. `run_query` 注册数据源时记录 schema，SQL 失败时附带：

```rust
let mut schemas: Vec<(String, Vec<String>)> = Vec::new();
for source in sources {
    // ...原有 resolve / load_source...
    schemas.push((
        name.to_string(),
        df.get_column_names().iter().map(|s| s.to_string()).collect(),
    ));
    sql_ctx.register(name, df.lazy());
}
let result = sql_ctx
    .execute(sql)
    .and_then(|lf| lf.collect())
    .map_err(|e| {
        let hint = schemas
            .iter()
            .map(|(n, cols)| format!("{n}: {cols:?}"))
            .collect::<Vec<_>>()
            .join("; ");
        ToolError::Execution(format!("{e}\n可用表结构: {hint}\n提示: 结构不规则的 Excel 请先用 excel_describe / excel_normalize 清洗为 CSV 再查询"))
    })?;
```

### mod.rs 注册（节选）

```rust
pub fn describe_tool() -> ToolSpec { /* schema 见上文「工具接口定义」 */ }
pub fn normalize_tool() -> ToolSpec { /* 同上 */ }

fn describe_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let sheet = args.get("sheet").and_then(|v| v.as_str());
    let preview = args
        .get("preview_rows")
        .and_then(|v| v.as_u64())
        .unwrap_or(15)
        .clamp(1, 50) as usize;
    describe::run_describe(ctx, &path, sheet, preview)
}

fn normalize_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let out_path = required_str_arg(&args, "out_path")?;
    let sheet = args.get("sheet").and_then(|v| v.as_str());
    let header_row = args.get("header_row").and_then(|v| v.as_u64()).map(|v| v as usize);
    let fill = args.get("fill_merged").and_then(|v| v.as_bool()).unwrap_or(true);
    preprocess::run_normalize(ctx, &path, sheet, header_row, fill, &out_path)
}
```

`registry.rs::default_tools()` 追加 `crate::tools::data::describe_tool()` 与 `crate::tools::data::normalize_tool()`。

## 测试设计

| 测试 | 内容 |
|---|---|
| `normalize_headers_*`（单元） | 空名补 `column_N`；重复加 `_2`；trim/去换行；后缀撞名继续递增 |
| `fill_merged_*`（单元） | 纵向/横向/矩形区域锚点填充；锚点为空不填充 |
| `suggest_header_row_*`（单元） | 首行为标题行时建议第 1 行；规整表建议第 0 行 |
| `describe_reports_structure`（集成） | umya 构造含合并单元格（`add_merge_cells("A2:A4")`）+ 标题行的 xlsx → describe 返回 merged_regions、warnings、suggested_header_row |
| `normalize_produces_tidy_csv`（集成） | 同上文件 → normalize → CSV 表头无空/重复名、合并值已填充、行数正确 |
| `query_messy_xlsx_no_dup_error`（集成） | 含空/重复表头 xlsx 直接 data_query `SELECT *` → 不再报 duplicate column |
| `query_error_contains_schema`（集成） | SQL 引用不存在的列 → 错误信息含「可用表结构」与真实列名 |

## Risks / Trade-offs

- [calamine `merged_regions` API 签名与版本差异] → 实现第一步即写 API 验证测试；若 0.35 行为不符，本文档 `load_xlsx_grid` 一节先行更新
- [横向合并表头去重后语义弱（`指标_2`）] → describe 的 warnings 已提示模型；语义化拼接留二期多行表头
- [启发式表头推测误判] → 仅为建议值；describe 同时给出原始 preview，模型可显式传 `header_row` 覆盖
- [normalize 全量载入内存，超大文件（10万行+）占用高] → 与现有 `data_query` xlsx 路径同量级，不恶化现状；超大文件场景留观察
- [CSV 中间文件残留沙箱] → 视为特性（可调试、可复用）；输出建议归口 `normalized/` 目录

## Open Questions

- `merged_regions_by_sheet` 在 0.35 的确切返回结构（`(String, String, Dimensions)` 元组字段含义）需实现时以编译器为准核实
- 是否需要在 describe 中返回每列的「疑似类型」（数值/日期/文本占比）辅助模型写 CAST——倾向二期再说，先控制返回体积
