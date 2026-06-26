mod comment;
mod pack;
mod redline;
pub mod style_lint;
mod unpack;
pub(crate) mod validate;

use super::{ensure_parent_dir, required_str_arg, ToolContext, ToolError, ToolSpec};
use crate::core::cache_paths::ooxml_work_dir;
use serde_json::{json, Value};

pub fn unpack_tool() -> ToolSpec {
    ToolSpec {
        name: "ooxml_unpack",
        description: "Unpack docx/pptx/xlsx to an editable directory of XML parts. \
            Omit out_dir to get an auto-generated workspace under .cache/ooxml/<session_key>/<work_key>/ (work_key includes session+turn+source; reuse the returned out_dir for fs_patch/ooxml_pack). \
            Do not use a fixed shared out_dir like unpacked/ across parallel sessions.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "out_dir": {
                    "type": "string",
                    "description": "Optional output directory; omit for auto-generated .cache/ooxml/<session_key>/<work_key>/"
                },
                "merge_runs": { "type": "boolean", "default": true }
            },
            "required": ["path"]
        }),
        handler: unpack_handler,
    }
}

pub fn pack_tool() -> ToolSpec {
    ToolSpec {
        name: "ooxml_pack",
        description:
            "Pack an unpacked OOXML directory into docx/pptx/xlsx with validation and auto-repair. \
            dir MUST be the out_dir returned by ooxml_unpack (often under .cache/ooxml/).",
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
        description: "Add a comment (or reply) to an unpacked docx directory and wire it into the \
            document so Word renders it. dir MUST be the out_dir from ooxml_unpack. \
            The comment is anchored to the paragraph at `paragraph_index` (0-based, counting only \
            top-level <w:p> elements under <w:body> in word/document.xml). Optionally pass \
            `text_hint` to assert that paragraph contains the given substring (guards against \
            miscounting); a mismatch is an error. The tool itself inserts commentRangeStart/End + \
            commentReference anchors, writes comments.xml (and commentsExtended/people.xml as \
            needed), so callers must NOT add range markup manually.",
        parameters: json!({
            "type": "object",
            "properties": {
                "dir": { "type": "string" },
                "id": { "type": "integer" },
                "text": { "type": "string" },
                "author": { "type": "string", "default": "Claude" },
                "parent": { "type": "integer" },
                "paragraph_index": {
                    "type": "integer",
                    "description": "0-based index of the top-level <w:p> in word/document.xml to anchor the comment to."
                },
                "text_hint": {
                    "type": "string",
                    "description": "Optional substring that the target paragraph must contain; mismatch is an error."
                }
            },
            "required": ["dir", "id", "text", "paragraph_index"]
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
    let (out_dir, generated) = match args.get("out_dir").and_then(|v| v.as_str()) {
        Some(dir) => (dir.to_string(), false),
        None => {
            if ctx.session_id.is_empty() || ctx.turn_id.is_empty() {
                return Err(ToolError::InvalidArgs(
                    "out_dir required when turn context is unavailable".into(),
                ));
            }
            (ooxml_work_dir(ctx.session_id, ctx.turn_id, &path), true)
        }
    };
    let merge_runs = args
        .get("merge_runs")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let src = ctx.sandbox.resolve(&path)?;
    let dst = ctx.sandbox.resolve_for_write(&out_dir)?;
    // 生成目录由 session+turn+path 确定；若已存在说明本轮已解包过该文档。
    // 直接重新解包会 remove_dir_all 静默删除已编辑的 XML，因此拒绝并引导复用。
    if generated && dst.exists() {
        return Err(ToolError::Execution(format!(
            "该文档已在本轮解包到 {out_dir}，请直接使用该 out_dir 继续编辑，避免重复解包覆盖已修改内容；如确需重新解包，请显式传入新的 out_dir。"
        )));
    }
    let report = unpack::unpack(&src, &dst, merge_runs)?;
    Ok(json!({ "out_dir": out_dir, "parts": report.parts }))
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
    let paragraph_index = args
        .get("paragraph_index")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ToolError::InvalidArgs("paragraph_index required".into()))?
        as usize;
    let text_hint = args
        .get("text_hint")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let unpacked = ctx.sandbox.resolve(&dir)?;
    comment::add_comment(
        &unpacked,
        id,
        &text,
        author,
        parent,
        paragraph_index,
        text_hint.as_deref(),
    )?;
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
