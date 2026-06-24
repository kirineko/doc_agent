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
        "fs_read" | "office_read_to_markdown" | "excel_read" | "pdf_read" | "pdf_render_pages" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
        }
        "fs_write" | "excel_write" => {
            add_write_arg(ctx, args, "path", &mut plan)?;
        }
        "xlsx_recalc" => {
            add_read_arg(ctx, args, "path", &mut plan)?;
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
        | "typst_list_templates"
        | "typst_read_template" => {}
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
                add_optional_subtree_write_arg(ctx, args, "out_dir", &mut plan)?;
            } else {
                add_optional_write_arg(ctx, args, "out_path", &mut plan)?;
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
        "image_download" => {
            // 输出目录（缺省 images）整体排他写，避免同项目并发会话写花同一目录。
            // 外部图片 URL 不参与文件锁——它们是网络资源，不是沙箱路径。
            // 与 handler 共用 normalize_output_dir，确保两侧对 dir（trim/空串/
            // 越界/项目根/.cache）的处理一致。
            let dir = crate::tools::image_download::normalize_output_dir(
                args.get("dir").and_then(|v| v.as_str()),
            )?;
            add_subtree_write_path(ctx, &dir, &mut plan)?;
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
        | "excel_describe"
        | "xlsx_recalc" => {
            serde_json::json!({ "path": "a.txt" })
        }
        "typst_read_template" => serde_json::json!({ "template": "syntax/typst-guide" }),
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
        "image_download" => {
            serde_json::json!({ "urls": ["https://example.com/a.png"], "dir": "images" })
        }
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
    let target_ext = match legacy_target_extension(src_ext) {
        Some(ext) => ext,
        None => return Ok(()),
    };
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

fn add_optional_write_arg(
    ctx: &ToolContext<'_>,
    args: &Value,
    key: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    if let Some(path) = args.get(key).and_then(|v| v.as_str()) {
        add_write_path(ctx, path, plan)?;
    }
    Ok(())
}

fn add_optional_subtree_write_arg(
    ctx: &ToolContext<'_>,
    args: &Value,
    key: &str,
    plan: &mut ToolIoPlan,
) -> Result<(), ToolError> {
    if let Some(path) = args.get(key).and_then(|v| v.as_str()) {
        add_subtree_write_path(ctx, path, plan)?;
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
    fn image_download_uses_subtree_write_on_dir() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(
            &ctx,
            "image_download",
            &json!({ "urls": ["https://example.com/a.png"] }),
        )
        .unwrap();
        // 缺省 dir 为 images，整体排他写；URL 不参与文件锁
        assert_eq!(plan.locks.len(), 1);
        assert_eq!(plan.locks[0].mode, LockMode::SubtreeWrite);
        assert_eq!(plan.locks[0].resource.path, "images");
        assert!(!plan.dynamic_writes);
    }

    #[test]
    fn image_download_normalizes_and_rejects_bad_dir_consistently() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        // 空白/空串归一到 images（与 handler 一致，不再因 sandbox 路径非法而失败）
        let plan = plan_tool_io(
            &ctx,
            "image_download",
            &json!({ "urls": ["https://example.com/a.png"], "dir": "   " }),
        )
        .unwrap();
        assert_eq!(plan.locks[0].resource.path, "images");
        // 项目根、.cache、越界路径在 io_plan 阶段即拒绝（与 handler 同口径）
        for bad in [".", ".cache", ".cache/imgs", "../escape"] {
            assert!(
                plan_tool_io(
                    &ctx,
                    "image_download",
                    &json!({ "urls": ["https://example.com/a.png"], "dir": bad }),
                )
                .is_err(),
                "dir {bad:?} should be rejected in io_plan"
            );
        }
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

    fn schema_required_args(tool_name: &str, parameters: &Value) -> Value {
        let mut obj = serde_json::Map::new();
        let Some(required) = parameters.get("required").and_then(|v| v.as_array()) else {
            if tool_name == "skill_run" {
                obj.insert("code".into(), json!("async function main() {}"));
            }
            return Value::Object(obj);
        };
        for key in required {
            let Some(key) = key.as_str() else {
                continue;
            };
            if let Some(value) = dummy_arg_value(tool_name, key) {
                obj.insert(key.to_string(), value);
            }
        }
        Value::Object(obj)
    }

    fn dummy_arg_value(tool_name: &str, key: &str) -> Option<Value> {
        Some(match key {
            "path" => json!(match tool_name {
                "office_convert" => "legacy.doc",
                "pdf_read" | "pdf_render_pages" | "pdf_split" | "pdf_rotate"
                | "pdf_delete_pages" => {
                    "a.pdf"
                }
                "ooxml_unpack" | "docx_accept_changes" => "a.docx",
                "excel_read" | "excel_describe" | "excel_normalize" | "xlsx_recalc" => "a.xlsx",
                "html_to_pdf" | "typst_to_pdf" => "a.typ",
                _ => "a.txt",
            }),
            "template" => json!("syntax/typst-guide"),
            "dir" => json!("unpacked"),
            "out_dir" => json!("tables"),
            "out_path" => json!(match tool_name {
                "html_to_pdf" | "typst_to_pdf" | "pdf_merge" | "pdf_rotate"
                | "pdf_delete_pages" => {
                    "out.pdf"
                }
                "excel_normalize" => "out.csv",
                _ => "out.docx",
            }),
            "skill" => json!("runtime"),
            "code" => json!("async function main() {}"),
            "query" => json!("test"),
            "urls" => json!(["https://example.com"]),
            "inputs" => json!(["a.pdf"]),
            "paths" => json!([".cache/attachments/x.png"]),
            "sources" => json!([{ "name": "t", "path": "a.csv" }]),
            "sql" => json!("select 1"),
            "id" => json!("q1"),
            "kind" => json!("text"),
            "prompt" => json!("test?"),
            "rotation" => json!(90),
            "pages" => json!([1]),
            "cells" => json!([{ "cell": "A1", "value": "x" }]),
            "content" => json!("x"),
            "edits" => json!([]),
            _ => json!("x"),
        })
    }

    #[test]
    fn xlsx_recalc_uses_read_lock_only() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.xlsx"), b"x").unwrap();
        let sb = Sandbox::new(dir.path()).unwrap();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(&ctx, "xlsx_recalc", &json!({ "path": "a.xlsx" })).unwrap();
        assert_eq!(plan.locks.len(), 1);
        assert_eq!(plan.locks[0].mode, LockMode::Read);
    }

    #[test]
    fn office_convert_unsupported_ext_only_read_lock_at_io_plan() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("report.docx"), b"x").unwrap();
        let sb = Sandbox::new(dir.path()).unwrap();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(&ctx, "office_convert", &json!({ "path": "report.docx" })).unwrap();
        assert_eq!(plan.locks.len(), 1);
        assert_eq!(plan.locks[0].mode, LockMode::Read);
    }

    #[test]
    fn skill_run_path_only_oneof_branch_io_plan() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(
            &ctx,
            "skill_run",
            &json!({ "path": ".cache/skill-run/test/script.js" }),
        )
        .unwrap();
        assert!(plan.dynamic_writes);
        assert!(plan.locks.iter().any(|l| l.mode == LockMode::Read));
    }

    #[test]
    fn typst_read_template_io_plan_uses_template_not_path() {
        let dir = tempdir().unwrap();
        let sb = Sandbox::new(dir.path()).unwrap();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(
            &ctx,
            "typst_read_template",
            &serde_json::json!({ "template": "syntax/typst-guide" }),
        )
        .unwrap();
        assert!(plan.locks.is_empty());
    }

    #[test]
    fn pdf_split_path_only_does_not_require_out_path_at_io_plan() {
        let (_dir, sb) = sandbox_with_files();
        let ctx = ctx(&sb);
        let plan = plan_tool_io(&ctx, "pdf_split", &json!({ "path": "a.pdf" })).unwrap();
        assert_eq!(plan.locks.len(), 1);
        assert_eq!(plan.locks[0].mode, LockMode::Read);
    }

    #[test]
    fn io_plan_accepts_schema_required_args_only() {
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
        for def in crate::tools::ToolRegistry::default_tools().definitions(true) {
            let args = schema_required_args(&def.name, &def.parameters);
            plan_tool_io(&ctx, &def.name, &args).unwrap_or_else(|e| {
                panic!(
                    "io_plan failed for {} with schema-required args {args}: {e}",
                    def.name
                )
            });
        }
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
