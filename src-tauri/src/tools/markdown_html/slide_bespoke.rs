//! Marp CLI Bespoke template shell for slide HTML output.

use super::output::mermaid_init_script;

const BESPOKE_VIEWER_CSS: &str = include_str!("../../../assets/markdown-vendor/bespoke-viewer.css");
const BESPOKE_JS: &str = include_str!("../../../assets/markdown-vendor/bespoke.js");
const MARP_BROWSER_POLYFILL: &str =
    include_str!("../../../assets/markdown-vendor/marp-browser-polyfill.js");

const BESPOKE_OSC: &str = r#"<div class="bespoke-marp-osc"><button data-bespoke-marp-osc="prev" tabindex="-1" title="上一页">Previous slide</button><span data-bespoke-marp-osc="page"></span><button data-bespoke-marp-osc="next" tabindex="-1" title="下一页">Next slide</button><button data-bespoke-marp-osc="fullscreen" tabindex="-1" title="全屏 (f)">Toggle fullscreen</button><button data-bespoke-marp-osc="overview" tabindex="-1" title="概览 (o)">Toggle overview view</button><button data-bespoke-marp-osc="presenter" tabindex="-1" title="演讲者模式 (p)">Open presenter view</button></div>"#;

/// Marp CLI uses `#:$p` as the deck root; Marpit renders `div.marpit`.
pub fn prepare_bespoke_deck(slide_html: &str, slide_css: &str) -> (String, String) {
    (adapt_marpit_html(slide_html), adapt_marpit_css(slide_css))
}

fn adapt_marpit_html(html: &str) -> String {
    let trimmed = html.trim();
    if let Some(inner) = trimmed.strip_prefix("<div class=\"marpit\">") {
        if let Some(inner) = inner.strip_suffix("</div>") {
            return format!(r#"<div id=":$p">{inner}</div>"#);
        }
    }
    if trimmed.contains("id=\":$p\"") {
        return trimmed.to_string();
    }
    format!(r#"<div id=":$p">{trimmed}</div>"#)
}

fn adapt_marpit_css(css: &str) -> String {
    css.replace("div.marpit", r"div#\:\$p")
}

pub fn assemble_slide_html(
    lang: &str,
    slide_html: &str,
    slide_css: &str,
    needs_katex: bool,
    needs_mermaid: bool,
    assets_prefix: &str,
) -> String {
    let (deck_html, deck_css) = prepare_bespoke_deck(slide_html, slide_css);
    let mut out = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,height=device-height,initial-scale=1"/>
<meta name="apple-mobile-web-app-capable" content="yes"/>
<title>Slide</title>
<style>{BESPOKE_VIEWER_CSS}</style>
<style>{deck_css}</style>
"#
    );
    if needs_katex {
        out.push_str(&format!(
            r#"<link rel="stylesheet" href="{assets_prefix}katex.min.css"/>"#
        ));
    }
    out.push_str("</head>\n<body>\n");
    out.push_str(BESPOKE_OSC);
    out.push_str(&deck_html);
    out.push_str("<script>");
    out.push_str(MARP_BROWSER_POLYFILL);
    out.push_str("</script>\n<script>");
    out.push_str(BESPOKE_JS);
    out.push_str("</script>\n");
    if needs_katex {
        out.push_str(&format!(
            r#"<script src="{assets_prefix}katex.min.js"></script>
<script>
document.querySelectorAll('p,span,div').forEach(el=>{{
  if(el.childElementCount===0 && /\$[^$\n]+\$/.test(el.textContent||'')) {{
    try {{ el.innerHTML = katex.renderToString((el.textContent||'').replace(/\$/g,''), {{throwOnError:false}}); }} catch(_) {{}}
  }}
}});
</script>
"#
        ));
    }
    if needs_mermaid {
        out.push_str(&mermaid_init_script(assets_prefix));
    }
    out.push_str("</body></html>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapts_marpit_wrapper_to_bespoke_root() {
        let html = r#"<div class="marpit"><svg data-marpit-svg=""></svg></div>"#;
        let out = adapt_marpit_html(html);
        assert!(out.contains(r#"id=":$p""#));
        assert!(!out.contains("class=\"marpit\""));
    }

    #[test]
    fn adapts_marpit_css_selectors() {
        let css = "div.marpit > svg > foreignObject > section { color: red; }";
        let out = adapt_marpit_css(css);
        assert!(out.contains(r"div#\:\$p"));
        assert!(!out.contains("div.marpit"));
    }

    #[test]
    fn assembled_slide_has_bespoke_shell() {
        let html = assemble_slide_html(
            "zh-CN",
            r#"<div class="marpit"><svg data-marpit-svg=""></svg></div>"#,
            "div.marpit > svg > foreignObject > section { padding: 1em; }",
            false,
            false,
            "",
        );
        assert!(html.contains("bespoke-marp-osc"));
        assert!(html.contains(r#"id=":$p""#));
        assert!(html.contains(r"div#\:\$p"));
        assert!(!html.contains("slide-stage"));
    }
}
