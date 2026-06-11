use super::pdf;
use super::{ensure_parent_dir, ToolContext, ToolError, ToolSpec};
use office_oxide::format::DocumentFormat;
use office_oxide::Document;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const CONVERTED_SUFFIX: &str = "-converted";

pub fn read_markdown_tool() -> ToolSpec {
    ToolSpec {
        name: "office_read_to_markdown",
        description:
            "Read an Office document (docx/xlsx/pptx/doc/xls/ppt) or PDF and return Markdown/text. Prefer this for analysis on legacy .doc/.xls/.ppt — no new project file is created.",
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

pub fn convert_tool() -> ToolSpec {
    ToolSpec {
        name: "office_convert",
        description: "Convert legacy Office (.doc/.xls/.ppt) to OOXML only when required (OOXML edit/unpack, excel_write/xlsx_recalc, user asks for .docx/.xlsx/.pptx output). Do NOT use for read-only analysis — use office_read_to_markdown or data_query on .xls instead. Conversion may lose formatting; output must use -converted suffix.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Source legacy file path" },
                "out_path": { "type": "string", "description": "Optional output path; must end with -converted.{docx|xlsx|pptx}" }
            },
            "required": ["path"]
        }),
        handler: convert_handler,
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

pub(crate) fn legacy_target_extension(ext: &str) -> Option<&'static str> {
    match ext.to_ascii_lowercase().as_str() {
        "doc" => Some("docx"),
        "xls" => Some("xlsx"),
        "ppt" => Some("pptx"),
        _ => None,
    }
}

/// 旧格式 → OOXML，query.rs 的 .xls 数据源与 convert_handler 共用。
pub(crate) fn convert_legacy(src: &Path, dest: &Path) -> Result<(), ToolError> {
    let doc = Document::open(src).map_err(|e| ToolError::Execution(e.to_string()))?;
    doc.save_as(dest)
        .map_err(|e| ToolError::Execution(e.to_string()))
}

fn is_valid_converted_out_path(path: &Path, target_ext: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext == target_ext)
        && path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|stem| stem.ends_with(CONVERTED_SUFFIX))
}

/// 同目录下生成 `{stem}-converted.{target_ext}`。
fn converted_sibling_path(src: &Path, target_ext: &str) -> Result<PathBuf, ToolError> {
    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ToolError::InvalidArgs("source has no filename".into()))?;
    Ok(src.with_file_name(format!("{stem}{CONVERTED_SUFFIX}.{target_ext}")))
}

fn convert_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let resolved = ctx.sandbox.resolve(path)?;
    let src_ext = resolved.extension().and_then(|e| e.to_str()).unwrap_or("");
    let target_ext = legacy_target_extension(src_ext).ok_or_else(|| {
        ToolError::InvalidArgs(format!(
            "only legacy formats .doc/.xls/.ppt can be converted; got '.{src_ext}'"
        ))
    })?;

    let out_rel = if let Some(out) = args.get("out_path").and_then(|v| v.as_str()) {
        if !is_valid_converted_out_path(Path::new(out), target_ext) {
            return Err(ToolError::InvalidArgs(format!(
                "out_path must use -converted suffix and .{target_ext} extension, e.g. report-converted.{target_ext}"
            )));
        }
        out.to_string()
    } else {
        converted_sibling_path(Path::new(path), target_ext)?
            .to_string_lossy()
            .into_owned()
    };

    let dest = ctx.sandbox.resolve_for_write(&out_rel)?;
    if dest.exists() {
        return Err(ToolError::Execution(format!(
            "output already exists: {out_rel}"
        )));
    }
    ensure_parent_dir(&dest)?;
    convert_legacy(&resolved, &dest)?;

    Ok(json!({
        "path": out_rel,
        "format": target_ext
    }))
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
        let ctx = ToolContext::new(&sandbox);
        let err = read_markdown_handler(&ctx, json!({ "path": "broken.pdf" })).unwrap_err();
        assert!(err.to_string().contains("PDF"));
    }

    #[test]
    fn converted_sibling_path_keeps_directory() {
        assert_eq!(
            converted_sibling_path(Path::new("报告.xls"), "xlsx").unwrap(),
            PathBuf::from("报告-converted.xlsx")
        );
        assert_eq!(
            converted_sibling_path(Path::new("docs/报告.xls"), "xlsx").unwrap(),
            PathBuf::from("docs/报告-converted.xlsx")
        );
    }

    #[test]
    fn converted_output_name_validation() {
        assert!(is_valid_converted_out_path(
            Path::new("memo-converted.docx"),
            "docx"
        ));
        assert!(!is_valid_converted_out_path(Path::new("memo.docx"), "docx"));
        // 扩展名必须与源格式的目标格式一致
        assert!(!is_valid_converted_out_path(
            Path::new("memo-converted.xlsx"),
            "docx"
        ));
    }

    #[test]
    fn convert_rejects_non_legacy_source() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("new.docx"), b"x").unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = convert_handler(&ctx, json!({ "path": "new.docx" })).unwrap_err();
        assert!(err.to_string().contains("legacy"));
    }

    #[test]
    fn convert_rejects_out_path_without_converted_suffix() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("legacy.doc"), b"x").unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = convert_handler(
            &ctx,
            json!({ "path": "legacy.doc", "out_path": "out.docx" }),
        )
        .unwrap_err();
        assert!(err.to_string().contains("-converted"));
    }
}
