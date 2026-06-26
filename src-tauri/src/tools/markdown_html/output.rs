use std::path::{Path, PathBuf};

use super::bundled::Profile;

pub const MAX_INPUT_BYTES: usize = 512 * 1024;
pub const MAX_SLIDE_PAGES: usize = 200;

pub const KATEX_JS: &[u8] = include_bytes!("../../../assets/markdown-vendor/katex.min.js");
pub const KATEX_CSS: &[u8] = include_bytes!("../../../assets/markdown-vendor/katex.min.css");
pub const MERMAID_JS: &[u8] = include_bytes!("../../../assets/markdown-vendor/mermaid.min.js");
pub const HIGHLIGHT_CSS: &[u8] = include_bytes!("../../../assets/markdown-vendor/highlight.css");

#[derive(Debug, Clone)]
pub struct ConvertOptions {
    pub toc: bool,
    pub math: bool,
    pub highlight: bool,
    pub mermaid: bool,
    pub lang: String,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            toc: true,
            math: true,
            highlight: true,
            mermaid: true,
            lang: "zh-CN".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlideResult {
    pub html: String,
    pub css: String,
    pub needs_katex: bool,
    pub needs_mermaid: bool,
}

#[derive(Debug, Clone)]
pub struct DocumentResult {
    pub body_html: String,
    pub meta: serde_json::Value,
    pub toc: Vec<TocEntry>,
    pub needs_katex: bool,
    pub needs_mermaid: bool,
}

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub id: String,
    pub text: String,
    pub level: u8,
}

fn slide_body_for_page_count(markdown: &str) -> &str {
    if !markdown.starts_with("---") {
        return markdown;
    }
    let rest = &markdown[3..];
    let Some(pos) = rest.find("\n---") else {
        return markdown;
    };
    rest[pos + 4..]
        .strip_prefix('\n')
        .unwrap_or(&rest[pos + 4..])
}

pub fn validate_input(profile: Profile, markdown: &str) -> Result<(), String> {
    if markdown.len() > MAX_INPUT_BYTES {
        return Err(format!(
            "Markdown 超过规模上限（{} 字节 > {} 字节），请拆分后重试",
            markdown.len(),
            MAX_INPUT_BYTES
        ));
    }
    if profile == Profile::Slide {
        let body = slide_body_for_page_count(markdown);
        let pages = body.matches("\n---").count() + 1;
        if pages > MAX_SLIDE_PAGES {
            return Err(format!(
                "幻灯片页数超过上限（{pages} > {MAX_SLIDE_PAGES}），请拆分后重试"
            ));
        }
    }
    Ok(())
}

pub fn resolve_output_path(out_path: &str) -> Result<PathBuf, String> {
    if out_path.replace('\\', "/").contains(".cache/skill-run") {
        return Err("禁止写入 .cache/skill-run/，请使用项目目录内路径".into());
    }
    let path = Path::new(out_path);
    let is_dir_hint = out_path.ends_with('/') || out_path.ends_with('\\');
    let is_html = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("html"));

    if is_dir_hint || !is_html {
        Ok(path.to_path_buf().join("index.html"))
    } else {
        Ok(path.to_path_buf())
    }
}

pub fn write_assets(
    assets_dir: &Path,
    theme_css: &str,
    include_theme: bool,
    needs_highlight: bool,
    needs_katex: bool,
    needs_mermaid: bool,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(assets_dir).map_err(|e| e.to_string())?;
    let mut written = Vec::new();
    if include_theme {
        std::fs::write(assets_dir.join("theme.css"), theme_css).map_err(|e| e.to_string())?;
        written.push("theme.css".into());
    }
    if needs_highlight {
        std::fs::write(assets_dir.join("highlight.css"), HIGHLIGHT_CSS)
            .map_err(|e| e.to_string())?;
        written.push("highlight.css".into());
    }
    if needs_katex {
        std::fs::write(assets_dir.join("katex.min.js"), KATEX_JS).map_err(|e| e.to_string())?;
        std::fs::write(assets_dir.join("katex.min.css"), KATEX_CSS).map_err(|e| e.to_string())?;
        written.push("katex.min.js".into());
        written.push("katex.min.css".into());
    }
    if needs_mermaid {
        std::fs::write(assets_dir.join("mermaid.min.js"), MERMAID_JS).map_err(|e| e.to_string())?;
        written.push("mermaid.min.js".into());
    }
    Ok(written)
}

/// Rewrite local `<img src>` paths from markdown-relative to HTML-output-relative.
pub fn rewrite_local_image_srcs(html: &str, md_rel: &str, html_rel: &str, root: &Path) -> String {
    let md_dir = Path::new(md_rel)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let html_dir = Path::new(html_rel)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(pos) = rest.find("<img") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];
        let Some(rel_end) = after.find('>') else {
            out.push_str(after);
            break;
        };
        let tag = &after[..=rel_end];
        out.push_str(&rewrite_img_tag_src(tag, &md_dir, &html_dir, root));
        rest = &after[rel_end + 1..];
    }
    out.push_str(rest);
    out
}

fn rewrite_img_tag_src(tag: &str, md_dir: &Path, html_dir: &Path, root: &Path) -> String {
    let Some((quote, src)) = extract_img_src(tag) else {
        return tag.to_string();
    };
    if is_remote_or_data_src(&src) {
        return tag.to_string();
    }
    let resolved = resolve_image_path(&src, md_dir, root);
    let new_src = relativize_path(html_dir, &resolved);
    let old = format!("{quote}{src}{quote}");
    let new = format!("{quote}{new_src}{quote}");
    tag.replacen(&old, &new, 1)
}

fn extract_img_src(tag: &str) -> Option<(char, String)> {
    for quote in ['"', '\''] {
        let marker = format!("src={quote}");
        if let Some(start) = tag.find(&marker) {
            let value_start = start + marker.len();
            let rest = &tag[value_start..];
            if let Some(end) = rest.find(quote) {
                return Some((quote, rest[..end].to_string()));
            }
        }
    }
    None
}

fn is_remote_or_data_src(src: &str) -> bool {
    let s = src.trim();
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("data:")
        || s.starts_with("//")
}

fn resolve_image_path(src: &str, md_dir: &Path, root: &Path) -> PathBuf {
    let from_md = normalize_rel_path(md_dir, src);
    if root.join(&from_md).exists() {
        return from_md;
    }
    let from_root = normalize_rel_path(Path::new(""), src);
    if root.join(&from_root).exists() {
        return from_root;
    }
    from_md
}

fn normalize_rel_path(base: &Path, rel: &str) -> PathBuf {
    use std::path::Component;
    let mut parts: Vec<&str> = base
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    let rel_norm = rel.replace('\\', "/");
    for comp in Path::new(&rel_norm).components() {
        match comp {
            Component::Normal(s) => {
                if let Some(part) = s.to_str() {
                    parts.push(part);
                }
            }
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir => {}
            _ => {}
        }
    }
    PathBuf::from(parts.join("/"))
}

fn relativize_path(from_dir: &Path, target: &Path) -> String {
    use std::path::Component;
    let from: Vec<_> = from_dir
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    let to: Vec<_> = target
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    let mut i = 0;
    while i < from.len() && i < to.len() && from[i] == to[i] {
        i += 1;
    }
    let mut out: Vec<&str> = std::iter::repeat_n("..", from.len().saturating_sub(i)).collect();
    out.extend(&to[i..]);
    if out.is_empty() {
        ".".into()
    } else {
        out.join("/")
    }
}

pub fn rel_from_root(root: &Path, abs: &Path) -> Option<String> {
    abs.strip_prefix(root).ok().map(|p| {
        let s = p.to_string_lossy().replace('\\', "/");
        if s.is_empty() {
            ".".into()
        } else {
            s
        }
    })
}

pub(crate) fn mermaid_init_script(assets_prefix: &str) -> String {
    format!(
        r#"<script src="{assets_prefix}mermaid.min.js"></script>
<script>
document.querySelectorAll('pre code.language-mermaid').forEach(el=>{{
  const div=document.createElement('div'); div.className='mermaid'; div.textContent=el.textContent||''; el.closest('pre').replaceWith(div);
}});
mermaid.initialize({{startOnLoad:false, theme:'neutral'}});
mermaid.run({{querySelector:'.mermaid'}});
</script>
"#
    )
}

#[allow(clippy::too_many_arguments)]
pub fn assemble_document_html(
    profile: Profile,
    lang: &str,
    meta: &serde_json::Value,
    body_html: &str,
    toc: &[TocEntry],
    options: &ConvertOptions,
    needs_katex: bool,
    needs_mermaid: bool,
    needs_highlight: bool,
) -> String {
    let raw_title = meta
        .get("title")
        .or_else(|| meta.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Document");
    let title = html_escape(raw_title);
    let author = meta.get("author").and_then(|v| v.as_str());
    let date = meta
        .get("date")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| {
            meta.get("subtitle")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        });

    let mut head = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title}</title>
<link rel="stylesheet" href="./assets/theme.css"/>
"#
    );
    if needs_highlight {
        head.push_str(r#"<link rel="stylesheet" href="./assets/highlight.css"/>"#);
    }
    if needs_katex {
        head.push_str(r#"<link rel="stylesheet" href="./assets/katex.min.css"/>"#);
    }
    head.push_str("\n</head>\n<body>\n");

    let cover = if profile == Profile::Resume {
        let name = meta
            .get("name")
            .and_then(|v| v.as_str())
            .or_else(|| meta.get("title").and_then(|v| v.as_str()))
            .unwrap_or("Resume");
        let role = meta
            .get("title")
            .and_then(|v| v.as_str())
            .filter(|t| meta.get("name").and_then(|v| v.as_str()) != Some(*t));
        let mut contacts: Vec<String> = Vec::new();
        for key in [
            "email", "phone", "location", "website", "github", "linkedin",
        ] {
            if let Some(v) = meta.get(key).and_then(|v| v.as_str()) {
                if !v.trim().is_empty() {
                    contacts.push(html_escape(v));
                }
            }
        }
        let role_html = role
            .map(|r| format!(r#"<div class="role">{}</div>"#, html_escape(r)))
            .unwrap_or_default();
        let contact_html = if contacts.is_empty() {
            String::new()
        } else {
            format!(r#"<div class="contact">{}</div>"#, contacts.join(" · "))
        };
        format!(
            r#"<header class="resume-header"><h1>{}</h1>{role_html}{contact_html}</header>"#,
            html_escape(name)
        )
    } else {
        let meta_line = match (author, date.as_deref()) {
            (Some(a), Some(d)) => format!("{} · {}", html_escape(a), html_escape(d)),
            (Some(a), None) => html_escape(a),
            (None, Some(d)) => html_escape(d),
            _ => String::new(),
        };
        format!(
            r#"<div class="cover"><h1>{title}</h1>{meta}</div>"#,
            meta = if meta_line.is_empty() {
                String::new()
            } else {
                format!(r#"<div class="meta">{meta_line}</div>"#)
            }
        )
    };

    let toc_html = if options.toc && !toc.is_empty() && profile == Profile::Report {
        let mut items = String::new();
        for entry in toc {
            let cls = if entry.level == 3 {
                " class=\"level-3\""
            } else {
                ""
            };
            items.push_str(&format!(
                "<li{cls}><a href=\"#{id}\">{text}</a></li>\n",
                id = html_escape(&entry.id),
                text = html_escape(&entry.text),
            ));
        }
        format!(r#"<nav class="toc no-print"><h2>目录</h2><ul>{items}</ul></nav>"#)
    } else {
        String::new()
    };

    let main_class = match profile {
        Profile::Resume => "content resume-main",
        _ => "content markdown-body",
    };

    let mut body = format!("{cover}{toc_html}<main class=\"{main_class}\">{body_html}</main>\n");

    if needs_katex {
        body.push_str(
            r#"<script src="./assets/katex.min.js"></script>
<script>
document.querySelectorAll('code').forEach(el=>{
  const t=el.textContent||'';
  if(t.startsWith('$') && t.endsWith('$') && !el.closest('pre')) {
    try { const span=document.createElement('span'); span.innerHTML=katex.renderToString(t.slice(1,-1),{displayMode:false,throwOnError:false}); el.replaceWith(span);} catch(_) {}
  }
});
document.querySelectorAll('p').forEach(el=>{
  let t=el.innerHTML;
  if(t.includes('$$')) {
    t=t.replace(/\$\$([\s\S]+?)\$\$/g,(_,m)=>'<div class="katex-display">'+katex.renderToString(m,{displayMode:true,throwOnError:false})+'</div>');
  }
  t=t.replace(/\$([^$\n<]+?)\$/g,(_,m)=>katex.renderToString(m,{displayMode:false,throwOnError:false}));
  el.innerHTML=t;
});
</script>
"#,
        );
    }
    if needs_mermaid {
        body.push_str(&mermaid_init_script("./assets/"));
    }
    body.push_str("</body></html>");
    format!("{head}{body}")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rewrite_image_src_relative_to_html_output() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("images")).unwrap();
        fs::write(dir.path().join("images/foo.jpg"), b"x").unwrap();
        let html = r#"<p><img src="images/foo.jpg" alt="x"></p>"#;
        let out = rewrite_local_image_srcs(html, "report.md", "report-dir/index.html", dir.path());
        assert!(out.contains(r#"src="../images/foo.jpg""#));
    }

    #[test]
    fn rewrite_image_src_from_markdown_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("images")).unwrap();
        fs::write(dir.path().join("images/foo.jpg"), b"x").unwrap();
        let html = r#"<img src="../images/foo.jpg" alt="x">"#;
        let out = rewrite_local_image_srcs(html, "docs/report.md", "out/index.html", dir.path());
        assert!(out.contains(r#"src="../images/foo.jpg""#));
    }
}
