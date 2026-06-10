use super::{ToolContext, ToolError, ToolSpec};
use docx_rs::*;
use office_oxide::create::create_from_markdown;
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

