mod describe;
mod extract_docx;
pub(crate) mod preprocess;
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

pub fn describe_tool() -> ToolSpec {
    ToolSpec {
        name: "excel_describe",
        description: "Inspect the raw structure of an xlsx/xls sheet: dimensions, merged regions, first rows preview, suggested header row, and structural warnings (empty/duplicate headers). Use BEFORE excel_normalize / data_query on irregular files.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "sheet": { "type": "string", "description": "Sheet name, default first sheet" },
                "preview_rows": { "type": "integer", "default": 15, "maximum": 50 }
            },
            "required": ["path"]
        }),
        handler: describe_handler,
    }
}

pub fn normalize_tool() -> ToolSpec {
    ToolSpec {
        name: "excel_normalize",
        description: "Clean an irregular xlsx/xls sheet into a tidy CSV in the sandbox: fill merged cells with anchor value, take header from header_row, dedupe/auto-name columns. Output CSV is directly consumable by data_query.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "sheet": { "type": "string" },
                "header_row": { "type": "integer", "description": "0-based header row index; default = describe suggestion heuristic" },
                "fill_merged": { "type": "boolean", "default": true },
                "out_path": { "type": "string", "description": "Output CSV path in sandbox, e.g. normalized/指标.csv" }
            },
            "required": ["path", "out_path"]
        }),
        handler: normalize_handler,
    }
}

pub fn query_tool() -> ToolSpec {
    ToolSpec {
        name: "data_query",
        description: "Run SQL (polars-sql) over sandbox CSV/xlsx/xls sources. For .xls, reads in-memory without creating a converted project file. For irregular Excel (merged cells, header not on row 0, column name errors), use excel_describe then excel_normalize to produce a clean CSV first.",
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

fn describe_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let sheet = args.get("sheet").and_then(|v| v.as_str());
    let preview_rows = args
        .get("preview_rows")
        .and_then(|v| v.as_u64())
        .unwrap_or(15)
        .clamp(1, 50) as usize;
    describe::run_describe(ctx, &path, sheet, preview_rows)
}

fn normalize_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let out_path = required_str_arg(&args, "out_path")?;
    let sheet = args.get("sheet").and_then(|v| v.as_str());
    let header_row = args
        .get("header_row")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let fill = args
        .get("fill_merged")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    preprocess::run_normalize(ctx, &path, sheet, header_row, fill, &out_path)
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
    let mut out = String::new();
    let mut first = true;
    for cell in cells {
        if !first {
            out.push(',');
        }
        first = false;
        out.push('"');
        for ch in cell.chars() {
            if ch == '"' {
                out.push_str("\"\"");
            } else {
                out.push(ch);
            }
        }
        out.push('"');
    }
    out
}
