use crate::agent::types::ModelId;
use crate::tools::pdf::extract_text;
use crate::tools::pdf::render_pages_cached;
use crate::tools::pdf_cache::{self, PageEntry};
use crate::tools::vision_subcall::vision_subcall;
use crate::tools::{required_str_arg, ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};
use std::path::Path;

const VISION_BATCH: usize = 4;
const VISION_PAGE_PROMPT: &str = "按图片顺序逐页提取全部可见文字、公式与题号，保留数学符号，用 Markdown 输出。";

pub fn tool() -> ToolSpec {
    ToolSpec {
        name: "pdf_read",
        description: description_for_model(ModelId::KimiK26),
        parameters: parameters_for_model(ModelId::KimiK26),
        handler: |_ctx, _args| Err(ToolError::NotImplemented),
    }
}

pub fn description_for_model(model_id: ModelId) -> &'static str {
    if model_id.supports_vision() {
        "Read PDF. Omit mode (recommended): auto extracts text then vision-understands every page. Use mode=vision only for scan-only PDFs without text layer. Do NOT use mode=text on vision models — use office_read_to_markdown for text-only."
    } else {
        "Read PDF. Omit mode (recommended): auto returns PDFium text. Use mode=text for explicit text-only. mode=vision is not available on this model."
    }
}

pub fn parameters_for_model(model_id: ModelId) -> Value {
    let mode_schema = if model_id.supports_vision() {
        json!({
            "type": "string",
            "enum": ["auto", "vision"],
            "description": "Omit in normal use (same as auto). auto: text extraction then vision on all pages. vision: skip text, render+vision only (scanned PDFs)."
        })
    } else {
        json!({
            "type": "string",
            "enum": ["auto", "text"],
            "description": "Omit in normal use (same as auto). auto/text: PDFium text only on non-vision models."
        })
    };

    json!({
        "type": "object",
        "properties": {
            "path": { "type": "string" },
            "mode": mode_schema,
            "pages": {
                "type": "string",
                "description": "1-based page range for render in auto/vision (default all); also accepts array e.g. [1,3]"
            },
            "dpi": { "type": "integer", "description": "Render DPI when using vision (default 150, 72-300)" }
        },
        "required": ["path"]
    })
}

pub async fn handler(
    ctx: &ToolContext<'_>,
    args: Value,
    model_id: ModelId,
) -> Result<Value, ToolError> {
    let rel_path = required_str_arg(&args, "path")?;
    let abs_path = ctx.sandbox.resolve(&rel_path)?;
    if abs_path
        .extension()
        .and_then(|e| e.to_str())
        .is_none_or(|ext| !ext.eq_ignore_ascii_case("pdf"))
    {
        return Err(ToolError::InvalidArgs("path must be a .pdf file".into()));
    }

    let mode = resolve_mode(&args, model_id)?;
    match mode {
        PdfReadMode::Text => read_text(&abs_path),
        PdfReadMode::Vision => {
            let dpi = parse_dpi_arg(&args)?;
            let pages_spec = parse_pages_arg(&args)?;
            read_vision(
                ctx,
                model_id,
                &rel_path,
                &abs_path,
                dpi,
                pages_spec.as_deref(),
                "vision",
                None,
            )
            .await
        }
        PdfReadMode::Auto => read_auto(ctx, model_id, &rel_path, &abs_path, &args).await,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PdfReadMode {
    Text,
    Vision,
    Auto,
}

fn resolve_mode(args: &Value, model_id: ModelId) -> Result<PdfReadMode, ToolError> {
    let mode = args.get("mode").and_then(|v| v.as_str());
    match mode {
        None | Some("auto") => Ok(PdfReadMode::Auto),
        Some("text") => {
            if model_id.supports_vision() {
                // Agent 误传 mode=text 时仍走 auto，避免公式/版式丢失
                Ok(PdfReadMode::Auto)
            } else {
                Ok(PdfReadMode::Text)
            }
        }
        Some("vision") => {
            if model_id.supports_vision() {
                Ok(PdfReadMode::Vision)
            } else {
                Err(ToolError::InvalidArgs(
                    "mode=vision requires a vision-capable model; use mode=auto or mode=text".into(),
                ))
            }
        }
        Some(other) => Err(ToolError::InvalidArgs(format!("unknown mode: {other}"))),
    }
}

async fn read_auto(
    ctx: &ToolContext<'_>,
    model_id: ModelId,
    rel_path: &str,
    abs_path: &Path,
    args: &Value,
) -> Result<Value, ToolError> {
    let text_layer = try_extract_text(abs_path)?;

    if !model_id.supports_vision() {
        let markdown = text_layer.ok_or_else(|| {
            ToolError::Execution(
                "PDF 未提取到文本（可能为扫描件）；请切换至支持 vision 的模型（如 Kimi K2.6）后重试 pdf_read"
                    .into(),
            )
        })?;
        return Ok(json!({
            "mode": "auto",
            "resolved": "text",
            "markdown": markdown,
        }));
    }

    let dpi = parse_dpi_arg(args)?;
    let pages_spec = parse_pages_arg(args)?;
    read_vision(
        ctx,
        model_id,
        rel_path,
        abs_path,
        dpi,
        pages_spec.as_deref(),
        "auto",
        text_layer.as_deref(),
    )
    .await
}

async fn read_vision(
    ctx: &ToolContext<'_>,
    model_id: ModelId,
    rel_path: &str,
    abs_path: &Path,
    dpi: u32,
    pages_spec: Option<&str>,
    response_mode: &'static str,
    text_layer: Option<&str>,
) -> Result<Value, ToolError> {
    let render = render_pages_cached(ctx.sandbox.root(), rel_path, abs_path, dpi, pages_spec)
        .map_err(ToolError::Execution)?;

    let mut sections = Vec::new();
    for chunk in render.manifest.pages.chunks(VISION_BATCH) {
        let page_label = format_page_label(chunk);
        let paths: Vec<String> = chunk.iter().map(|p| p.path.clone()).collect();
        let prompt = format!(
            "{VISION_PAGE_PROMPT}\n本批为第 {page_label} 页，共 {} 张图。",
            paths.len()
        );
        let text = vision_subcall(ctx, model_id, &paths, &prompt).await?;
        sections.push(format!("## Pages {page_label}\n\n{text}"));
    }

    let mut out = json!({
        "mode": response_mode,
        "resolved": "vision",
        "cache_hit": render.cache_hit,
        "cache_key": render.cache_key,
        "page_count": render.manifest.page_count,
        "markdown": sections.join("\n\n"),
    });
    if let Some(layer) = text_layer {
        out["text_layer"] = json!(layer);
    }
    Ok(out)
}

fn parse_dpi_arg(args: &Value) -> Result<u32, ToolError> {
    pdf_cache::parse_dpi(args.get("dpi").and_then(|v| v.as_u64()))
        .map_err(ToolError::InvalidArgs)
}

fn parse_pages_arg(args: &Value) -> Result<Option<String>, ToolError> {
    pdf_cache::normalize_pages_arg(args.get("pages")).map_err(ToolError::InvalidArgs)
}

fn try_extract_text(abs_path: &Path) -> Result<Option<String>, ToolError> {
    let text = extract_text(abs_path).map_err(ToolError::Execution)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn format_page_label(entries: &[PageEntry]) -> String {
    let indices: Vec<u32> = entries.iter().map(|e| e.index).collect();
    let contiguous =
        indices.len() <= 1 || indices.windows(2).all(|pair| pair[1] == pair[0] + 1);
    if contiguous && indices.len() > 1 {
        format!("{}-{}", indices[0], indices[indices.len() - 1])
    } else {
        indices
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn read_text(abs_path: &Path) -> Result<Value, ToolError> {
    let markdown = try_extract_text(abs_path)?.ok_or_else(|| {
        ToolError::Execution(
            "PDF 未提取到文本（可能为扫描件）；请使用 vision 模型调用 pdf_read（mode=auto 或 mode=vision）"
                .into(),
        )
    })?;
    Ok(json!({
        "mode": "text",
        "markdown": markdown,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::pdf_cache::PageEntry;

    #[test]
    fn resolve_mode_auto_is_default() {
        assert_eq!(
            resolve_mode(&json!({}), ModelId::DeepSeekV4Flash).unwrap(),
            PdfReadMode::Auto
        );
        assert_eq!(
            resolve_mode(&json!({ "mode": "auto" }), ModelId::KimiK26).unwrap(),
            PdfReadMode::Auto
        );
    }

    #[test]
    fn resolve_mode_vision_requires_vision_model() {
        assert!(resolve_mode(&json!({ "mode": "vision" }), ModelId::DeepSeekV4Flash).is_err());
        assert_eq!(
            resolve_mode(&json!({ "mode": "vision" }), ModelId::KimiK26).unwrap(),
            PdfReadMode::Vision
        );
    }

    #[test]
    fn format_page_label_contiguous_range() {
        let chunk = vec![
            PageEntry {
                index: 1,
                path: "p1.png".into(),
            },
            PageEntry {
                index: 2,
                path: "p2.png".into(),
            },
        ];
        assert_eq!(format_page_label(&chunk), "1-2");
    }

    #[test]
    fn format_page_label_non_contiguous_list() {
        let chunk = vec![
            PageEntry {
                index: 1,
                path: "p1.png".into(),
            },
            PageEntry {
                index: 3,
                path: "p3.png".into(),
            },
            PageEntry {
                index: 5,
                path: "p5.png".into(),
            },
        ];
        assert_eq!(format_page_label(&chunk), "1,3,5");
    }

    #[test]
    fn resolve_mode_text_on_vision_model_maps_to_auto() {
        assert_eq!(
            resolve_mode(&json!({ "mode": "text" }), ModelId::KimiK26).unwrap(),
            PdfReadMode::Auto
        );
        assert_eq!(
            resolve_mode(&json!({ "mode": "text" }), ModelId::DeepSeekV4Flash).unwrap(),
            PdfReadMode::Text
        );
    }

    #[test]
    fn parameters_for_vision_model_omits_text_mode() {
        let params = parameters_for_model(ModelId::KimiK26);
        let modes = params["properties"]["mode"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(modes, vec!["auto", "vision"]);
        assert!(!modes.contains(&"text"));
    }

    #[test]
    fn parameters_for_non_vision_model_omits_vision_mode() {
        let params = parameters_for_model(ModelId::DeepSeekV4Flash);
        let modes = params["properties"]["mode"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(modes, vec!["auto", "text"]);
    }
}
