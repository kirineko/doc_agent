pub mod bundled;
pub mod compile;

use compile::{
    compile_to_temp_pdf, finalize_pdf, remove_temp_pdf, resolve_typst_entry, CompileInput,
};
use serde_json::{json, Value};

use super::{ensure_parent_dir, required_str_arg, ToolContext, ToolError, ToolSpec};

const TIMEOUT_SECS: u64 = 60;

pub fn typst_to_pdf_tool() -> ToolSpec {
    ToolSpec {
        name: "typst_to_pdf",
        description: "Compile a Typst (.typ) file or directory with main.typ in the project sandbox to PDF. \
            Prerequisite: typst_read_template syntax/typst-guide once per conversation. \
            New or heavily rewritten documents: also typst_list_templates and typst_read_template (scene) before writing .typ. \
            Recompile-only edits may skip list/scene. Import built-ins via #import \"/doc-agent/typst/...\".",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project-relative .typ file or directory containing main.typ"
                },
                "out_path": {
                    "type": "string",
                    "description": "Output .pdf path in sandbox"
                }
            },
            "required": ["path", "out_path"]
        }),
        handler: typst_to_pdf_stub,
    }
}

pub fn typst_list_templates_tool() -> ToolSpec {
    ToolSpec {
        name: "typst_list_templates",
        description: "List built-in Typst syntax guide and scene templates (report/exam/paper/lecture × zh/en). \
            New-document workflow step 2 (after syntax/typst-guide); pass ids to typst_read_template.",
        parameters: json!({
            "type": "object",
            "properties": {}
        }),
        handler: list_templates_handler,
    }
}

pub fn typst_read_template_tool() -> ToolSpec {
    ToolSpec {
        name: "typst_read_template",
        description: "Read built-in Typst syntax guide (syntax/typst-guide) or scene template source (e.g. report/report-zh). \
            Workflow: step 1 = guide (once per conversation); step 3 = one scene template before writing a new .typ.",
        parameters: json!({
            "type": "object",
            "properties": {
                "template": {
                    "type": "string",
                    "description": "Template id, e.g. report/report-zh, exam/exam-en"
                }
            },
            "required": ["template"]
        }),
        handler: read_template_handler,
    }
}

fn typst_to_pdf_stub(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let _ = prepare_export(ctx, &args)?;
    Err(ToolError::Execution(
        "typst_to_pdf requires async registry path".into(),
    ))
}

pub async fn typst_to_pdf_handler(ctx: &ToolContext<'_>, args: Value) -> Result<Value, ToolError> {
    let (entry, out, out_path, sandbox_root) = prepare_export(ctx, &args)?;
    let (temp_ready_tx, temp_ready_rx) = tokio::sync::oneshot::channel();
    let input = CompileInput {
        sandbox_root,
        entry,
        out_path: out.clone(),
    };

    let join = tokio::task::spawn_blocking(move || {
        let result = compile_to_temp_pdf(input);
        if let Ok(ref output) = result {
            let _ = temp_ready_tx.send(output.temp_path.clone());
        }
        result
    });

    let compile_result =
        match tokio::time::timeout(std::time::Duration::from_secs(TIMEOUT_SECS), join).await {
            Ok(join_result) => join_result
                .map_err(|e| ToolError::Execution(format!("typst 编译线程失败: {e}")))?
                .map_err(ToolError::Execution)?,
            Err(_) => {
                tokio::spawn(async move {
                    if let Ok(temp_path) = temp_ready_rx.await {
                        remove_temp_pdf(&temp_path);
                    }
                });
                return Err(ToolError::Execution(format!(
                    "typst_to_pdf 超时（{TIMEOUT_SECS}s）"
                )));
            }
        };

    if let Err(err) = finalize_pdf(&compile_result.temp_path, &out) {
        remove_temp_pdf(&compile_result.temp_path);
        return Err(ToolError::Execution(err));
    }

    Ok(json!({
        "path": out_path,
        "pages": compile_result.pages
    }))
}

fn list_templates_handler(_ctx: &ToolContext, _args: Value) -> Result<Value, ToolError> {
    let templates: Vec<Value> = bundled::LISTABLE
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "category": m.category,
                "lang": m.lang,
                "title": m.title,
                "description": m.description,
                "import_path": bundled::vpath(m.rel_path),
            })
        })
        .collect();
    Ok(json!({ "templates": templates }))
}

fn read_template_handler(_ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let template = required_str_arg(&args, "template")?;
    let meta = bundled::find_by_id(&template).ok_or_else(|| {
        ToolError::InvalidArgs(format!(
            "unknown template: {template}. Use typst_list_templates for valid ids."
        ))
    })?;
    let source = bundled::find_source(&template)
        .ok_or_else(|| ToolError::Execution(format!("template source missing: {template}")))?;
    Ok(json!({
        "id": meta.id,
        "import_path": bundled::vpath(meta.rel_path),
        "content": source,
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
        std::path::PathBuf,
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
    let entry = resolve_typst_entry(&resolved).map_err(ToolError::Execution)?;

    let out = ctx
        .sandbox
        .resolve_for_write(&out_path)
        .map_err(ToolError::Sandbox)?;
    ensure_parent_dir(&out)?;

    Ok((entry, out, out_path, ctx.sandbox.root().to_path_buf()))
}
