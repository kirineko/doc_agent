use crate::tools::{ensure_parent_dir, ToolContext, ToolError};
use polars::prelude::*;
use polars::sql::SQLContext;
use serde_json::{json, Value};
use std::path::Path;

pub fn run_query(
    ctx: &ToolContext,
    sources: &[Value],
    sql: &str,
    out_path: Option<&str>,
) -> Result<Value, ToolError> {
    let mut sql_ctx = SQLContext::new();
    for source in sources {
        let name = source
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("source.name required".into()))?;
        let path = source
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("source.path required".into()))?;
        let resolved = resolve_existing(ctx, path)?;
        let df = load_source(&resolved, source.get("sheet").and_then(|v| v.as_str()))?;
        sql_ctx.register(name, df.lazy());
    }
    let result = sql_ctx
        .execute(sql)
        .map_err(|e| ToolError::Execution(e.to_string()))?
        .collect()
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    let rows = result.height();
    if rows <= 50 && out_path.is_none() {
        return Ok(json!({
            "rows": rows,
            "data": dataframe_to_json(&result)?
        }));
    }
    let out = if let Some(p) = out_path {
        ctx.sandbox
            .resolve_for_write(p)
            .map_err(ToolError::Sandbox)?
    } else {
        ctx.sandbox
            .resolve_for_write("query_result.csv")
            .map_err(ToolError::Sandbox)?
    };
    ensure_parent_dir(&out)?;
    let mut file = std::fs::File::create(&out).map_err(|e| ToolError::Execution(e.to_string()))?;
    CsvWriter::new(&mut file)
        .finish(&mut result.clone())
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({ "rows": rows, "path": out.display().to_string() }))
}

fn resolve_existing(ctx: &ToolContext, path: &str) -> Result<std::path::PathBuf, ToolError> {
    ctx.sandbox
        .resolve(path)
        .or_else(|_| {
            let candidate = ctx.sandbox.resolve_for_write(path)?;
            if candidate.exists() {
                Ok(candidate)
            } else {
                Err(crate::core::sandbox::SandboxError::NotFound)
            }
        })
        .map_err(ToolError::Sandbox)
}

fn load_source(path: &Path, sheet: Option<&str>) -> Result<DataFrame, ToolError> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("csv") => CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(path.to_path_buf()))
            .map_err(|e| ToolError::Execution(e.to_string()))?
            .finish()
            .map_err(|e| ToolError::Execution(e.to_string())),
        Some("xlsx") | Some("xlsm") => xlsx_to_dataframe(path, sheet),
        Some("xls") => xls_to_dataframe(path, sheet),
        other => Err(ToolError::InvalidArgs(format!(
            "unsupported source type: {other:?}"
        ))),
    }
}

fn xls_to_dataframe(path: &Path, sheet: Option<&str>) -> Result<DataFrame, ToolError> {
    let temp = tempfile::Builder::new()
        .suffix(".xlsx")
        .tempfile()
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    crate::tools::office::convert_legacy(path, temp.path())?;
    xlsx_to_dataframe(temp.path(), sheet)
}

fn xlsx_to_dataframe(path: &Path, sheet: Option<&str>) -> Result<DataFrame, ToolError> {
    use calamine::{Reader, Xlsx};
    let mut workbook: Xlsx<_> = calamine::open_workbook(path)
        .map_err(|e| ToolError::Execution(format!("xlsx open: {e}")))?;
    let sheet_name = sheet
        .map(str::to_string)
        .or_else(|| workbook.sheet_names().first().cloned())
        .ok_or_else(|| ToolError::Execution("xlsx has no sheets".into()))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut headers: Vec<String> = Vec::new();
    let mut columns: Vec<Vec<String>> = Vec::new();
    for (i, row) in range.rows().enumerate() {
        let values: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        if i == 0 {
            headers = values;
            columns = headers.iter().map(|_| Vec::new()).collect();
            continue;
        }
        for (idx, v) in values.into_iter().enumerate() {
            if let Some(col) = columns.get_mut(idx) {
                col.push(v);
            }
        }
    }
    let cols: Vec<Column> = headers
        .into_iter()
        .enumerate()
        .map(|(i, name)| Column::new(name.into(), columns.get(i).cloned().unwrap_or_default()))
        .collect();
    DataFrame::new_infer_height(cols).map_err(|e| ToolError::Execution(e.to_string()))
}

fn dataframe_to_json(df: &DataFrame) -> Result<Value, ToolError> {
    let height = df.height();
    let columns = df
        .get_column_names()
        .into_iter()
        .map(|name| {
            df.column(name)
                .map(|series| (name.to_string(), series.clone()))
                .map_err(|e| ToolError::Execution(e.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::new();
    for i in 0..height {
        let mut obj = serde_json::Map::new();
        for (name, series) in &columns {
            let value = series
                .get(i)
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            obj.insert(name.clone(), json!(format!("{value:?}")));
        }
        rows.push(Value::Object(obj));
    }
    Ok(Value::Array(rows))
}
