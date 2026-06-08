use super::{ToolContext, ToolError, ToolSpec};
use docx_rs::*;
use office_oxide::create::create_from_markdown;
use office_oxide::edit::EditableDocument;
use office_oxide::format::DocumentFormat;
use serde_json::{json, Value};
use std::fs::File;

pub fn create_tool() -> ToolSpec {
    ToolSpec {
        name: "word_create",
        description: "Create a Word document from Markdown or plain title/body",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "markdown": { "type": "string" },
                "title": { "type": "string" },
                "body": { "type": "string" }
            },
            "required": ["path"]
        }),
        handler: create_handler,
    }
}

pub fn edit_tool() -> ToolSpec {
    ToolSpec {
        name: "word_edit",
        description: "Replace text in an existing Word document while preserving formatting",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "find": { "type": "string" },
                "replace": { "type": "string" },
                "output_path": { "type": "string" }
            },
            "required": ["path", "find", "replace"]
        }),
        handler: edit_handler,
    }
}

fn create_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let resolved = ctx.sandbox.resolve_for_write(path)?;
    if let Some(parent) = resolved.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ToolError::Execution(e.to_string()))?;
    }

    if let Some(markdown) = args.get("markdown").and_then(|v| v.as_str()) {
        create_from_markdown(markdown, DocumentFormat::Docx, &resolved)
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        return Ok(json!({ "path": resolved.display().to_string(), "mode": "markdown" }));
    }

    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");
    let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let file = File::create(&resolved).map_err(|e| ToolError::Execution(e.to_string()))?;
    Docx::new()
        .add_paragraph(Paragraph::new().add_run(Run::new().add_text(title).bold()))
        .add_paragraph(Paragraph::new().add_run(Run::new().add_text(body)))
        .build()
        .pack(file)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({ "path": resolved.display().to_string(), "mode": "docx-rs" }))
}

fn edit_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let find = args
        .get("find")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("find required".into()))?;
    let replace = args
        .get("replace")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("replace required".into()))?;
    let source = ctx.sandbox.resolve(path)?;
    let output = if let Some(out) = args.get("output_path").and_then(|v| v.as_str()) {
        ctx.sandbox.resolve_for_write(out)?
    } else {
        source.clone()
    };
    let mut editable =
        EditableDocument::open(&source).map_err(|e| ToolError::Execution(e.to_string()))?;
    let count = editable.replace_text(find, replace);
    editable
        .save(&output)
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(json!({
        "replacements": count,
        "output_path": output.display().to_string()
    }))
}
