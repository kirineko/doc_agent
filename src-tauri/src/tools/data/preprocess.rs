use crate::tools::{ensure_parent_dir, ToolContext, ToolError};
use calamine::{Reader, Xlsx};
use polars::prelude::*;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct RawGrid {
    pub cells: Vec<Vec<String>>,
    pub merged: Vec<MergedRegion>,
    pub sheet: String,
    pub sheets: Vec<String>,
}

pub struct MergedRegion {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

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
    Ok(RawGrid {
        cells,
        merged,
        sheet: sheet_name,
        sheets,
    })
}

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
            let Some(row) = cells.get_mut(r) else {
                continue;
            };
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

pub fn normalize_headers(raw: &[String]) -> (Vec<String>, Vec<(String, String)>) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut allocated: HashSet<String> = HashSet::new();
    let mut out = Vec::with_capacity(raw.len());
    let mut renamed = Vec::new();

    for (i, original) in raw.iter().enumerate() {
        let cleaned = original.replace(['\n', '\r', '\t'], "").trim().to_string();
        let base = if cleaned.is_empty() {
            format!("column_{}", i + 1)
        } else {
            cleaned
        };

        let occurrence = seen.entry(base.clone()).or_insert(0);
        *occurrence += 1;

        let mut name = if *occurrence == 1 {
            base.clone()
        } else {
            format!("{base}_{occurrence}")
        };

        while allocated.contains(&name) {
            *occurrence += 1;
            name = format!("{base}_{occurrence}");
        }

        allocated.insert(name.clone());
        if name != *original {
            renamed.push((original.clone(), name.clone()));
        }
        out.push(name);
    }
    (out, renamed)
}

pub fn suggest_header_row(cells: &[Vec<String>]) -> usize {
    let max_cols = cells.iter().map(Vec::len).max().unwrap_or(0);
    let mut best = (0usize, 0usize);
    for (i, row) in cells.iter().take(10).enumerate() {
        let non_empty = row.iter().filter(|c| !c.trim().is_empty()).count();
        if non_empty < 2 {
            continue;
        }
        let unique = row
            .iter()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .collect::<HashSet<_>>()
            .len();
        let numeric = row
            .iter()
            .filter(|c| {
                let t = c.trim();
                !t.is_empty() && t.parse::<f64>().is_ok()
            })
            .count();
        let score = non_empty + unique;
        let adjusted = score.saturating_sub(numeric * 2);
        if non_empty == max_cols && unique == non_empty && max_cols > 0 && numeric == 0 {
            return i;
        }
        if adjusted > best.0 || (adjusted == best.0 && i < best.1) {
            best = (adjusted, i);
        }
    }
    best.1
}

pub fn cells_to_dataframe(
    cells: &[Vec<String>],
    header_row: usize,
) -> Result<DataFrame, ToolError> {
    if header_row >= cells.len() {
        return Err(ToolError::Execution(format!(
            "header row {header_row} out of range ({} rows)",
            cells.len()
        )));
    }
    let (headers, _) = normalize_headers(&cells[header_row]);
    let data_len = cells.len().saturating_sub(header_row + 1);
    let mut columns: Vec<Vec<String>> = (0..headers.len())
        .map(|_| Vec::with_capacity(data_len))
        .collect();
    for row in &cells[header_row + 1..] {
        for (idx, col) in columns.iter_mut().enumerate() {
            col.push(row.get(idx).map(String::as_str).unwrap_or("").to_string());
        }
    }
    let cols: Vec<Column> = headers
        .into_iter()
        .zip(columns)
        .map(|(name, values)| Column::new(name.into(), values))
        .collect();
    DataFrame::new_infer_height(cols).map_err(|e| ToolError::Execution(e.to_string()))
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
    let file = std::fs::File::create(&out).map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        "{}",
        super::csv_row(headers.iter().map(String::as_str))
    )
    .map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut data_rows = 0usize;
    for row in &grid.cells[header_idx + 1..] {
        let padded: Vec<&str> = (0..headers.len())
            .map(|i| row.get(i).map(String::as_str).unwrap_or(""))
            .collect();
        writeln!(writer, "{}", super::csv_row(padded))
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        data_rows += 1;
    }
    writer
        .flush()
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({
        "path": out.display().to_string(),
        "rows": data_rows,
        "columns": headers,
        "renamed": renamed.into_iter().collect::<HashMap<_, _>>()
    }))
}
