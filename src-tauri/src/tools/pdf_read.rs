use crate::agent::types::ModelId;
use crate::tools::pdf::render_pages_cached;
use crate::tools::pdf::{extract_text_pages, join_extracted_pages};
use crate::tools::pdf_cache::{self, PageEntry};
use crate::tools::pdf_judge::{judge_page_compare, JudgeVerdict};
use crate::tools::pdf_text_quality::{build_page_stats, full_text_hard_rule, pick_sample_page};
use crate::tools::vision_subcall::vision_subcall;
use crate::tools::{required_str_arg, ToolContext, ToolError, ToolSpec};
use serde_json::{json, Value};
use std::path::Path;

const VISION_BATCH: usize = 4;
/// 不超过此页数的 PDF 在 vision 模型下直接全量 vision，跳过 Judge。
const VISION_DIRECT_PAGE_THRESHOLD: u32 = 4;
/// 超过此页数的 PDF 在 vision 模型下仅返回 PDFium 文本，跳过 Judge 与全量 vision 以节省 token。
const VISION_TEXT_ONLY_PAGE_THRESHOLD: u32 = 20;
const PAGE_COUNT_THRESHOLD_NOTE: &str =
    "超过 20 页已返回 PDFium 文本；如需 vision 请用 pages 分段（如 \"1-20\"）。";
const VISION_PAGE_PROMPT: &str =
    "按图片顺序逐页提取全部可见文字、公式与题号，保留数学符号，用 Markdown 输出。";

pub fn tool() -> ToolSpec {
    ToolSpec {
        name: "pdf_read",
        description: "Read PDF intelligently: extracts text first; on vision models judges whether full page vision is needed. Pass path only.",
        parameters: parameters_schema(),
        handler: |_ctx, _args| Err(ToolError::NotImplemented),
    }
}

pub fn description_for_model(model_id: ModelId) -> &'static str {
    if model_id.supports_vision() {
        "Read PDF intelligently. Pass path only — extracts text first. PDFs with ≤4 pages go directly to full vision (no judge). PDFs over 20 pages return PDFium text only (no vision) to save tokens. PDFs with 5–20 pages may judge whether full page vision is needed. For PDFium-only without judging, use office_read_to_markdown."
    } else {
        "Read PDF via PDFium text extraction. Pass path only. Scan PDFs with no text layer require a vision-capable model (e.g. Kimi K2.6)."
    }
}

pub fn parameters_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": { "type": "string", "description": "Project-relative PDF path" },
            "pages": {
                "type": "string",
                "description": "Optional 1-based page range (default all); also accepts array e.g. [1,3]"
            },
            "dpi": { "type": "integer", "description": "Render DPI when vision runs (default 150, 72-300)" }
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

    let dpi = parse_dpi_arg(&args)?;
    let pages_spec = parse_pages_arg(&args)?;
    let pages_spec_ref = pages_spec.as_deref();

    let page_texts = extract_text_pages(&abs_path, pages_spec_ref).map_err(ToolError::Execution)?;
    let full_text = join_extracted_pages(&page_texts);
    let page_count = page_texts.len() as u32;
    let has_text = !full_text.is_empty();

    if !model_id.supports_vision() {
        if !has_text {
            return Err(ToolError::Execution(
                "PDF 未提取到文本（可能为扫描件）；请切换至支持 vision 的模型（如 Kimi K2.6）后重试 pdf_read"
                    .into(),
            ));
        }
        return Ok(json!({
            "resolved": "text",
            "page_count": page_count,
            "markdown": full_text,
        }));
    }

    if page_count > VISION_TEXT_ONLY_PAGE_THRESHOLD {
        if !has_text {
            return Err(ToolError::Execution(format!(
                "PDF 共 {page_count} 页且未提取到文本（可能为扫描件）；超过 {VISION_TEXT_ONLY_PAGE_THRESHOLD} 页时已跳过全量 vision 以节省 token。请通过 pages 参数分段读取（如 \"1-20\"），或缩小范围后重试"
            )));
        }
        return Ok(text_response(
            page_count,
            &full_text,
            judge_meta_skipped("page_count_threshold", None, None),
            Some(PAGE_COUNT_THRESHOLD_NOTE),
        ));
    }

    if page_count <= VISION_DIRECT_PAGE_THRESHOLD {
        return read_full_vision(
            ctx,
            model_id,
            &rel_path,
            &abs_path,
            dpi,
            pages_spec_ref,
            has_text.then_some(full_text.as_str()),
            judge_meta_skipped("page_count_short", None, None),
        )
        .await;
    }

    if !has_text {
        return read_full_vision(
            ctx,
            model_id,
            &rel_path,
            &abs_path,
            dpi,
            pages_spec_ref,
            None,
            judge_meta_skipped("no_text_layer", None, None),
        )
        .await;
    }

    if let Some(rule) = full_text_hard_rule(&full_text, page_count) {
        return read_full_vision(
            ctx,
            model_id,
            &rel_path,
            &abs_path,
            dpi,
            pages_spec_ref,
            Some(full_text.as_str()),
            judge_meta_skipped("hard_rule", Some(rule), None),
        )
        .await;
    }

    let stats = build_page_stats(
        &page_texts
            .iter()
            .map(|p| (p.index, p.text.clone()))
            .collect::<Vec<_>>(),
    );
    let sample = pick_sample_page(&stats)
        .ok_or_else(|| ToolError::Execution("failed to pick sample page for judge".into()))?;
    let sample_text = page_texts
        .iter()
        .find(|p| p.index == sample.index)
        .map(|p| p.text.as_str())
        .unwrap_or("");

    let sample_pages_spec = Some(sample.index.to_string());
    let render = render_pages_cached(
        ctx.sandbox.root(),
        &rel_path,
        &abs_path,
        dpi,
        sample_pages_spec.as_deref(),
    )
    .map_err(ToolError::Execution)?;

    let image_path = render
        .manifest
        .pages
        .iter()
        .find(|p| p.index == sample.index)
        .map(|p| p.path.as_str())
        .ok_or_else(|| ToolError::Execution("sample page render missing".into()))?;

    let verdict =
        match judge_page_compare(ctx, model_id, sample.index, image_path, sample_text).await {
            Ok(v) => v,
            Err(_) => {
                return read_full_vision(
                    ctx,
                    model_id,
                    &rel_path,
                    &abs_path,
                    dpi,
                    pages_spec_ref,
                    Some(full_text.as_str()),
                    judge_meta_judge_failed(sample),
                )
                .await;
            }
        };

    if verdict == JudgeVerdict::TextOk {
        return Ok(text_response(
            page_count,
            &full_text,
            judge_meta_compare(sample, &verdict, false),
            None,
        ));
    }

    read_full_vision(
        ctx,
        model_id,
        &rel_path,
        &abs_path,
        dpi,
        pages_spec_ref,
        Some(full_text.as_str()),
        judge_meta_compare(sample, &verdict, true),
    )
    .await
}

fn text_response(page_count: u32, markdown: &str, judge: Value, note: Option<&str>) -> Value {
    let mut out = json!({
        "resolved": "text",
        "page_count": page_count,
        "markdown": markdown,
        "judge": judge,
    });
    if let Some(note) = note {
        out["note"] = json!(note);
    }
    out
}

fn judge_meta_skipped(
    method: &'static str,
    hard_rule: Option<&str>,
    sample_page: Option<u32>,
) -> Value {
    json!({
        "skipped": true,
        "method": method,
        "hard_rule": hard_rule,
        "sample_page": sample_page,
    })
}

fn judge_meta_judge_failed(sample: crate::tools::pdf_text_quality::SamplePagePick) -> Value {
    json!({
        "skipped": false,
        "method": "page_compare",
        "sample_page": sample.index,
        "sample_reason": sample.reason,
        "verdict": "JUDGE_FAILED",
        "followed_by_full_vision": true,
    })
}

fn judge_meta_compare(
    sample: crate::tools::pdf_text_quality::SamplePagePick,
    verdict: &JudgeVerdict,
    followed_by_vision: bool,
) -> Value {
    json!({
        "skipped": false,
        "method": "page_compare",
        "sample_page": sample.index,
        "sample_reason": sample.reason,
        "verdict": if *verdict == JudgeVerdict::TextOk { "TEXT_OK" } else { "NEED_VISION" },
        "followed_by_full_vision": followed_by_vision,
    })
}

async fn read_full_vision(
    ctx: &ToolContext<'_>,
    model_id: ModelId,
    rel_path: &str,
    abs_path: &Path,
    dpi: u32,
    pages_spec: Option<&str>,
    text_layer: Option<&str>,
    judge: Value,
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
        "resolved": "vision",
        "cache_hit": render.cache_hit,
        "cache_key": render.cache_key,
        "page_count": render.manifest.page_count,
        "markdown": sections.join("\n\n"),
        "judge": judge,
    });
    if let Some(layer) = text_layer {
        out["text_layer"] = json!(layer);
    }
    Ok(out)
}

fn parse_dpi_arg(args: &Value) -> Result<u32, ToolError> {
    pdf_cache::parse_dpi(args.get("dpi").and_then(|v| v.as_u64())).map_err(ToolError::InvalidArgs)
}

fn parse_pages_arg(args: &Value) -> Result<Option<String>, ToolError> {
    pdf_cache::normalize_pages_arg(args.get("pages")).map_err(ToolError::InvalidArgs)
}

fn format_page_label(entries: &[PageEntry]) -> String {
    let indices: Vec<u32> = entries.iter().map(|e| e.index).collect();
    let contiguous = indices.len() <= 1 || indices.windows(2).all(|pair| pair[1] == pair[0] + 1);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_schema_has_no_mode() {
        let params = parameters_schema();
        assert!(params["properties"]["path"].is_object());
        assert!(params["properties"]["mode"].is_null());
    }

    #[test]
    fn vision_page_thresholds_are_exclusive() {
        assert!(4 <= VISION_DIRECT_PAGE_THRESHOLD);
        assert!(!(5 <= VISION_DIRECT_PAGE_THRESHOLD));
        assert!(!(20 > VISION_TEXT_ONLY_PAGE_THRESHOLD));
        assert!(21 > VISION_TEXT_ONLY_PAGE_THRESHOLD);
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
}
