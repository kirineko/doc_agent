use super::{ToolContext, ToolError, ToolSpec};
use crate::core::skills;
use crate::tools::ooxml::style_lint::lint_docx;
use crate::tools::runtime;
use crate::tools::skill_run_tmp::{self, SCRIPT_REL};
use serde_json::{json, Map, Value};
use std::time::Duration;

pub fn read_tool() -> ToolSpec {
    ToolSpec {
        name: "skill_read",
        description: "Read a built-in Document Skill guide. \
            skill MUST be one of: docx, pdf, pptx, xlsx, html-report, clarify. \
            doc is optional (default SKILL.md). \
            docx template editing: doc=editing.md. pptx API: doc=pptxgenjs.md; pptx template: skill=pptx, doc=editing.md. \
            Filenames like pptxgenjs.md alone work only when unique; editing.md requires skill=docx or skill=pptx.",
        parameters: json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "string",
                    "description": "Skill id: docx, pdf, pptx, xlsx, html-report, clarify. Filenames like pptxgenjs.md are auto-resolved to skill=pptx."
                },
                "doc": {
                    "type": "string",
                    "description": "Optional doc filename, e.g. pptxgenjs.md, editing.md, reference.md"
                }
            },
            "required": ["skill"]
        }),
        handler: read_handler,
    }
}

pub fn run_tool() -> ToolSpec {
    ToolSpec {
        name: "skill_run",
        description: "Execute JavaScript in the embedded skill runtime. \
            Before generating any .docx/.pptx/.xlsx deliverable you MUST first call skill_read for that format. \
            Provide exactly one of code (inline script) or path (project-relative .js file). \
            Long scripts: failed inline runs are saved to .cache/skill-run/script.js; repair with fs_patch (not fs_write) and rerun with path. \
            After writing deliverables the script stays at script_path for in-turn fixes; it is cleaned automatically when the turn ends. \
            Define async function main() returning JSON-serializable value; do NOT call main() at end. \
            Libraries (auto-loaded): ExcelJS, PptxGenJS, PDFLib, docx — use globals OR require('exceljs') etc. \
            Save files: await wb.xlsx.writeFile('out.xlsx') (shimmed), doc_write(path, base64), doc_write_bytes(path, bytes). \
            Buffer.from(buf).toString('base64') and fs.writeFileSync are shimmed. No fetch/npm/shell. \
            After writing .docx files, check style_warnings and verify content with office_read_to_markdown before finishing.",
        parameters: json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Inline JavaScript source. Saved to .cache/skill-run/script.js before execution."
                },
                "path": {
                    "type": "string",
                    "description": "Project-relative path to a JavaScript file, e.g. .cache/skill-run/script.js after repair."
                },
                "timeout_secs": { "type": "integer", "default": 30 }
            },
            "oneOf": [
                { "required": ["code"], "not": { "required": ["path"] } },
                { "required": ["path"], "not": { "required": ["code"] } }
            ]
        }),
        handler: run_handler,
    }
}

fn read_handler(_ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let skill = super::required_str_arg(&args, "skill")?;
    let doc = args.get("doc").and_then(|v| v.as_str());
    let (resolved_skill, resolved_doc) =
        skills::resolve_skill_doc(&skill, doc).map_err(ToolError::Execution)?;
    let content = skills::read(&skill, doc).map_err(ToolError::Execution)?;
    Ok(json!({
        "skill": resolved_skill,
        "doc": resolved_doc,
        "content": content
    }))
}

struct ScriptSource {
    code: String,
    diagnostic_path: Option<String>,
    from_inline: bool,
}

fn resolve_script_source(ctx: &ToolContext, args: &Value) -> Result<ScriptSource, ToolError> {
    let code = args.get("code").and_then(|v| v.as_str());
    let path = args.get("path").and_then(|v| v.as_str());
    match (code, path) {
        (Some(_), Some(_)) => Err(ToolError::InvalidArgs(
            "skill_run accepts either code or path, not both".into(),
        )),
        (None, None) => Err(ToolError::InvalidArgs("code or path required".into())),
        (Some(code), None) => {
            skill_run_tmp::write_temp_script(ctx, code)?;
            Ok(ScriptSource {
                code: code.to_string(),
                diagnostic_path: Some(SCRIPT_REL.to_string()),
                from_inline: true,
            })
        }
        (None, Some(path)) => {
            let script = skill_run_tmp::read_script_path(ctx, path)?;
            Ok(ScriptSource {
                code: script,
                diagnostic_path: Some(path.to_string()),
                from_inline: false,
            })
        }
    }
}

fn run_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let source = resolve_script_source(ctx, &args)?;
    let timeout_secs = args
        .get("timeout_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(30)
        .clamp(1, 120);

    let result = runtime::execute_script(
        ctx.sandbox,
        &source.code,
        Duration::from_secs(timeout_secs),
        source.diagnostic_path.as_deref(),
    );

    match result {
        Ok((result, written_paths)) => {
            let style_warnings = collect_docx_style_warnings(ctx, &written_paths);
            let has_style_warnings = style_warnings
                .as_ref()
                .is_some_and(|warnings| !warnings.is_empty());
            let retain_script = should_retain_skill_run_script(&written_paths, has_style_warnings);
            if retain_script {
                // 成功执行：清除上一次失败遗留的 error.json，仅保留可修复的脚本
                skill_run_tmp::clear_error(ctx);
            } else {
                skill_run_tmp::cleanup(ctx);
            }
            let mut response = json!({ "result": result });
            if !written_paths.is_empty() {
                response["written_paths"] = json!(written_paths);
            }
            if retain_script && skill_run_tmp::tmp_dir_exists(ctx) {
                response["script_path"] = json!(SCRIPT_REL);
                response["script_retain_reason"] =
                    json!(script_retain_reason(has_style_warnings, &written_paths));
                response["repair_hint"] = json!(
                    "To fix the deliverable, use fs_read + fs_patch on script_path, then skill_run with path. \
                     The script is cleaned automatically when the turn ends."
                );
            }
            if let Some(style_warnings) = style_warnings {
                response["style_warnings"] = Value::Object(style_warnings);
                response["style_hint"] =
                    json!("检测到排版问题，请修正后重新生成（参考 docx skill 的中文排版章节）");
            }
            Ok(response)
        }
        Err(err) => {
            let error_value = err.to_json_value();
            if source.from_inline || source.diagnostic_path.as_deref() == Some(SCRIPT_REL) {
                let _ = skill_run_tmp::write_error(ctx, &error_value);
            }
            Err(ToolError::Structured(error_value))
        }
    }
}

/// Lint failures are skipped silently — lint enhances skill_run but must not block it.
fn collect_docx_style_warnings(
    ctx: &ToolContext,
    written_paths: &[String],
) -> Option<Map<String, Value>> {
    let mut style_warnings = Map::new();
    let mut seen = std::collections::HashSet::new();
    for path in written_paths {
        if !seen.insert(path.as_str()) || !is_docx_path(path) {
            continue;
        }
        let Ok(resolved) = ctx.sandbox.resolve(path) else {
            continue;
        };
        let Ok(warnings) = lint_docx(&resolved) else {
            continue;
        };
        if !warnings.is_empty() {
            style_warnings.insert(path.clone(), json!(warnings));
        }
    }
    (!style_warnings.is_empty()).then_some(style_warnings)
}

fn is_docx_path(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("docx"))
}

fn is_office_deliverable(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "docx" | "pptx" | "xlsx" | "xlsm"
            )
        })
}

/// Keep `.cache/skill-run/script.js` for in-turn repair; the turn-end hook in the
/// agent loop removes it once the turn finishes without a pending failure.
fn should_retain_skill_run_script(written_paths: &[String], has_style_warnings: bool) -> bool {
    has_style_warnings || written_paths.iter().any(|path| is_office_deliverable(path))
}

fn script_retain_reason(has_style_warnings: bool, written_paths: &[String]) -> &'static str {
    if has_style_warnings {
        "style_warnings"
    } else if written_paths.iter().any(|path| is_office_deliverable(path)) {
        "deliverable_pending_review"
    } else {
        "repair"
    }
}
