use super::{
    prepare_compaction_split, reserved_context_size, should_auto_compact, COMPACTION_TRIGGER_RATIO,
    MAX_PRESERVED_MESSAGES,
};
use crate::core::store::{Message, ToolCallRecord};

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
fn working_messages_after_compaction_include_system_tokens() {
    let history = vec![msg("m1", "user", "recent")];
    let working =
        crate::agent::loop_support::build_working_messages(&history, &[], None, &[], false, None)
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
