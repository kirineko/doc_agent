use super::pdf;
use super::{ToolContext, ToolError, ToolSpec};
use office_oxide::format::DocumentFormat;
use office_oxide::Document;
use serde_json::{json, Value};
use std::fs;
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

/// 读取文档为 Markdown/纯文本，供工具与推荐问上下文共用。
pub fn read_document_text(path: &Path) -> Result<String, ToolError> {
    if is_pdf(path) {
        let text = pdf::extract_text(path).map_err(ToolError::Execution)?;
        let markdown = text.trim();
        if markdown.is_empty() {
            return Err(ToolError::Execution(
                "PDF 未提取到文本（可能为扫描件或纯图片 PDF）".into(),
            ));
        }
        return Ok(markdown.to_string());
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if matches!(ext, "md" | "csv" | "txt") {
        return fs::read_to_string(path).map_err(|e| ToolError::Execution(e.to_string()));
    }

    if DocumentFormat::from_extension(ext).is_none() {
        return Err(ToolError::InvalidArgs(format!(
            "unsupported format '.{ext}'; supported: docx, xlsx, pptx, doc, xls, ppt, pdf, md, csv"
        )));
    }

    let doc = Document::open(path).map_err(|e| ToolError::Execution(e.to_string()))?;
    Ok(doc.to_markdown())
}

fn read_markdown_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let resolved = ctx.sandbox.resolve(path)?;
    let markdown = read_document_text(&resolved)?;
    let format = if is_pdf(&resolved) {
        "pdf".to_string()
    } else {
        let ext = resolved.extension().and_then(|e| e.to_str()).unwrap_or("");
        DocumentFormat::from_extension(ext)
            .map(|f| format!("{f:?}"))
            .unwrap_or_else(|| ext.to_string())
    };
    Ok(json!({
        "format": format,
        "markdown": markdown
    }))
}

fn is_pdf(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
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
