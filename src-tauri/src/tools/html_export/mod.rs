mod print;

use super::{ensure_parent_dir, required_str_arg, ToolContext, ToolError, ToolSpec};
use print::{pdf_page_count, render_html_to_pdf, resolve_html_entry, ExportOptions};
use serde_json::{json, Value};
use tauri::{AppHandle, Runtime};

const TIMEOUT_SECS: u64 = 30;

pub fn tool() -> ToolSpec {
    ToolSpec {
        name: "html_to_pdf",
        description: "Export an existing HTML file (or directory with index.html) in the project sandbox to PDF via system WebView. Independent of html-report generation — input HTML may come from any source. Does not require skill_read.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project-relative .html file or directory containing index.html"
                },
                "out_path": {
                    "type": "string",
                    "description": "Output .pdf path in sandbox"
                },
                "page_size": {
                    "type": "string",
                    "enum": ["A4", "Letter"],
                    "default": "A4"
                },
                "landscape": {
                    "type": "boolean",
                    "default": false
                },
                "margin_mm": {
                    "type": "number",
                    "default": 15,
                    "description": "Page margin in millimeters"
                }
            },
            "required": ["path", "out_path"]
        }),
        handler: stub_handler,
    }
}

fn stub_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let _ = prepare_export(ctx, &args)?;
    Err(ToolError::Execution(
        "html_to_pdf requires AppHandle; use async registry path".into(),
    ))
}

pub async fn handler<R: Runtime>(
    ctx: &ToolContext<'_>,
    app: &AppHandle<R>,
    args: Value,
) -> Result<Value, ToolError> {
    let (html_entry, out, out_path, options) = prepare_export(ctx, &args)?;

    tokio::time::timeout(
        std::time::Duration::from_secs(TIMEOUT_SECS),
        render_html_to_pdf(app, &html_entry, &out, &options),
    )
    .await
    .map_err(|_| ToolError::Execution(format!("html_to_pdf 超时（{TIMEOUT_SECS}s）")))?
    .map_err(ToolError::Execution)?;

    let pages = pdf_page_count(&out).map_err(ToolError::Execution)?;
    if pages == 0 {
        return Err(ToolError::Execution("导出的 PDF 无页面".into()));
    }

    Ok(json!({
        "path": out_path,
        "pages": pages
    }))
}

fn prepare_export(
    ctx: &ToolContext<'_>,
    args: &Value,
) -> Result<
    (
        std::path::PathBuf,
        std::path::PathBuf,
        String,
        ExportOptions,
    ),
    ToolError,
> {
    let path = required_str_arg(args, "path")?;
    let out_path = required_str_arg(args, "out_path")?;
    if !out_path.to_ascii_lowercase().ends_with(".pdf") {
        return Err(ToolError::InvalidArgs("out_path must end with .pdf".into()));
    }

    let resolved = ctx.sandbox.resolve(&path).map_err(|e| match e {
        crate::core::sandbox::SandboxError::NotFound
        | crate::core::sandbox::SandboxError::Io(_) => {
            ToolError::Execution(format!("路径不存在: {path}"))
        }
        other => ToolError::Sandbox(other),
    })?;
    let html_entry = resolve_html_entry(&resolved).map_err(ToolError::Execution)?;

    let out = ctx
        .sandbox
        .resolve_for_write(&out_path)
        .map_err(ToolError::Sandbox)?;
    ensure_parent_dir(&out)?;

    let page_size = args
        .get("page_size")
        .and_then(|v| v.as_str())
        .unwrap_or("A4");
    if !matches!(page_size, "A4" | "Letter") {
        return Err(ToolError::InvalidArgs(
            "page_size must be A4 or Letter".into(),
        ));
    }

    let margin_mm = args
        .get("margin_mm")
        .and_then(|v| v.as_f64())
        .unwrap_or(15.0);
    if !(0.0..=100.0).contains(&margin_mm) {
        return Err(ToolError::InvalidArgs(
            "margin_mm must be between 0 and 100".into(),
        ));
    }

    let options = ExportOptions {
        page_size: page_size.to_string(),
        landscape: args
            .get("landscape")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        margin_mm,
    };

    Ok((html_entry, out, out_path, options))
}
