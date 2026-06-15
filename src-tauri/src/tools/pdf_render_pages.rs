use crate::tools::pdf::render_pages_cached;
use crate::tools::pdf_cache;
use crate::tools::{required_str_arg, ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};

pub fn tool() -> ToolSpec {
    ToolSpec {
        name: "pdf_render_pages",
        description: "Render PDF pages to PNG in .cache/pdf/ (cached). Usually unnecessary — pdf_read renders internally when vision is needed. Use for manual image_read workflows.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Project-relative PDF path" },
                "pages": { "type": "string", "description": "1-based pages: all, 1-4, or 1,3,5 (default all)" },
                "dpi": { "type": "integer", "description": "Render DPI (default 150)" }
            },
            "required": ["path"]
        }),
        handler,
    }
}

pub fn handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let rel_path = required_str_arg(&args, "path")?;
    let abs_path = ctx.sandbox.resolve(&rel_path)?;
    if abs_path
        .extension()
        .and_then(|e| e.to_str())
        .is_none_or(|ext| !ext.eq_ignore_ascii_case("pdf"))
    {
        return Err(ToolError::InvalidArgs("path must be a .pdf file".into()));
    }

    let dpi = pdf_cache::parse_dpi(args.get("dpi").and_then(|v| v.as_u64()))
        .map_err(ToolError::InvalidArgs)?;

    let pages_spec =
        pdf_cache::normalize_pages_arg(args.get("pages")).map_err(ToolError::InvalidArgs)?;
    let pages_spec = pages_spec.as_deref();

    let result = render_pages_cached(ctx.sandbox.root(), &rel_path, &abs_path, dpi, pages_spec)
        .map_err(ToolError::Execution)?;

    let page_paths: Vec<&str> = result
        .manifest
        .pages
        .iter()
        .map(|p| p.path.as_str())
        .collect();

    Ok(json!({
        "cache_hit": result.cache_hit,
        "cache_key": result.cache_key,
        "page_count": result.manifest.page_count,
        "pages": page_paths,
        "manifest_path": format!(".cache/pdf/{}/manifest.json", result.cache_key),
        "dpi": result.manifest.dpi,
        "pages_spec": result.manifest.pages_spec,
    }))
}
