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
    match s.char_indices().nth(max) {
        None => s.to_string(),
        Some((byte_pos, _)) => format!("{}…", &s[..byte_pos]),
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
        if c < 26 {
            break;
        }
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
                "第{header_row}行（推测表头）存在 {} 个空表头（第{empty:?}列），直接 SQL 查询会失败，建议先 excel_normalize",
                empty.len()
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
            warnings.push(format!(
                "列名「{name}」重复 {n} 次，将被自动改名为 {name}_2 等"
            ));
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
