mod extract_docx;
mod query;
mod recalc;

use super::{required_str_arg, ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};

pub fn extract_docx_tool() -> ToolSpec {
    ToolSpec {
        name: "docx_extract_table",
        description: "Extract tables from a docx file into CSV files in the sandbox",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "out_dir": { "type": "string" },
                "table_index": { "type": "integer" }
            },
            "required": ["path", "out_dir"]
        }),
        handler: extract_docx_handler,
    }
}

pub fn query_tool() -> ToolSpec {
    ToolSpec {
        name: "data_query",
        description: "Run SQL (polars-sql) over sandbox CSV/xlsx/xls sources. For .xls, reads in-memory without creating a converted project file.",
        parameters: json!({
            "type": "object",
            "properties": {
                "sources": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "path": { "type": "string" },
                            "sheet": { "type": "string" }
                        },
                        "required": ["name", "path"]
                    }
                },
                "sql": { "type": "string" },
                "out_path": { "type": "string" }
            },
            "required": ["sources", "sql"]
        }),
        handler: query_handler,
    }
}

pub fn recalc_tool() -> ToolSpec {
    ToolSpec {
        name: "xlsx_recalc",
        description: "Recalculate formulas with IronCalc and report errors",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
            "required": ["path"]
        }),
        handler: recalc_handler,
    }
}

fn extract_docx_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let out_dir = required_str_arg(&args, "out_dir")?;
    let table_index = args
        .get("table_index")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let src = ctx.sandbox.resolve(&path)?;
    let dst = ctx.sandbox.resolve_for_write(&out_dir)?;
    let files = extract_docx::extract_tables(&src, &dst, table_index)?;
    Ok(json!({ "files": files }))
}

fn query_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let sources = args
        .get("sources")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::InvalidArgs("sources required".into()))?;
    let sql = required_str_arg(&args, "sql")?;
    let out_path = args.get("out_path").and_then(|v| v.as_str());
    query::run_query(ctx, sources, &sql, out_path)
}

fn recalc_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let src = ctx.sandbox.resolve(&path)?;
    recalc::recalc(&src)
}

pub(super) fn csv_row<'a, I>(cells: I) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    cells
        .into_iter()
        .map(|cell| format!("\"{}\"", cell.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(",")
}
