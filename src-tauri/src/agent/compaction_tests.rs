use super::{
    prepare_compaction_split, reserved_context_size, should_auto_compact, COMPACTION_TRIGGER_RATIO,
    MAX_PRESERVED_MESSAGES,
};
use crate::core::store::{Message, ToolCallRecord};
use crate::state::AppState;

#[test]
fn deepseek_ratio_triggers_first() {
    assert!(should_auto_compact(
        850_000,
        1_000_000,
        COMPACTION_TRIGGER_RATIO,
        reserved_context_size(1_000_000),
    ));
}

#[test]
fn kimi_reserved_triggers_first() {
    assert!(should_auto_compact(
        210_000,
        256_000,
        COMPACTION_TRIGGER_RATIO,
        50_000,
    ));
}

#[test]
fn empty_context_never_triggers() {
    assert!(!should_auto_compact(
        0,
        256_000,
        COMPACTION_TRIGGER_RATIO,
        50_000,
    ));
}

#[test]
fn prepare_split_keeps_recent_messages() {
    let messages = vec![
        msg("m1", "user", "old"),
        msg("m2", "assistant", "old reply"),
        msg("m3", "user", "recent"),
        msg("m4", "assistant", "latest"),
    ];
    let prepared = prepare_compaction_split(&messages, &[], MAX_PRESERVED_MESSAGES).unwrap();
    assert_eq!(prepared.to_compact.len(), 2);
    assert_eq!(prepared.to_preserve.len(), 2);
    assert_eq!(prepared.to_preserve[0].id, "m3");
}

#[test]
fn prepare_split_expands_tool_group() {
    let messages = vec![
        msg("m1", "user", "old"),
        msg("m2", "assistant", "call"),
        msg("m3", "tool", "result"),
        msg("m4", "user", "recent"),
        msg("m5", "assistant", "latest"),
    ];
    let tool_calls = vec![ToolCallRecord {
        id: "call_1".into(),
        message_id: "m2".into(),
        name: "fs_list".into(),
        args_json: "{}".into(),
        result_json: Some("{}".into()),
        status: "done".into(),
        duration_ms: 1,
        created_at: "now".into(),
    }];
    let prepared =
        prepare_compaction_split(&messages, &tool_calls, MAX_PRESERVED_MESSAGES).unwrap();
    assert!(prepared.to_preserve.iter().any(|m| m.id == "m2"));
    assert!(prepared.to_preserve.iter().any(|m| m.id == "m3"));
}

#[test]
fn compaction_summary_is_not_counted_as_recent_turn() {
    let messages = vec![
        msg(
            "m1",
            "user",
            &super::build_summary_message_content("old summary"),
        ),
        msg("m2", "user", "current request"),
    ];
    assert!(prepare_compaction_split(&messages, &[], MAX_PRESERVED_MESSAGES).is_none());
    assert_eq!(super::fallback_preserve_start(&messages, &[]), 0);
}

#[test]
fn store_token_estimate_counts_assistant_tool_calls() {
    let messages = vec![msg("m1", "assistant", "call")];
    let tool_calls = vec![ToolCallRecord {
        id: "call_1".into(),
        message_id: "m1".into(),
        name: "fs_read".into(),
        args_json: "{\"path\":\"".to_string() + &"a".repeat(400) + "\"}",
        result_json: None,
        status: "done".into(),
        duration_ms: 1,
        created_at: "now".into(),
    }];
    let without_tools = super::estimate_text_tokens("call");
    let with_tools = super::estimate_store_messages_tokens(&messages, &tool_calls);
    assert!(with_tools > without_tools + 50);
}

#[test]
fn compacted_token_estimate_includes_summary_prefix() {
    let summary = "short summary";
    let summary_content = super::build_summary_message_content(summary);
    let tokens = super::estimate_compacted_tokens(&summary_content, &[], &[]);
    assert!(tokens > super::estimate_text_tokens(summary));
}

#[test]
fn compaction_rebuild_preserves_profile_init_hint() {
    let messages = crate::agent::loop_support::build_working_messages(
        &[],
        &[],
        Some("/init"),
        &[],
        false,
        None,
        true,
    )
    .unwrap();
    let system = messages[0].content.as_ref().unwrap();
    assert!(system.contains("skill_read profile"));
}

#[test]
fn working_messages_after_compaction_include_system_tokens() {
    let history = vec![msg("m1", "user", "recent")];
    let working = crate::agent::loop_support::build_working_messages(
        &history,
        &[],
        None,
        &[],
        false,
        None,
        false,
    )
    .unwrap();
    let compact_only = super::estimate_compacted_tokens(
        &super::build_summary_message_content("summary"),
        &[],
        &[],
    );
    let with_system = super::estimate_chat_messages_tokens(&working);
    assert!(
        with_system > compact_only,
        "token baseline must include injected system prompt"
    );
    assert!(working.first().is_some_and(|m| m.role == "system"));
}

fn msg(id: &str, role: &str, content: &str) -> Message {
    Message {
        id: id.into(),
        session_id: "s1".into(),
        role: role.into(),
        content: Some(content.into()),
        reasoning_content: None,
        tool_call_id: if role == "tool" {
            Some("call_1".into())
        } else {
            None
        },
        seq: 0,
        created_at: "now".into(),
        archived: false,
        attachments_json: None,
    }
}

#[test]
fn fallback_preserve_start_keeps_tool_chain() {
    let messages = vec![
        msg("m1", "user", "only user"),
        msg("m2", "assistant", "call"),
        msg("m3", "tool", "result"),
    ];
    let tool_calls = vec![ToolCallRecord {
        id: "call_1".into(),
        message_id: "m2".into(),
        name: "fs_list".into(),
        args_json: "{}".into(),
        result_json: Some("{}".into()),
        status: "done".into(),
        duration_ms: 1,
        created_at: "now".into(),
    }];
    let preserve_start = super::fallback_preserve_start(&messages, &tool_calls);
    assert_eq!(preserve_start, 1);
    assert_eq!(messages[preserve_start..].len(), 2);
}

#[test]
fn expand_archive_includes_tools_for_archived_assistant() {
    let messages = vec![
        msg("m1", "user", "old"),
        msg("m2", "assistant", "call"),
        msg("m3", "tool", "result"),
    ];
    let tool_calls = vec![ToolCallRecord {
        id: "call_1".into(),
        message_id: "m2".into(),
        name: "fs_list".into(),
        args_json: "{}".into(),
        result_json: Some("{}".into()),
        status: "done".into(),
        duration_ms: 1,
        created_at: "now".into(),
    }];
    let expanded = super::expand_archive_ids_for_tool_pairs(
        &messages,
        &tool_calls,
        messages.len(),
        vec!["m2".into()],
    );
    assert_eq!(expanded.len(), 2);
    assert!(expanded.contains(&"m2".to_string()));
    assert!(expanded.contains(&"m3".to_string()));
}

#[test]
fn force_compact_short_history_is_noop() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let project_root = dir.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();
        let session_id = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            let session = store
                .create_session(&project.id, "s1", "mock", false, "high")
                .unwrap();
            store
                .add_message(&session.id, "user", Some("hi"), None, None, None)
                .unwrap();
            store
                .add_message(&session.id, "assistant", Some("hello"), None, None, None)
                .unwrap();
            session.id
        };
        let resp = super::force_compact_session(&app.handle(), &state, &session_id)
            .await
            .unwrap();
        assert!(!resp.compacted);
        assert_eq!(resp.reason.as_deref(), Some(super::NOTHING_TO_COMPACT));
    });
}

#[test]
fn force_compact_rejects_busy_session() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let (session_id, project_id) = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", dir.path().join("project").to_str().unwrap())
                .unwrap();
            std::fs::create_dir_all(&project.root_path).unwrap();
            let session = store
                .create_session(&project.id, "s1", "mock", false, "high")
                .unwrap();
            (session.id, project.id)
        };
        state
            .turns
            .register(session_id.clone(), "t1".into(), project_id)
            .unwrap();
        let err = super::force_compact_session(&app.handle(), &state, &session_id)
            .await
            .unwrap_err();
        assert!(err.contains("正在执行任务"));
    });
}

#[test]
fn truncate_fallback_reports_false_when_nothing_archived() {
    // History far below the truncate target must archive nothing and report
    // false, so manual /compact does not falsely claim success.
    let dir = tempfile::tempdir().unwrap();
    let state = AppState::new(dir.path().join("data")).unwrap();
    let candidates = vec![msg("m1", "user", "hi"), msg("m2", "assistant", "hello")];
    let archived = super::truncate_fallback_compact_only(
        &state,
        "s1",
        &candidates,
        &[],
        1_000_000,
        super::reserved_context_size(1_000_000),
    )
    .unwrap();
    assert!(!archived);
}

#[test]
fn force_compact_rejects_when_run_limiter_full() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let session_id = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", dir.path().join("project").to_str().unwrap())
                .unwrap();
            std::fs::create_dir_all(&project.root_path).unwrap();
            let session = store
                .create_session(&project.id, "s1", "mock", false, "high")
                .unwrap();
            store
                .add_message(&session.id, "user", Some("hi"), None, None, None)
                .unwrap();
            store
                .add_message(&session.id, "assistant", Some("hello"), None, None, None)
                .unwrap();
            session.id
        };
        let _g0 = state
            .run_limiter
            .acquire("other-0".into(), "t0".into(), "p".into())
            .unwrap();
        let _g1 = state
            .run_limiter
            .acquire("other-1".into(), "t1".into(), "p".into())
            .unwrap();
        let _g2 = state
            .run_limiter
            .acquire("other-2".into(), "t2".into(), "p".into())
            .unwrap();
        let err = super::force_compact_session(&app.handle(), &state, &session_id)
            .await
            .unwrap_err();
        assert_eq!(err, crate::agent::run_limiter::GLOBAL_PARALLEL_FULL_MSG);
    });
}

#[test]
fn truncate_for_compaction_input_handles_utf8_without_panic() {
    // 全中文内容（3 字节/字符），按字节切 head 会落在字符中间，必须回退到字符边界
    let mut body = "汉".repeat(150_000);
    assert!(body.len() > 200_000);
    super::truncate_for_compaction_input(&mut body, 200_000);
    assert!(body.contains("[... omitted"));
    assert!(body.len() <= 200_000 + 128);
}
