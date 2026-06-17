use crate::core::cache_paths::{ooxml_work_dir, skill_run_dir};
use crate::core::file_locks::{normalize_project_path, FileResource, LockMode, LockRequest};
use crate::tools::office::legacy_target_extension;
use crate::tools::{ToolContext, ToolError};
use serde_json::Value;
use std::path::Path;

pub struct ToolIoPlan {
    pub locks: Vec<LockRequest>,
    pub dynamic_writes: bool,
}

pub fn plan_tool_io(
    ctx: &ToolContext<'_>,
    tool_name: &str,
    args: &Value,
) -> Result<ToolIoPlan, ToolError> {
    let mut plan = ToolIoPlan {
        locks: Vec::new(),
        dynamic_writes: false,
    };
    match tool_name {
        "fs_read"
        | "office_read_to_markdown"
        | "excel_read"
        | "pdf_read"
        | "pdf_render_pages"
        | "typst_read_template" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
        }
        "fs_write" | "excel_write" | "xlsx_recalc" => {
            add_write_arg(ctx, args, "path", &mut plan)?;
        }
        "fs_patch" => {
            add_write_arg(ctx, args, "path", &mut plan)?;
        }
        // fs_list 仅枚举目录条目（只读快照），无数据竞争一致性要求；
        // 若对根目录 `.` 申请 Read 锁会与任意 SubtreeWrite（如 skill_run scratch）冲突，
        // 错误地阻塞同项目并行，因此不申请文件锁。
        "fs_list"
        | "fs_search"
        | "skill_read"
        | "clarify_ask"
        | "web_search"
        | "web_extract"
        | "typst_list_templates" => {}
        "office_convert" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
            plan_office_convert_write(ctx, args, &mut plan)?;
        }
        "ooxml_unpack" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
            plan_ooxml_unpack_out(ctx, args, &mut plan)?;
        }
        "ooxml_pack" => {
            // pack 读取整个解包子树压缩回包；用 SubtreeWrite 排他持有该子树，
            // 防止同项目并发会话在打包期间写入子文件导致打包到半写 XML。
            add_subtree_write_arg(ctx, args, "dir", &mut plan)?;
            add_write_arg(ctx, args, "out_path", &mut plan)?;
            add_optional_read_arg(ctx, args, "original", &mut plan)?;
        }
        "docx_comment" => {
            add_subtree_write_arg(ctx, args, "dir", &mut plan)?;
        }
        "docx_accept_changes" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
            if args.get("out_path").and_then(|v| v.as_str()).is_some() {
                add_write_arg(ctx, args, "out_path", &mut plan)?;
            } else {
                add_write_arg(ctx, args, "path", &mut plan)?;
            }
        }
        "docx_extract_table" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
            add_subtree_write_arg(ctx, args, "out_dir", &mut plan)?;
        }
        "excel_describe" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
        }
        "excel_normalize" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
            add_write_arg(ctx, args, "out_path", &mut plan)?;
        }
        "data_query" => {
            plan_data_query(ctx, args, &mut plan)?;
        }
        "pdf_merge" => {
            add_read_paths_array(ctx, args, "inputs", &mut plan)?;
            add_write_arg(ctx, args, "out_path", &mut plan)?;
        }
        "pdf_split" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
            if args.get("mode").and_then(|v| v.as_str()) == Some("burst") {
                add_subtree_write_arg(ctx, args, "out_dir", &mut plan)?;
            } else {
                add_write_arg(ctx, args, "out_path", &mut plan)?;
            }
        }
        "pdf_rotate" | "pdf_delete_pages" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
            add_write_arg(ctx, args, "out_path", &mut plan)?;
        }
        "html_to_pdf" | "typst_to_pdf" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
            add_write_arg(ctx, args, "out_path", &mut plan)?;
        }
        "skill_run" => {
            plan.dynamic_writes = true;
            plan_skill_run(ctx, args, &mut plan)?;
        }
        "image_read" => {
            add_read_paths_array(ctx, args, "paths", &mut plan)?;
        }
        other => {
            return Err(ToolError::Execution(format!(
                "io_plan missing for tool: {other}"
            )));
        }
    }
    Ok(plan)
}

#[cfg(test)]
fn minimal_args_for(tool_name: &str) -> Value {
    match tool_name {
        "fs_list"
        | "fs_read"
        | "office_read_to_markdown"
        | "excel_read"
        | "pdf_read"
        | "pdf_render_pages"
        | "typst_read_template"
        | "excel_describe"
        | "xlsx_recalc" => {
            serde_json::json!({ "path": "a.txt" })
        }
        "fs_write" | "excel_write" => serde_json::json!({ "path": "a.txt", "content": "x" }),
        "fs_patch" => serde_json::json!({ "path": "a.txt", "edits": [] }),
        "office_convert" => serde_json::json!({ "path": "legacy.doc" }),
        "ooxml_unpack" => serde_json::json!({ "path": "a.docx" }),
        "ooxml_pack" => serde_json::json!({ "dir": "unpacked", "out_path": "out.docx" }),
        "docx_comment" => serde_json::json!({ "dir": "unpacked", "id": 1, "text": "x" }),
        "docx_accept_changes" => serde_json::json!({ "path": "a.docx" }),
        "docx_extract_table" => serde_json::json!({ "path": "a.docx", "out_dir": "tables" }),
        "excel_normalize" => serde_json::json!({ "path": "a.xlsx", "out_path": "out.csv" }),
        "data_query" => {
            serde_json::json!({ "sources": [{ "name": "t", "path": "a.csv" }], "sql": "select 1" })
        }
        "pdf_merge" => serde_json::json!({ "inputs": ["a.pdf"], "out_path": "out.pdf" }),
        "pdf_split" => serde_json::json!({ "path": "a.pdf", "out_path": "out.pdf" }),
        "pdf_rotate" | "pdf_delete_pages" => {
            serde_json::json!({ "path": "a.pdf", "out_path": "out.pdf" })
        }
        "html_to_pdf" | "typst_to_pdf" => {
            serde_json::json!({ "path": "a.typ", "out_path": "out.pdf" })
        }
        "skill_run" => serde_json::json!({ "code": "async function main(){}" }),
        "image_read" => serde_json::json!({ "paths": [".cache/attachments/x.png"] }),
        _ => serde_json::json!({}),
    }
}

fn plan_office_convert_write(
    ctx: &ToolContext<'_>,
    args: &Value,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    if args.get("out_path").and_then(|v| v.as_str()).is_some() {
        return add_write_arg(ctx, args, "out_path", plan);
    }
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    let src_ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let target_ext = legacy_target_extension(src_ext).ok_or_else(|| {
        ToolError::InvalidArgs(format!("unsupported convert source extension: .{src_ext}"))
    })?;
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ToolError::InvalidArgs("source has no filename".into()))?;
    let out_rel = if path.contains('/') {
        format!(
            "{}/{}-converted.{}",
            Path::new(path).parent().unwrap().to_string_lossy(),
            stem,
            target_ext
        )
    } else {
        format!("{stem}-converted.{target_ext}")
    };
    add_write_path(ctx, &out_rel, plan)
}

fn plan_ooxml_unpack_out(
    ctx: &ToolContext<'_>,
    args: &Value,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    if let Some(out) = args.get("out_dir").and_then(|v| v.as_str()) {
        return add_subtree_write_path(ctx, out, plan);
    }
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("path required".into()))?;
    if ctx.session_id.is_empty() || ctx.turn_id.is_empty() {
        return Err(ToolError::Execution(
            "ooxml_unpack without out_dir requires turn context".into(),
        ));
    }
    let generated = ooxml_work_dir(ctx.session_id, ctx.turn_id, path);
    add_subtree_write_path(ctx, &generated, plan)
}

fn plan_data_query(
    ctx: &ToolContext<'_>,
    args: &Value,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    let sources = args
        .get("sources")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::InvalidArgs("sources required".into()))?;
    for source in sources {
        if let Some(path) = source.get("path").and_then(|v| v.as_str()) {
            add_read_path(ctx, path, plan)?;
        }
    }
    let out = args
        .get("out_path")
        .and_then(|v| v.as_str())
        .unwrap_or("query_result.csv");
    add_write_path(ctx, out, plan)
}

fn plan_skill_run(
    ctx: &ToolContext<'_>,
    args: &Value,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    if ctx.session_id.is_empty() || ctx.turn_id.is_empty() {
        return Err(ToolError::Execution(
            "skill_run requires turn context for scratch workspace".into(),
        ));
    }
    let scratch = skill_run_dir(ctx.session_id);
    add_subtree_write_path(ctx, &scratch, plan)?;
    if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
        add_read_path(ctx, path, plan)?;
    }
    Ok(())
}

fn add_read_arg(
    ctx: &ToolContext<'_>,
    args: &Value,
    key: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    let path = args
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs(format!("{key} required")))?;
    add_read_path(ctx, path, plan)
}

fn add_write_arg(
    ctx: &ToolContext<'_>,
    args: &Value,
    key: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    let path = args
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs(format!("{key} required")))?;
    add_write_path(ctx, path, plan)
}

fn add_subtree_write_arg(
    ctx: &ToolContext<'_>,
    args: &Value,
    key: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    let path = args
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs(format!("{key} required")))?;
    add_subtree_write_path(ctx, path, plan)
}

fn add_optional_read_arg(
    ctx: &ToolContext<'_>,
    args: &Value,
    key: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    if let Some(path) = args.get(key).and_then(|v| v.as_str()) {
        add_read_path(ctx, path, plan)?;
    }
    Ok(())
}

fn add_read_paths_array(
    ctx: &ToolContext<'_>,
    args: &Value,
    key: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    let items = args
        .get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::InvalidArgs(format!("{key} required")))?;
    for item in items {
        if let Some(path) = item.as_str() {
            add_read_path(ctx, path, plan)?;
        }
    }
    Ok(())
}

fn add_read_path(
    ctx: &ToolContext<'_>,
    user_path: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    push_lock(ctx, user_path, LockMode::Read, plan)
}

fn add_write_path(
    ctx: &ToolContext<'_>,
    user_path: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    push_lock(ctx, user_path, LockMode::Write, plan)
}

fn add_subtree_write_path(
    ctx: &ToolContext<'_>,
    user_path: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    push_lock(ctx, user_path, LockMode::SubtreeWrite, plan)
}

fn push_lock(
    ctx: &ToolContext<'_>,
    user_path: &str,
    mode: LockMode,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    let path = normalize_project_path(ctx.sandbox, user_path).map_err(ToolError::Sandbox)?;
    plan.locks.push(LockRequest {
        resource: FileResource {
            project_id: ctx.project_id.to_string(),
            path,
        },
        mode,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn ctx<'a>(sandbox: &'a Sandbox) -> ToolContext<'a> {
        ToolContext::with_test_turn(sandbox, "p1", "s1", "t1", "Test")
    }

    fn sandbox_with_files() -> (tempfile::TempDir, Sandbox) {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.docx"), b"x").unwrap();
        fs::write(dir.path().join("a.pdf"), b"x").unwrap();
        fs::write(dir.path().join("legacy.doc"), b"x").unwrap();
        fs::write(dir.path().join("a.csv"), b"x").unwrap();
        let sb = Sandbox::new(dir.path()).unwrap();
        (dir, sb)
    }

    #[test]
    fn pdf_read_only_source_read_lock() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(&ctx, "pdf_read", &json!({ "path": "a.pdf" })).unwrap();
        assert_eq!(plan.locks.len(), 1);
        assert_eq!(plan.locks[0].mode, LockMode::Read);
        assert!(!plan.dynamic_writes);
    }

    #[test]
    fn ooxml_unpack_without_out_dir_generates_subtree_lock() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(&ctx, "ooxml_unpack", &json!({ "path": "a.docx" })).unwrap();
        assert_eq!(plan.locks.len(), 2);
        assert!(plan.locks.iter().any(|l| l.mode == LockMode::SubtreeWrite));
    }

    #[test]
    fn fs_list_requests_no_file_lock() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(&ctx, "fs_list", &json!({ "path": "." })).unwrap();
        assert!(plan.locks.is_empty());
        assert!(!plan.dynamic_writes);
    }

    #[test]
    fn ooxml_pack_dir_uses_subtree_write_lock() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(
            &ctx,
            "ooxml_pack",
            &json!({ "dir": "unpacked", "out_path": "out.docx" }),
        )
        .unwrap();
        assert!(plan
            .locks
            .iter()
            .any(|l| l.mode == LockMode::SubtreeWrite && l.resource.path == "unpacked"));
    }

    #[test]
    fn skill_run_marks_dynamic_and_locks_scratch() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(
            &ctx,
            "skill_run",
            &json!({ "code": "async function main(){}" }),
        )
        .unwrap();
        assert!(plan.dynamic_writes);
        assert!(plan
            .locks
            .iter()
            .any(|l| l.mode == LockMode::SubtreeWrite && l.resource.path.contains("skill-run")));
    }

    #[test]
    fn all_default_tools_have_io_plan() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"x").unwrap();
        fs::write(dir.path().join("a.docx"), b"x").unwrap();
        fs::write(dir.path().join("a.pdf"), b"x").unwrap();
        fs::write(dir.path().join("legacy.doc"), b"x").unwrap();
        fs::write(dir.path().join("a.csv"), b"x").unwrap();
        fs::write(dir.path().join("a.xlsx"), b"x").unwrap();
        fs::write(dir.path().join("a.typ"), b"x").unwrap();
        fs::create_dir_all(dir.path().join(".cache/attachments")).unwrap();
        fs::write(dir.path().join(".cache/attachments/x.png"), b"x").unwrap();
        let sb = Sandbox::new(dir.path()).unwrap();
        let ctx = ctx(&sb);
        for tool_name in crate::tools::ToolRegistry::default_tools().tool_names() {
            let args = minimal_args_for(tool_name);
            plan_tool_io(&ctx, tool_name, &args)
                .unwrap_or_else(|e| panic!("io_plan failed for {tool_name}: {e}"));
        }
    }
}
