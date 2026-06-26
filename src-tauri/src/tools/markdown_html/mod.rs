mod bundled;
mod convert;
mod output;
mod slide_bespoke;

pub use output::{resolve_output_path, rewrite_local_image_srcs, MAX_INPUT_BYTES};

use std::path::{Path, PathBuf};

use convert::{convert_document, convert_slide};
use output::{assemble_document_html, rel_from_root, validate_input, write_assets, ConvertOptions};
use serde_json::{json, Value};
use slide_bespoke::assemble_slide_html;

use super::{ensure_parent_dir, required_str_arg, ToolContext, ToolError, ToolSpec};
use bundled::{Profile, TemplateMeta};

pub fn markdown_to_html_tool() -> ToolSpec {
    ToolSpec {
        name: "markdown_to_html",
        description: "Convert a Markdown (.md) file in the project sandbox to static HTML (slide / report / resume). \
            Prerequisite: skill_read markdown once per conversation; new documents should call markdown_list_templates and markdown_read_template first. \
            Prefer bundled offline assets (./assets/...); external links in body are allowed. Output MUST be under project root, NOT .cache/skill-run/.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Project-relative .md source file" },
                "out_path": { "type": "string", "description": "Output .html file or directory (directory writes index.html + assets/)" },
                "profile": { "type": "string", "enum": ["slide", "report", "resume"] },
                "template": { "type": "string", "description": "Template id, e.g. slide/default, report/github-light" },
                "options": {
                    "type": "object",
                    "properties": {
                        "toc": { "type": "boolean", "default": true },
                        "math": { "type": "boolean", "default": true },
                        "highlight": { "type": "boolean", "default": true },
                        "mermaid": { "type": "boolean", "default": true },
                        "lang": { "type": "string", "default": "zh-CN" }
                    }
                }
            },
            "required": ["path", "out_path", "profile"]
        }),
        handler: markdown_to_html_handler,
    }
}

pub fn markdown_list_templates_tool() -> ToolSpec {
    ToolSpec {
        name: "markdown_list_templates",
        description: "List built-in Markdown HTML templates for slide / report / resume profiles.",
        parameters: json!({ "type": "object", "properties": {} }),
        handler: list_templates_handler,
    }
}

pub fn markdown_read_template_tool() -> ToolSpec {
    ToolSpec {
        name: "markdown_read_template",
        description: "Read example Markdown (with frontmatter) for a built-in template id.",
        parameters: json!({
            "type": "object",
            "properties": {
                "template": { "type": "string", "description": "Template id, e.g. report/github-light" }
            },
            "required": ["template"]
        }),
        handler: read_template_handler,
    }
}

fn markdown_to_html_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let path = required_str_arg(&args, "path")?;
    let out_path = required_str_arg(&args, "out_path")?;
    let profile_str = required_str_arg(&args, "profile")?;
    let profile = Profile::parse(&profile_str).ok_or_else(|| {
        ToolError::InvalidArgs(format!(
            "profile must be slide, report, or resume (got {profile_str})"
        ))
    })?;

    let template = resolve_template(profile, args.get("template").and_then(|v| v.as_str()))?;
    let options = parse_options(&args);

    let src = ctx.sandbox.resolve(&path).map_err(|e| match e {
        crate::core::sandbox::SandboxError::NotFound
        | crate::core::sandbox::SandboxError::Io(_) => {
            ToolError::Execution(format!("路径不存在: {path}"))
        }
        other => ToolError::Sandbox(other),
    })?;
    let markdown =
        std::fs::read_to_string(&src).map_err(|e| ToolError::Execution(e.to_string()))?;
    validate_input(profile, &markdown).map_err(ToolError::InvalidArgs)?;

    let html_path = resolve_output_path(&out_path).map_err(ToolError::InvalidArgs)?;
    let html_abs = ctx
        .sandbox
        .resolve_for_write(
            html_path
                .to_str()
                .ok_or_else(|| ToolError::InvalidArgs("invalid out_path".into()))?,
        )
        .map_err(ToolError::Sandbox)?;
    ensure_parent_dir(&html_abs)?;

    let theme_css = bundled::theme_css(template);
    let root = ctx.sandbox.root();
    let html_rel = html_path.to_string_lossy().replace('\\', "/");
    let mut written_paths = Vec::new();

    match profile {
        Profile::Slide => {
            let theme_name = template.id.split('/').nth(1).unwrap_or("default");
            let slide =
                convert_slide(&markdown, &theme_css, theme_name).map_err(ToolError::Execution)?;
            let needs_katex = slide.needs_katex && options.math;
            let needs_mermaid = slide.needs_mermaid && options.mermaid;
            let slide_html = rewrite_local_image_srcs(&slide.html, &path, &html_rel, root);
            let html = assemble_slide_html(
                &options.lang,
                &slide_html,
                &slide.css,
                needs_katex,
                needs_mermaid,
                if needs_katex || needs_mermaid {
                    "./assets/"
                } else {
                    ""
                },
            );
            if needs_katex || needs_mermaid {
                let assets_abs = assets_dir_for(&html_abs);
                record_asset_paths(
                    root,
                    &assets_abs,
                    &write_assets(
                        &assets_abs,
                        &theme_css,
                        false,
                        false,
                        needs_katex,
                        needs_mermaid,
                    )
                    .map_err(ToolError::Execution)?,
                    &mut written_paths,
                );
            }
            write_html_file(&html_abs, &html)?;
        }
        Profile::Report | Profile::Resume => {
            let resume_grid = matches!(template.id, "resume/two-col" | "resume/even");
            let doc =
                convert_document(&markdown, &options, resume_grid).map_err(ToolError::Execution)?;
            let needs_katex = doc.needs_katex && options.math;
            let needs_mermaid = doc.needs_mermaid && options.mermaid;
            let needs_highlight = options.highlight;
            let body_html = rewrite_local_image_srcs(&doc.body_html, &path, &html_rel, root);
            let html = assemble_document_html(
                profile,
                &options.lang,
                &doc.meta,
                &body_html,
                &doc.toc,
                &options,
                needs_katex,
                needs_mermaid,
                needs_highlight,
            );
            let assets_abs = assets_dir_for(&html_abs);
            record_asset_paths(
                root,
                &assets_abs,
                &write_assets(
                    &assets_abs,
                    &theme_css,
                    true,
                    needs_highlight,
                    needs_katex,
                    needs_mermaid,
                )
                .map_err(ToolError::Execution)?,
                &mut written_paths,
            );
            write_html_file(&html_abs, &html)?;
        }
    }

    if let Some(html_rel) = rel_from_root(root, &html_abs) {
        written_paths.insert(0, html_rel);
    }
    written_paths.sort();
    written_paths.dedup();

    Ok(json!({
        "path": html_rel,
        "profile": profile.as_str(),
        "template": template.id,
        "written_paths": written_paths,
    }))
}

fn assets_dir_for(html_abs: &Path) -> PathBuf {
    html_abs
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("assets")
}

fn record_asset_paths(
    root: &Path,
    assets_abs: &Path,
    names: &[String],
    written_paths: &mut Vec<String>,
) {
    for name in names {
        if let Some(p) = rel_from_root(root, &assets_abs.join(name)) {
            written_paths.push(p);
        }
    }
}

fn write_html_file(html_abs: &Path, html: &str) -> Result<(), ToolError> {
    std::fs::write(html_abs, html).map_err(|e| ToolError::Execution(e.to_string()))
}

fn list_templates_handler(_ctx: &ToolContext, _args: Value) -> Result<Value, ToolError> {
    let templates: Vec<Value> = bundled::LISTABLE
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "profile": m.profile.as_str(),
                "title": m.title,
                "description": m.description,
            })
        })
        .collect();
    Ok(json!({ "templates": templates }))
}

fn read_template_handler(_ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let template = required_str_arg(&args, "template")?;
    let meta = bundled::find_by_id(&template).ok_or_else(|| template_error(&template, None))?;
    let content = bundled::sample_for_template(&template)
        .ok_or_else(|| ToolError::Execution(format!("sample missing: {template}")))?;
    Ok(json!({
        "id": meta.id,
        "profile": meta.profile.as_str(),
        "content": content,
    }))
}

fn resolve_template(
    profile: Profile,
    template: Option<&str>,
) -> Result<&'static TemplateMeta, ToolError> {
    match template {
        Some(id) => bundled::find_by_id(id)
            .ok_or_else(|| template_error(id, Some(profile)))
            .and_then(|m| {
                if m.profile != profile {
                    Err(ToolError::InvalidArgs(format!(
                        "template {id} is for profile {}, not {}",
                        m.profile.as_str(),
                        profile.as_str()
                    )))
                } else {
                    Ok(m)
                }
            }),
        None => bundled::default_for_profile(profile).ok_or_else(|| {
            ToolError::Execution(format!("no default template for {}", profile.as_str()))
        }),
    }
}

fn template_error(id: &str, profile: Option<Profile>) -> ToolError {
    let available = match profile {
        Some(p) => bundled::ids_for_profile(p).join(", "),
        None => bundled::list_ids().join(", "),
    };
    ToolError::InvalidArgs(format!("unknown template: {id}. Available: {available}"))
}

fn parse_options(args: &Value) -> ConvertOptions {
    let mut options = ConvertOptions::default();
    let Some(opts) = args.get("options") else {
        return options;
    };
    if let Some(v) = opts.get("toc").and_then(|v| v.as_bool()) {
        options.toc = v;
    }
    if let Some(v) = opts.get("math").and_then(|v| v.as_bool()) {
        options.math = v;
    }
    if let Some(v) = opts.get("highlight").and_then(|v| v.as_bool()) {
        options.highlight = v;
    }
    if let Some(v) = opts.get("mermaid").and_then(|v| v.as_bool()) {
        options.mermaid = v;
    }
    if let Some(v) = opts.get("lang").and_then(|v| v.as_str()) {
        options.lang = v.to_string();
    }
    options
}
