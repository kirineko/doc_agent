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
    let mut schemas: Vec<(String, Vec<String>)> = Vec::new();
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
        schemas.push((
            name.to_string(),
            df.get_column_names()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ));
        sql_ctx.register(name, df.lazy());
    }
    let mut result = sql_ctx
        .execute(sql)
        .map_err(|e| schema_query_error(&schemas, e))?
        .collect()
        .map_err(|e| schema_query_error(&schemas, e))?;
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
        .finish(&mut result)
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
        Some("xlsx") | Some("xlsm") | Some("xls") => {
            let grid = super::preprocess::load_grid(path, sheet)?;
            super::preprocess::cells_to_dataframe(&grid.cells, 0)
        }
        other => Err(ToolError::InvalidArgs(format!(
            "unsupported source type: {other:?}"
        ))),
    }
}

fn schema_query_error(schemas: &[(String, Vec<String>)], err: impl std::fmt::Display) -> ToolError {
    let hint = schemas
        .iter()
        .map(|(n, cols)| format!("{n}: {cols:?}"))
        .collect::<Vec<_>>()
        .join("; ");
    ToolError::Execution(format!(
        "{err}\n可用表结构: {hint}\n提示: 结构不规则的 Excel 请先用 excel_describe / excel_normalize 清洗为 CSV 再查询"
    ))
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
