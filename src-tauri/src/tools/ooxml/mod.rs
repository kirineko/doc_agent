mod comment;
mod pack;
mod redline;
pub mod style_lint;
mod unpack;
pub(crate) mod validate;

use super::{ensure_parent_dir, required_str_arg, ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};

pub fn unpack_tool() -> ToolSpec {
    ToolSpec {
        name: "ooxml_unpack",
        description: "Unpack docx/pptx/xlsx to an editable directory of XML parts",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "out_dir": { "type": "string" },
                "merge_runs": { "type": "boolean", "default": true }
            },
            "required": ["path", "out_dir"]
        }),
        handler: unpack_handler,
    }
}

pub fn pack_tool() -> ToolSpec {
    ToolSpec {
        name: "ooxml_pack",
        description:
            "Pack an unpacked OOXML directory into docx/pptx/xlsx with validation and auto-repair",
        parameters: json!({
            "type": "object",
            "properties": {
                "dir": { "type": "string" },
                "out_path": { "type": "string" },
                "original": { "type": "string" }
            },
            "required": ["dir", "out_path"]
        }),
        handler: pack_handler,
    }
}

pub fn comment_tool() -> ToolSpec {
    ToolSpec {
        name: "docx_comment",
        description: "Add a comment (or reply) to an unpacked docx directory",
        parameters: json!({
            "type": "object",
            "properties": {
                "dir": { "type": "string" },
                "id": { "type": "integer" },
                "text": { "type": "string" },
                "author": { "type": "string", "default": "Claude" },
                "parent": { "type": "integer" }
            },
            "required": ["dir", "id", "text"]
        }),
        handler: comment_handler,
    }
}

pub fn accept_changes_tool() -> ToolSpec {
    ToolSpec {
        name: "docx_accept_changes",
        description: "Accept all tracked changes in a docx and write a clean copy",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "out_path": { "type": "string" }
            },
            "required": ["path"]
        }),
        handler: accept_changes_handler,
    }
}

fn unpack_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let out_dir = required_str_arg(&args, "out_dir")?;
    let merge_runs = args
        .get("merge_runs")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let src = ctx.sandbox.resolve(&path)?;
    let dst = ctx.sandbox.resolve_for_write(&out_dir)?;
    let report = unpack::unpack(&src, &dst, merge_runs)?;
    Ok(json!({ "out_dir": dst.display().to_string(), "parts": report.parts }))
}

fn pack_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let dir = required_str_arg(&args, "dir")?;
    let out_path = required_str_arg(&args, "out_path")?;
    let original = args
        .get("original")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let src = ctx.sandbox.resolve(&dir)?;
    let dst = ctx.sandbox.resolve_for_write(&out_path)?;
    ensure_parent_dir(&dst)?;
    let original_path = original
        .as_deref()
        .map(|p| ctx.sandbox.resolve(p))
        .transpose()
        .map_err(ToolError::Sandbox)?;
    pack::pack(&src, &dst, original_path.as_deref())?;
    Ok(json!({ "path": dst.display().to_string() }))
}

fn comment_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let dir = required_str_arg(&args, "dir")?;
    let id = args
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ToolError::InvalidArgs("id required".into()))? as u32;
    let text = required_str_arg(&args, "text")?;
    let author = args
        .get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("Claude");
    let parent = args
        .get("parent")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let unpacked = ctx.sandbox.resolve(&dir)?;
    comment::add_comment(&unpacked, id, &text, author, parent)?;
    Ok(json!({ "dir": unpacked.display().to_string(), "comment_id": id }))
}

fn accept_changes_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let out_path = args
        .get("out_path")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.clone());
    let src = ctx.sandbox.resolve(&path)?;
    let dst = ctx.sandbox.resolve_for_write(&out_path)?;
    ensure_parent_dir(&dst)?;
    redline::accept_changes(&src, &dst)?;
    Ok(json!({ "path": dst.display().to_string() }))
}
