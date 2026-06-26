use std::time::Duration;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::object::builtins::JsPromise;
use boa_engine::{Context, JsValue, Source};
use serde_json::{json, Value};

use super::bundled::Profile;
use super::output::{validate_input, ConvertOptions, DocumentResult, SlideResult, TocEntry};

const BOA_POLYFILL: &str = r#"
var console = { log: function(){}, warn: function(){}, error: function(){}, info: function(){}, debug: function(){} };
var process = { env: {}, browser: true };
"#;

const MARKED_SHIM: &str = r#"
globalThis.marked = typeof marked !== "undefined" ? (marked.default || marked) : globalThis.marked;
"#;

const UNWRAP_SHIM: &str = r#"
if (typeof MarkdownConvert === "object" && MarkdownConvert.default) MarkdownConvert = MarkdownConvert.default;
if (typeof Marpit === "object") Marpit = Marpit.default || Marpit.Marpit || Marpit;
"#;

const SLIDE_SCRIPT: &str = r#"
(function(inputJson) {
  const input = JSON.parse(inputJson);
  const marpit = new Marpit({ html: true, inlineSVG: true });
  if (input.themeCss) marpit.themeSet.add(input.themeCss);
  let md = input.markdown || "";
  const fm = MarkdownConvert.matter(md);
  md = fm.content;
  md = MarkdownConvert.preprocessGfmTables(md);
  const directives = Object.assign({}, fm.data || {}, input.directives || {});
  delete directives.title;
  delete directives.author;
  delete directives.date;
  delete directives.marp;
  const dirLines = Object.entries(directives)
    .filter(([, v]) => v !== null && v !== undefined && typeof v !== "object")
    .map(([k, v]) => k + ": " + v);
  if (dirLines.length) md = "---\n" + dirLines.join("\n") + "\n---\n\n" + md;
  const { html, css } = marpit.render(md);
  const needs = MarkdownConvert.detectNeeds(input.markdown || "");
  return JSON.stringify({ html, css, needsKatex: !!needs.katex, needsMermaid: !!needs.mermaid });
})
"#;

const DOCUMENT_SCRIPT: &str = r#"
(function(inputJson) {
  const input = JSON.parse(inputJson);
  const fm = MarkdownConvert.matter(input.markdown || "");
  const opts = input.options || {};
  let body = MarkdownConvert.parseMarkdown(fm.content || "", opts);
  body = MarkdownConvert.injectFigureCaptionClasses(body);
  body = MarkdownConvert.wrapFigureBlocks(body);
  if (opts.toc !== false) body = MarkdownConvert.injectHeadingIds(body);
  const toc = opts.toc === false ? [] : MarkdownConvert.extractHeadings(body);
  const needs = MarkdownConvert.detectNeeds(input.markdown || "");
  return JSON.stringify({
    meta: fm.data || {},
    bodyHtml: body,
    toc,
    needsKatex: !!needs.katex,
    needsMermaid: !!needs.mermaid
  });
})
"#;

pub fn convert_slide(
    markdown: &str,
    theme_css: &str,
    theme_name: &str,
) -> Result<SlideResult, String> {
    validate_input(Profile::Slide, markdown)?;
    let payload = json!({
        "markdown": markdown,
        "themeCss": theme_css,
        "directives": { "paginate": true, "theme": theme_name }
    });
    let raw = run_boa(Profile::Slide, &payload, SLIDE_SCRIPT)?;
    parse_slide_result(&raw)
}

pub fn convert_document(
    markdown: &str,
    options: &ConvertOptions,
) -> Result<DocumentResult, String> {
    validate_input(Profile::Report, markdown)?;
    let payload = json!({
        "markdown": markdown,
        "options": {
            "toc": options.toc,
            "highlight": options.highlight,
        }
    });
    let raw = run_boa(Profile::Report, &payload, DOCUMENT_SCRIPT)?;
    parse_document_result(&raw)
}

const MARKED_BUNDLE: &str = include_str!("../../../assets/js/marked.bundle.js");
const MARKDOWN_BUNDLE: &str = include_str!("../../../assets/js/markdown.bundle.js");
const MARP_BUNDLE: &str = include_str!("../../../assets/js/marp-core.bundle.js");

fn run_boa(profile: Profile, payload: &Value, script: &str) -> Result<String, String> {
    let payload_str =
        serde_json::to_string(payload).map_err(|e| format!("serialize payload: {e}"))?;
    let bundles: Vec<&'static str> = match profile {
        Profile::Slide => vec![MARKED_BUNDLE, MARKDOWN_BUNDLE, MARP_BUNDLE],
        Profile::Report | Profile::Resume => vec![MARKED_BUNDLE, MARKDOWN_BUNDLE],
    };

    let (tx, rx) = std::sync::mpsc::channel();
    let payload_str = payload_str.clone();
    let script = script.to_string();

    std::thread::Builder::new()
        .name("markdown_html".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                eval_conversion(&bundles, &script, &payload_str)
            }))
            .unwrap_or_else(|_| Err("conversion panicked".into()));
            let _ = tx.send(result);
        })
        .map_err(|e| format!("spawn conversion thread: {e}"))?;

    rx.recv_timeout(Duration::from_secs(62))
        .map_err(|_| "markdown conversion timeout".to_string())?
}

fn eval_conversion(bundles: &[&str], script: &str, payload: &str) -> Result<String, String> {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(BOA_POLYFILL))
        .map_err(|e| format!("polyfill failed: {e}"))?;
    for (i, bundle) in bundles.iter().enumerate() {
        context
            .eval(Source::from_bytes(bundle))
            .map_err(|e| format!("bundle {i} failed: {e}"))?;
        if i == 0 {
            context
                .eval(Source::from_bytes(MARKED_SHIM))
                .map_err(|e| format!("marked shim failed: {e}"))?;
        }
    }
    context
        .eval(Source::from_bytes(UNWRAP_SHIM))
        .map_err(|e| format!("unwrap shim failed: {e}"))?;
    let call = format!("({script})({payload_json})", payload_json = json!(payload));
    let result = context
        .eval(Source::from_bytes(&call))
        .map_err(|e| format!("conversion failed: {e}"))?;
    let result = settle_promise(&mut context, result)?;
    js_to_string(&mut context, &result)
}

fn settle_promise(context: &mut Context, value: JsValue) -> Result<JsValue, String> {
    let Some(obj) = value.as_object() else {
        return Ok(value);
    };
    let Ok(promise) = JsPromise::from_object(obj.clone()) else {
        return Ok(value);
    };
    context
        .run_jobs()
        .map_err(|e| format!("microtask failed: {e}"))?;
    match promise.state() {
        PromiseState::Fulfilled(v) => Ok(v),
        PromiseState::Rejected(e) => Err(format!("conversion rejected: {}", e.display())),
        PromiseState::Pending => Err("conversion promise pending".into()),
    }
}

fn js_to_string(context: &mut Context, value: &JsValue) -> Result<String, String> {
    value
        .to_string(context)
        .map_err(|e| e.to_string())
        .map(|s| s.to_std_string_escaped())
}

fn parse_slide_result(raw: &str) -> Result<SlideResult, String> {
    let v: Value = serde_json::from_str(raw).map_err(|e| format!("parse slide result: {e}"))?;
    Ok(SlideResult {
        html: v
            .get("html")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        css: v
            .get("css")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        needs_katex: v
            .get("needsKatex")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        needs_mermaid: v
            .get("needsMermaid")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
    })
}

fn parse_document_result(raw: &str) -> Result<DocumentResult, String> {
    let v: Value = serde_json::from_str(raw).map_err(|e| format!("parse document result: {e}"))?;
    let toc = v
        .get("toc")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(TocEntry {
                        id: item.get("id")?.as_str()?.to_string(),
                        text: item.get("text")?.as_str()?.to_string(),
                        level: item.get("level")?.as_u64()? as u8,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(DocumentResult {
        body_html: v
            .get("bodyHtml")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        meta: v.get("meta").cloned().unwrap_or(json!({})),
        toc,
        needs_katex: v
            .get("needsKatex")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        needs_mermaid: v
            .get("needsMermaid")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
    })
}

#[cfg(test)]
mod boa_marked_tests {
    use super::*;
    use boa_engine::{Context, Source};

    #[test]
    fn marked_bundle_exposes_global_parse() {
        let mut context = Context::default();
        context
            .eval(Source::from_bytes(BOA_POLYFILL))
            .expect("polyfill");
        context
            .eval(Source::from_bytes(MARKED_BUNDLE))
            .expect("marked bundle");
        let ty = context
            .eval(Source::from_bytes(
                "JSON.stringify({ g: typeof globalThis.marked, p: typeof globalThis.marked?.parse, l: typeof marked })",
            ))
            .expect("check");
        let s = js_to_string(&mut context, &ty).expect("str");
        assert!(
            s.contains(r#""p":"function""#),
            "marked.parse expected, got {s}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::markdown_html::bundled;

    #[test]
    fn report_table_renders() {
        let md = "| a | b |\n| --- | --- |\n| 1 | 2 |";
        let out = convert_document(md, &ConvertOptions::default()).expect("convert");
        assert!(out.body_html.contains("<table"));
    }

    #[test]
    fn report_image_with_italic_caption_wraps_figure() {
        let md = "![alt](../images/x.jpg)\n*菲比——示例图注*";
        let out = convert_document(md, &ConvertOptions::default()).expect("convert");
        assert!(out.body_html.contains("md-figure"));
        assert!(out.body_html.contains("<figcaption>"));
    }

    #[test]
    fn slide_multipage() {
        let md = "# One\n\n---\n\n# Two";
        let out = convert_slide(
            md,
            &bundled::theme_css(bundled::default_for_profile(Profile::Slide).unwrap()),
            "default",
        )
        .expect("convert");
        assert!(out.html.matches("<section").count() >= 2);
        assert!(
            out.html.contains("data-marpit-svg"),
            "Marp inline SVG slides"
        );
    }

    #[test]
    fn slide_table_renders() {
        let md = "# Table\n\n| A | B |\n| --- | --- |\n| 1 | 2 |";
        let out = convert_slide(
            md,
            &bundled::theme_css(bundled::default_for_profile(Profile::Slide).unwrap()),
            "default",
        )
        .expect("convert");
        assert!(
            out.html.contains("<table"),
            "slide tables need Marp markdown-it"
        );
    }

    #[test]
    fn slide_with_yaml_frontmatter_no_empty_lead_slide() {
        let md = r#"---
marp: true
theme: gaia
paginate: true
title: 跨性别女性：理解、尊重与共融
author: Doc Agent
---

<!-- _class: lead -->
# 跨性别女性
## 理解 · 尊重 · 共融

---

## 什么是跨性别女性？
"#;
        let meta = bundled::find_by_id("slide/gaia").unwrap();
        let out = convert_slide(md, &bundled::theme_css(meta), "gaia").expect("convert");
        let section_count = out.html.matches("<section").count();
        assert!(
            section_count >= 2,
            "expected at least 2 slides, got {section_count}. html snippet: {}",
            &out.html[..out.html.len().min(500)]
        );
        assert_eq!(
            section_count, 2,
            "frontmatter must not create an extra empty lead slide (got {section_count})"
        );
        assert!(
            !out.html.contains("marp: true") && !out.html.contains("theme: gaia paginate"),
            "frontmatter must not leak into rendered HTML"
        );
        let first_h1 = out
            .html
            .split("<section")
            .nth(1)
            .unwrap_or("")
            .contains("跨性别女性");
        assert!(first_h1, "first slide should contain title heading");
    }

    #[test]
    fn slide_non_default_theme_applied() {
        let md = "# Title";
        let meta = bundled::find_by_id("slide/gaia").unwrap();
        let out = convert_slide(md, &bundled::theme_css(meta), "gaia").expect("convert");
        let default = convert_slide(
            md,
            &bundled::theme_css(bundled::default_for_profile(Profile::Slide).unwrap()),
            "default",
        )
        .expect("convert");
        assert_ne!(out.css, default.css, "gaia theme should change slide CSS");
        assert!(
            out.css.contains("312e81") || out.css.contains("135deg"),
            "gaia theme CSS should include gaia visual rules"
        );
    }

    #[test]
    fn oversize_input_rejected() {
        use crate::tools::markdown_html::MAX_INPUT_BYTES;
        let md = "x".repeat(MAX_INPUT_BYTES + 1);
        let err = convert_document(&md, &ConvertOptions::default()).unwrap_err();
        assert!(err.contains("规模上限"));
    }
}
