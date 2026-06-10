use super::{ToolContext, ToolError, ToolSpec};
use calamine::{open_workbook, Reader, Xlsx};
use serde_json::{json, Value};
use std::fs::File;
use std::io::BufReader;

pub fn read_tool() -> ToolSpec {
    ToolSpec {
        name: "excel_read",
        description: "Read cells from an Excel workbook sheet",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "sheet": { "type": "string", "description": "Sheet name, default first sheet" }
            },
            "required": ["path"]
        }),
        handler: read_handler,
    }
}

pub fn write_tool() -> ToolSpec {
    ToolSpec {
        name: "excel_write",
        description: "Write cell values to an Excel workbook",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "cells": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "cell": { "type": "string" },
                            "value": {}
                        },
                        "required": ["cell", "value"]
                    }
                }
            },
            "required": ["path", "cells"]
        }),
        handler: write_handler,
    }
}

fn read_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let resolved = ctx.sandbox.resolve(path)?;
    let mut workbook: Xlsx<BufReader<File>> = open_workbook(&resolved)
        .map_err(|e: calamine::XlsxError| ToolError::Execution(e.to_string()))?;
    let sheet_names = workbook.sheet_names().to_owned();
    let sheet = args
        .get("sheet")
        .and_then(|v| v.as_str())
        .or_else(|| sheet_names.first().map(|s| s.as_str()))
        .ok_or_else(|| ToolError::Execution("no sheets".into()))?;
    let range = workbook
        .worksheet_range(sheet)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    let mut rows = Vec::new();
    for row in range.rows() {
        rows.push(row.iter().map(|c| json!(c.to_string())).collect::<Vec<_>>());
    }
    Ok(json!({ "sheet": sheet, "rows": rows }))
}

fn write_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let resolved = ctx.sandbox.resolve_for_write(path)?;
    if let Some(parent) = resolved.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ToolError::Execution(e.to_string()))?;
    }

    let mut book = if resolved.exists() {
        umya_spreadsheet::reader::xlsx::read(&resolved)
            .map_err(|e| ToolError::Execution(e.to_string()))?
    } else {
        umya_spreadsheet::new_file()
    };

    let sheet = book
        .sheet_mut(0)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    let cells = args
        .get("cells")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::InvalidArgs("cells required".into()))?;
    for cell in cells {
        let addr = cell
            .get("cell")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("cell required".into()))?;
        let value = &cell["value"];
        let target = sheet.cell_mut(addr);
        if let Some(n) = value.as_f64() {
            target.set_value_number(n);
        } else if let Some(b) = value.as_bool() {
            target.set_value_bool(b);
        } else {
            target.set_value(value.as_str().unwrap_or_default());
        }
    }
    umya_spreadsheet::writer::xlsx::write(&book, &resolved)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({ "path": resolved.display().to_string() }))
}
