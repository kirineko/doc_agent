use super::pdf;
use super::{ToolContext, ToolError, ToolSpec};
use office_oxide::format::DocumentFormat;
use office_oxide::Document;
use serde_json::{json, Value};
use std::path::Path;

pub fn read_markdown_tool() -> ToolSpec {
    ToolSpec {
        name: "office_read_to_markdown",
        description:
            "Read an Office document (docx/xlsx/pptx/doc/xls/ppt) or PDF and return Markdown/text",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
            "required": ["path"]
        }),
        handler: read_markdown_handler,
    }
}

fn read_markdown_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let resolved = ctx.sandbox.resolve(path)?;

    if is_pdf(&resolved) {
        return read_pdf_markdown(&resolved);
    }

    let ext = resolved.extension().and_then(|e| e.to_str()).unwrap_or("");
    if DocumentFormat::from_extension(ext).is_none() {
        return Err(ToolError::InvalidArgs(format!(
            "unsupported format '.{ext}'; supported: docx, xlsx, pptx, doc, xls, ppt, pdf"
        )));
    }

    let doc = Document::open(&resolved).map_err(|e| ToolError::Execution(e.to_string()))?;
    let markdown = doc.to_markdown();
    Ok(json!({
        "format": format!("{:?}", doc.format()),
        "markdown": markdown
    }))
}

fn is_pdf(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
}

fn read_pdf_markdown(path: &Path) -> Result<Value, ToolError> {
    let text = pdf::extract_text(path).map_err(ToolError::Execution)?;
    let markdown = text.trim();
    if markdown.is_empty() {
        return Err(ToolError::Execution(
            "PDF 未提取到文本（可能为扫描件或纯图片 PDF）".into(),
        ));
    }
    Ok(json!({
        "format": "pdf",
        "engine": "pdfium",
        "markdown": markdown
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn invalid_pdf_returns_error_instead_of_panicking() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("broken.pdf"), b"not-a-pdf").unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext { sandbox: &sandbox };
        let err = read_markdown_handler(&ctx, json!({ "path": "broken.pdf" })).unwrap_err();
        assert!(err.to_string().contains("PDF"));
    }
}
