mod merge;
mod pages;

use super::{ensure_parent_dir, required_str_arg, ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};

pub fn merge_tool() -> ToolSpec {
    ToolSpec {
        name: "pdf_merge",
        description: "Merge multiple PDFs in order into one file (lopdf)",
        parameters: json!({
            "type": "object",
            "properties": {
                "inputs": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "out_path": { "type": "string" }
            },
            "required": ["inputs", "out_path"]
        }),
        handler: merge_handler,
    }
}

pub fn split_tool() -> ToolSpec {
    ToolSpec {
        name: "pdf_split",
        description: "Split a PDF by page ranges or burst into single-page files",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "ranges": { "type": "string", "description": "e.g. \"1-3,5\" (1-based)" },
                "mode": { "type": "string", "enum": ["burst"] },
                "out_path": { "type": "string" },
                "out_dir": { "type": "string" }
            },
            "required": ["path"]
        }),
        handler: split_handler,
    }
}

pub fn rotate_tool() -> ToolSpec {
    ToolSpec {
        name: "pdf_rotate",
        description: "Rotate all or selected PDF pages by 90/180/270 degrees",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "rotation": { "type": "integer", "description": "90, 180, or 270" },
                "pages": {
                    "type": "array",
                    "items": { "type": "integer" }
                },
                "mode": { "type": "string", "enum": ["absolute", "relative"] },
                "out_path": { "type": "string" }
            },
            "required": ["path", "rotation", "out_path"]
        }),
        handler: rotate_handler,
    }
}

pub fn delete_pages_tool() -> ToolSpec {
    ToolSpec {
        name: "pdf_delete_pages",
        description: "Delete selected pages from a PDF (1-based page numbers)",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "pages": {
                    "type": "array",
                    "items": { "type": "integer" }
                },
                "out_path": { "type": "string" }
            },
            "required": ["path", "pages", "out_path"]
        }),
        handler: delete_pages_handler,
    }
}

fn merge_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let inputs = args
        .get("inputs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::InvalidArgs("inputs required".into()))?;
    if inputs.is_empty() {
        return Err(ToolError::InvalidArgs("至少需要一个输入 PDF".into()));
    }
    let out_path = required_str_arg(&args, "out_path")?;
    let mut paths = Vec::with_capacity(inputs.len());
    for value in inputs {
        let rel = value
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("inputs 须为字符串数组".into()))?;
        paths.push(ctx.sandbox.resolve(rel)?);
    }
    let out = ctx.sandbox.resolve_for_write(&out_path)?;
    ensure_parent_dir(&out)?;
    let refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
    let pages = merge::merge_pdfs(&refs, &out)?;
    Ok(json!({ "path": out.display().to_string(), "pages": pages }))
}

fn split_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let src = ctx.sandbox.resolve(&path)?;
    let mode = args.get("mode").and_then(|v| v.as_str()).unwrap_or("range");

    if mode == "burst" {
        let out_dir = required_str_arg(&args, "out_dir")?;
        let dst = ctx.sandbox.resolve_for_write(&out_dir)?;
        let files = pages::split_burst(&src, &dst)?;
        return Ok(json!({ "files": files }));
    }

    let ranges = required_str_arg(&args, "ranges")?;
    let out_path = required_str_arg(&args, "out_path")?;
    let out = ctx.sandbox.resolve_for_write(&out_path)?;
    let count = pages::split_by_ranges(&src, &ranges, &out)?;
    Ok(json!({ "path": out.display().to_string(), "pages": count }))
}

fn rotate_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let out_path = required_str_arg(&args, "out_path")?;
    let rotation =
        args.get("rotation")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ToolError::InvalidArgs("rotation required".into()))? as i32;
    let relative = matches!(args.get("mode").and_then(|v| v.as_str()), Some("relative"));
    let page_list = match args.get("pages").and_then(|v| v.as_array()) {
        Some(values) => {
            let src = ctx.sandbox.resolve(&path)?;
            let probe = lopdf::Document::load(&src)
                .map_err(|e| ToolError::Execution(format!("load {}: {e}", src.display())))?;
            let total = probe.get_pages().len() as u32;
            Some(pages::parse_page_array(values, total)?)
        }
        None => None,
    };
    let src = ctx.sandbox.resolve(&path)?;
    let out = ctx.sandbox.resolve_for_write(&out_path)?;
    let rotated = pages::rotate_pages(&src, &out, rotation, page_list.as_deref(), relative)?;
    Ok(json!({ "path": out.display().to_string(), "rotated": rotated }))
}

fn delete_pages_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let out_path = required_str_arg(&args, "out_path")?;
    let page_values = args
        .get("pages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::InvalidArgs("pages required".into()))?;
    let src = ctx.sandbox.resolve(&path)?;
    let probe = lopdf::Document::load(&src)
        .map_err(|e| ToolError::Execution(format!("load {}: {e}", src.display())))?;
    let total = probe.get_pages().len() as u32;
    let page_list = pages::parse_page_array(page_values, total)?;
    let out = ctx.sandbox.resolve_for_write(&out_path)?;
    let remaining = pages::delete_pages(&src, &out, &page_list)?;
    Ok(json!({ "path": out.display().to_string(), "pages": remaining }))
}
