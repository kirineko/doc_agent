//! Integration tests for parallel file governance (locks, session/work scratch paths, conflicts).

#[cfg(test)]
mod tests {
    use super::super::{
        io_plan::plan_tool_io, runtime::write_gate::RuntimeWriteGate, ToolContext, ToolError,
        ToolRegistry,
    };
    use crate::agent::types::ModelId;
    use crate::core::cache_paths::{ooxml_work_dir, skill_run_script};
    use crate::core::file_locks::{FileLockRegistry, TurnFileLockStore};
    use crate::core::sandbox::Sandbox;
    use serde_json::{json, Value};
    use std::fs;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn setup() -> (
        tempfile::TempDir,
        Sandbox,
        Arc<FileLockRegistry>,
        ToolRegistry,
    ) {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let registry = ToolRegistry::default_tools();
        let bootstrap = ToolContext::new(&sandbox);
        create_docx(&bootstrap, &registry, "a.docx");
        create_docx(&bootstrap, &registry, "b.docx");
        fs::write(dir.path().join("sample.pdf"), b"%PDF-1.4 minimal\n").unwrap();
        fs::write(dir.path().join("sample.typ"), b"#title\nHello").unwrap();
        (dir, sandbox, Arc::new(FileLockRegistry::new()), registry)
    }

    fn create_docx(ctx: &ToolContext, registry: &ToolRegistry, path: &str) {
        let code = format!(
            r#"
async function main() {{
  const {{ Document, Packer, Paragraph, TextRun }} = docx;
  const doc = new Document({{
    sections: [{{ children: [new Paragraph({{ children: [new TextRun("test")] }})] }}],
  }});
  doc_write("{path}", await Packer.toBase64String(doc));
  return {{ ok: true }};
}}
"#
        );
        exec_tool(
            registry,
            ctx,
            "skill_run",
            json!({ "code": code, "timeout_secs": 60 }),
        )
        .unwrap();
    }

    fn turn_ctx<'a>(
        sandbox: &'a Sandbox,
        reg: Arc<FileLockRegistry>,
        session: &'a str,
        turn: &'a str,
    ) -> ToolContext<'a> {
        ToolContext::for_turn(
            sandbox, None, "p1", session, turn, session, reg, None, false, false,
        )
    }

    fn exec_tool(
        registry: &ToolRegistry,
        ctx: &ToolContext,
        name: &str,
        args: Value,
    ) -> Result<Value, ToolError> {
        let app = tauri::test::mock_app();
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(registry.execute(ctx, app.handle(), ModelId::Mock, name, args))
    }

    fn exec_with_locks(
        registry: &ToolRegistry,
        ctx: &ToolContext,
        reg: &Arc<FileLockRegistry>,
        name: &str,
        args: Value,
    ) -> Result<Value, String> {
        exec_with_turn_locks(registry, ctx, reg, &TurnFileLockStore::new(), name, args)
    }

    fn exec_with_turn_locks(
        registry: &ToolRegistry,
        ctx: &ToolContext,
        reg: &Arc<FileLockRegistry>,
        turn_locks: &TurnFileLockStore,
        name: &str,
        args: Value,
    ) -> Result<Value, String> {
        let plan = plan_tool_io(ctx, name, &args).map_err(|e| e.to_string())?;
        let guard = reg
            .acquire_many(
                ctx.project_id,
                ctx.session_id,
                ctx.turn_id,
                ctx.session_title,
                plan.locks,
            )
            .map_err(|e| e.to_tool_json().to_string())?;
        turn_locks
            .hold(guard)
            .map_err(|e| format!("turn lock hold failed: {e}"))?;
        let write_gate = if plan.dynamic_writes {
            Some(Arc::new(RuntimeWriteGate::new(
                reg.clone(),
                turn_locks.clone(),
                ctx.sandbox,
                ctx.project_id.to_string(),
                ctx.session_id.to_string(),
                ctx.turn_id.to_string(),
                ctx.session_title.to_string(),
                ctx.profile_init,
                ctx.agents_md_confirmed,
            )))
        } else {
            None
        };
        let exec_ctx = ToolContext::for_turn(
            ctx.sandbox,
            ctx.secrets,
            ctx.project_id,
            ctx.session_id,
            ctx.turn_id,
            ctx.session_title,
            reg.clone(),
            write_gate,
            ctx.profile_init,
            ctx.agents_md_confirmed,
        );
        exec_tool(registry, &exec_ctx, name, args).map_err(|e| e.to_json_value().to_string())
    }

    #[test]
    fn skill_run_inline_uses_distinct_session_scoped_script_paths() {
        use crate::tools::skill_run_tmp;
        let (_dir, sandbox, reg, _registry) = setup();
        let ctx_a = turn_ctx(&sandbox, reg.clone(), "session-a", "turn-a");
        let ctx_b = turn_ctx(&sandbox, reg, "session-b", "turn-b");
        let code = "async function main() { return { ok: true }; }";

        skill_run_tmp::write_temp_script(&ctx_a, code).unwrap();
        skill_run_tmp::write_temp_script(&ctx_b, code).unwrap();

        let expected_a = skill_run_script("session-a");
        let expected_b = skill_run_script("session-b");
        assert_eq!(skill_run_tmp::script_rel(&ctx_a).unwrap(), expected_a);
        assert_eq!(skill_run_tmp::script_rel(&ctx_b).unwrap(), expected_b);
        assert_ne!(expected_a, expected_b);
        assert!(_dir.path().join(&expected_a).exists());
        assert!(_dir.path().join(&expected_b).exists());
    }

    #[test]
    fn runtime_write_gate_rejects_cross_session_same_output() {
        let (_dir, sandbox, reg, _registry) = setup();
        let gate_a = RuntimeWriteGate::new(
            reg.clone(),
            TurnFileLockStore::new(),
            &sandbox,
            "p1".into(),
            "s1".into(),
            "t1".into(),
            "A".into(),
            false,
            false,
        );
        let gate_b = RuntimeWriteGate::new(
            reg,
            TurnFileLockStore::new(),
            &sandbox,
            "p1".into(),
            "s2".into(),
            "t2".into(),
            "B".into(),
            false,
            false,
        );
        gate_a.before_write("shared/out.txt").unwrap();
        let err = gate_b.before_write("shared/out.txt").unwrap_err();
        assert!(err.contains("shared/out.txt") || err.contains("shared"));
    }

    #[test]
    fn runtime_write_gate_reuses_lock_for_same_path_in_one_turn() {
        let (_dir, sandbox, reg, _registry) = setup();
        let gate = RuntimeWriteGate::new(
            reg,
            TurnFileLockStore::new(),
            &sandbox,
            "p1".into(),
            "s1".into(),
            "t1".into(),
            "A".into(),
            false,
            false,
        );
        gate.before_write("out.txt").unwrap();
        gate.before_write("out.txt").unwrap();
    }

    #[test]
    fn ooxml_auto_work_dirs_do_not_conflict() {
        let (_dir, sandbox, reg, registry) = setup();
        let ctx_a = turn_ctx(&sandbox, reg.clone(), "s-a", "t-a");
        let ctx_b = turn_ctx(&sandbox, reg.clone(), "s-b", "t-b");

        let dir_a = ooxml_work_dir("s-a", "t-a", "a.docx");
        let dir_b = ooxml_work_dir("s-b", "t-b", "b.docx");
        assert_ne!(dir_a, dir_b);

        let out_a = exec_with_locks(
            &registry,
            &ctx_a,
            &reg,
            "ooxml_unpack",
            json!({ "path": "a.docx" }),
        )
        .unwrap();
        let out_b = exec_with_locks(
            &registry,
            &ctx_b,
            &reg,
            "ooxml_unpack",
            json!({ "path": "b.docx" }),
        )
        .unwrap();
        assert_eq!(out_a["out_dir"].as_str().unwrap(), dir_a);
        assert_eq!(out_b["out_dir"].as_str().unwrap(), dir_b);
    }

    #[test]
    fn explicit_out_dir_lock_survives_between_tool_calls_in_same_turn() {
        // TurnFileLockStore 使 unpack 的 SubtreeWrite 跨 execute_one 持有；
        // 模拟 unpack →（gap）→ 另一会话再 unpack 同一显式 out_dir 必须 file_busy。
        let (_dir, sandbox, reg, registry) = setup();
        let ctx_a = turn_ctx(&sandbox, reg.clone(), "s-a", "t-a");
        let ctx_b = turn_ctx(&sandbox, reg.clone(), "s-b", "t-b");
        let turn_locks_a = TurnFileLockStore::new();

        exec_with_turn_locks(
            &registry,
            &ctx_a,
            &reg,
            &turn_locks_a,
            "ooxml_unpack",
            json!({ "path": "a.docx", "out_dir": "unpacked" }),
        )
        .expect("first unpack should succeed");
        assert_eq!(turn_locks_a.guard_count(), 1);

        let blocked = exec_with_locks(
            &registry,
            &ctx_b,
            &reg,
            "ooxml_unpack",
            json!({ "path": "b.docx", "out_dir": "unpacked" }),
        )
        .expect_err("second session must be blocked while first turn still holds subtree lock");
        assert!(
            blocked.contains("file_busy") || blocked.contains("unpacked"),
            "expected file_busy on shared out_dir: {blocked}"
        );

        drop(turn_locks_a);
        exec_with_locks(
            &registry,
            &ctx_b,
            &reg,
            "ooxml_unpack",
            json!({ "path": "b.docx", "out_dir": "unpacked" }),
        )
        .expect("lock should release when first turn context drops");
    }

    #[test]
    fn explicit_unpacked_out_dir_second_session_file_busy() {
        let (dir, sandbox, reg, _registry) = setup();
        let ctx_a = turn_ctx(&sandbox, reg.clone(), "s-a", "t-a");
        let ctx_b = turn_ctx(&sandbox, reg.clone(), "s-b", "t-b");
        let args_a = json!({ "path": "a.docx", "out_dir": "unpacked" });
        let args_b = json!({ "path": "b.docx", "out_dir": "unpacked" });

        let plan_a = plan_tool_io(&ctx_a, "ooxml_unpack", &args_a).unwrap();
        let _guard_a = reg
            .acquire_many("p1", "s-a", "t-a", "A", plan_a.locks)
            .unwrap();
        fs::create_dir_all(dir.path().join("unpacked/word")).unwrap();
        fs::write(
            dir.path().join("unpacked/word/document.xml"),
            b"<w:document/>",
        )
        .unwrap();

        let plan_b = plan_tool_io(&ctx_b, "ooxml_unpack", &args_b).unwrap();
        let blocked = reg
            .acquire_many("p1", "s-b", "t-b", "B", plan_b.locks)
            .unwrap_err();
        assert!(
            blocked.message.contains("unpacked"),
            "expected unpacked conflict: {}",
            blocked.message
        );
        assert!(
            dir.path().join("unpacked/word/document.xml").exists(),
            "first session unpacked dir must survive second session lock failure"
        );
    }

    #[test]
    fn docx_accept_changes_in_place_does_not_self_conflict() {
        // 省略 out_path 时 docx_accept_changes 对同一文件申请 Read+Write；同一批锁的冲突检查
        // 仅比对外部已持有锁，批内不互相检查，因此单会话就地接受修订不应 file_busy。
        let (_dir, sandbox, reg, _registry) = setup();
        let ctx = turn_ctx(&sandbox, reg.clone(), "s-a", "t-a");
        let plan = plan_tool_io(&ctx, "docx_accept_changes", &json!({ "path": "a.docx" })).unwrap();
        let _guard = reg
            .acquire_many("p1", "s-a", "t-a", "A", plan.locks)
            .expect("in-place accept_changes (Read+Write same path) must not self-conflict");
    }

    #[test]
    fn repeat_generated_unpack_same_path_is_rejected() {
        // 省略 out_dir 时生成目录由 session+turn+path 确定；同一轮对同一文档重复解包
        // 必须被拒，否则会 remove_dir_all 静默删除该轮已编辑的 XML。
        let (_dir, sandbox, reg, registry) = setup();
        let ctx = turn_ctx(&sandbox, reg.clone(), "s-a", "t-a");

        exec_with_locks(
            &registry,
            &ctx,
            &reg,
            "ooxml_unpack",
            json!({ "path": "a.docx" }),
        )
        .expect("first unpack should succeed");

        let err = exec_with_locks(
            &registry,
            &ctx,
            &reg,
            "ooxml_unpack",
            json!({ "path": "a.docx" }),
        )
        .expect_err("repeat unpack of same path in same turn must be rejected");
        assert!(
            err.contains("out_dir") || err.contains("解包"),
            "expected guidance to reuse out_dir: {err}"
        );
    }

    #[test]
    fn fs_list_not_blocked_by_concurrent_subtree_write() {
        // 一个会话持有 skill_run scratch 的 SubtreeWrite 时，另一会话 fs_list(".") 必须仍可执行，
        // 否则同项目并行会被无关的工作区子树锁阻塞。
        let (_dir, sandbox, reg, _registry) = setup();
        let ctx_a = turn_ctx(&sandbox, reg.clone(), "s-a", "t-a");
        let ctx_b = turn_ctx(&sandbox, reg.clone(), "s-b", "t-b");

        let plan_a = plan_tool_io(
            &ctx_a,
            "skill_run",
            &json!({ "code": "async function main(){}" }),
        )
        .unwrap();
        let _guard_a = reg
            .acquire_many("p1", "s-a", "t-a", "A", plan_a.locks)
            .unwrap();

        let plan_b = plan_tool_io(&ctx_b, "fs_list", &json!({ "path": "." })).unwrap();
        assert!(plan_b.locks.is_empty());
        let _guard_b = reg
            .acquire_many("p1", "s-b", "t-b", "B", plan_b.locks)
            .expect("fs_list must not be blocked by unrelated subtree write");
    }

    #[test]
    fn ooxml_pack_subtree_lock_blocks_concurrent_child_write() {
        // pack 持有 dir 的 SubtreeWrite，期间另一会话对 dir 子文件的 fs_write 必须 file_busy，
        // 否则会在打包读取整树时混入半写 XML。
        let (_dir, sandbox, reg, _registry) = setup();
        let ctx_a = turn_ctx(&sandbox, reg.clone(), "s-a", "t-a");
        let ctx_b = turn_ctx(&sandbox, reg.clone(), "s-b", "t-b");

        let plan_a = plan_tool_io(
            &ctx_a,
            "ooxml_pack",
            &json!({ "dir": "unpacked", "out_path": "out.docx" }),
        )
        .unwrap();
        let _guard_a = reg
            .acquire_many("p1", "s-a", "t-a", "A", plan_a.locks)
            .unwrap();

        let plan_b = plan_tool_io(
            &ctx_b,
            "fs_write",
            &json!({ "path": "unpacked/word/document.xml", "content": "x" }),
        )
        .unwrap();
        let err = reg
            .acquire_many("p1", "s-b", "t-b", "B", plan_b.locks)
            .unwrap_err();
        assert!(
            err.message.contains("unpacked"),
            "expected pack subtree lock to block child write: {}",
            err.message
        );
    }

    #[test]
    fn write_tools_conflict_on_same_output_path() {
        let (_dir, sandbox, reg, _registry) = setup();
        let ctx_a = turn_ctx(&sandbox, reg.clone(), "s-a", "t-a");
        let ctx_b = turn_ctx(&sandbox, reg.clone(), "s-b", "t-b");

        {
            let plan_a = plan_tool_io(
                &ctx_a,
                "fs_write",
                &json!({ "path": "out.txt", "content": "a" }),
            )
            .unwrap();
            let _g = reg
                .acquire_many("p1", "s-a", "t-a", "A", plan_a.locks)
                .unwrap();
            let plan_b = plan_tool_io(
                &ctx_b,
                "fs_write",
                &json!({ "path": "out.txt", "content": "b" }),
            )
            .unwrap();
            let err = reg
                .acquire_many("p1", "s-b", "t-b", "B", plan_b.locks)
                .unwrap_err();
            assert!(err.message.contains("out.txt"));
        }

        {
            let plan_x = plan_tool_io(
                &ctx_a,
                "excel_write",
                &json!({ "path": "book.xlsx", "cells": [{ "cell": "A1", "value": "1" }] }),
            )
            .unwrap();
            let _gx = reg
                .acquire_many("p1", "s-a", "t-a", "A", plan_x.locks)
                .unwrap();
            let plan_y = plan_tool_io(
                &ctx_b,
                "excel_write",
                &json!({ "path": "book.xlsx", "cells": [{ "cell": "A1", "value": "2" }] }),
            )
            .unwrap();
            let err_x = reg
                .acquire_many("p1", "s-b", "t-b", "B", plan_y.locks)
                .unwrap_err();
            assert!(err_x.message.contains("book.xlsx"));
        }

        {
            let plan_pdf_a = plan_tool_io(
                &ctx_a,
                "pdf_split",
                &json!({ "path": "sample.pdf", "mode": "burst", "out_dir": "pages" }),
            )
            .unwrap();
            let _gp = reg
                .acquire_many("p1", "s-a", "t-a", "A", plan_pdf_a.locks)
                .unwrap();
            let plan_pdf_b = plan_tool_io(
                &ctx_b,
                "pdf_split",
                &json!({ "path": "sample.pdf", "mode": "burst", "out_dir": "pages" }),
            )
            .unwrap();
            let err_p = reg
                .acquire_many("p1", "s-b", "t-b", "B", plan_pdf_b.locks)
                .unwrap_err();
            assert!(err_p.message.contains("pages"));
        }

        {
            let plan_typ_a = plan_tool_io(
                &ctx_a,
                "typst_to_pdf",
                &json!({ "path": "sample.typ", "out_path": "out.pdf" }),
            )
            .unwrap();
            let _gt = reg
                .acquire_many("p1", "s-a", "t-a", "A", plan_typ_a.locks)
                .unwrap();
            let plan_typ_b = plan_tool_io(
                &ctx_b,
                "typst_to_pdf",
                &json!({ "path": "sample.typ", "out_path": "out.pdf" }),
            )
            .unwrap();
            let err_t = reg
                .acquire_many("p1", "s-b", "t-b", "B", plan_typ_b.locks)
                .unwrap_err();
            assert!(err_t.message.contains("out.pdf"));
        }
    }
}
