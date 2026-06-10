use super::{required_str_arg, ToolContext, ToolError, ToolSpec};
use crate::core::skills;
use crate::tools::runtime;
use serde_json::{json, Value};
use std::time::Duration;

pub fn read_tool() -> ToolSpec {
    ToolSpec {
        name: "skill_read",
        description: "Read a built-in Document Skill guide. \
            skill MUST be one of: docx, pdf, pptx, xlsx. \
            doc is optional (default SKILL.md). \
            docx template editing: doc=editing.md. pptx API: doc=pptxgenjs.md; pptx template: skill=pptx, doc=editing.md. \
            Filenames like pptxgenjs.md alone work only when unique; editing.md requires skill=docx or skill=pptx.",
        parameters: json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "string",
                    "description": "Skill id: docx, pdf, pptx, xlsx. Filenames like pptxgenjs.md are auto-resolved to skill=pptx."
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
            Define async function main() returning JSON-serializable value; do NOT call main() at end. \
            Libraries (auto-loaded): ExcelJS, PptxGenJS, PDFLib, docx — use globals OR require('exceljs') etc. \
            Save files: await wb.xlsx.writeFile('out.xlsx') (shimmed), doc_write(path, base64), doc_write_bytes(path, bytes). \
            Buffer.from(buf).toString('base64') and fs.writeFileSync are shimmed. No fetch/npm/shell.",
        parameters: json!({
            "type": "object",
            "properties": {
                "code": { "type": "string" },
                "timeout_secs": { "type": "integer", "default": 30 }
            },
            "required": ["code"]
        }),
        handler: run_handler,
    }
}

fn read_handler(_ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let skill = required_str_arg(&args, "skill")?;
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

fn run_handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let code = required_str_arg(&args, "code")?;
    let timeout_secs = args
        .get("timeout_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(30)
        .clamp(1, 120);
    let result = runtime::execute_script(ctx.sandbox, &code, Duration::from_secs(timeout_secs))?;
    Ok(json!({ "result": result }))
}
