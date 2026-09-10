use super::tests::{call, message, request, turn};
use crate::agent::provider::{
    openai_compat::{extra_body_for, messages_from_store_text},
    openai_request::build_body,
};
use crate::agent::types::{AssistantTurn, ChatRequest};
use crate::core::store::Store;
use serde_json::Value;
use tempfile::{tempdir, TempDir};

pub(super) fn setup() -> (TempDir, Store, String) {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("test.db")).unwrap();
    let project = store
        .create_project("demo", dir.path().to_str().unwrap())
        .unwrap();
    let session = store
        .create_session(&project.id, "test", "gemini-3.8-flash", true, "medium")
        .unwrap();
    (dir, store, session.id)
}

pub(super) fn persist(store: &Store, session: &str, turn: &AssistantTurn) {
    let msg = store
        .add_message_with_provider_state(
            session,
            "assistant",
            Some(&turn.content),
            Some(&turn.reasoning_content),
            None,
            None,
            turn.provider_state.as_deref(),
        )
        .unwrap();
    for call in &turn.tool_calls {
        store
            .add_tool_call(
                &msg.id,
                &call.id,
                &call.function.name,
                &call.function.arguments,
            )
            .unwrap();
    }
    assert!(!serde_json::to_string(&msg)
        .unwrap()
        .contains("provider_state"));
    if let Some(raw) = &turn.provider_state {
        assert!(!format!("{msg:?}").contains(raw));
    }
}

pub(super) fn result(store: &Store, session: &str, id: &str, text: &str) {
    store.finish_tool_call(id, text, "done", 1).unwrap();
    store
        .add_message(session, "tool", Some(text), None, Some(id), None)
        .unwrap();
}

pub(super) fn reload(store: &Store, session: &str) -> ChatRequest {
    let mut req = request();
    req.messages = messages_from_store_text(
        &store.list_active_messages(session).unwrap(),
        &store.list_tool_calls_for_session(session).unwrap(),
    );
    req
}

fn body(store: &Store, session: &str) -> Value {
    let req = reload(store, session);
    build_body(&req, &extra_body_for(&req), true).unwrap()
}

#[test]
fn real_store_restart_preserves_parallel_order_after_id_collision_and_third_call() {
    let (dir, store, session) = setup();
    store
        .add_message(
            &session,
            "user",
            Some("alpha beta then gamma"),
            None,
            None,
            None,
        )
        .unwrap();
    let mut first = turn(vec![
        call("z", r#"{"name":"alpha"}"#, Some("sig-first")),
        call("a", r#"{"name":"beta"}"#, Some("sig-second")),
    ]);
    crate::agent::loop_support::normalize_tool_call_ids(&mut first.tool_calls, |id| id == "z");
    assert_ne!(first.tool_calls[0].id, "z");
    persist(&store, &session, &first);
    for (call, text) in first.tool_calls.iter().zip(["A17", "B29"]) {
        result(&store, &session, &call.id, text);
    }
    drop(store);
    let store = Store::open(dir.path().join("test.db")).unwrap();
    let b = body(&store, &session);
    let msgs = b["messages"].as_array().unwrap();
    assert_eq!(
        msgs.iter()
            .map(|m| m["role"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["user", "assistant", "tool", "tool"]
    );
    for (i, sig) in ["sig-first", "sig-second"].iter().enumerate() {
        assert_eq!(
            msgs[1]["tool_calls"][i]["extra_content"]["google"]["thought_signature"],
            *sig
        );
        assert_eq!(msgs[1]["tool_calls"][i]["id"], msgs[2 + i]["tool_call_id"]);
    }
    assert_eq!(msgs[1]["content"], "before");
    let third = turn(vec![call("c", r#"{"name":"gamma"}"#, Some("sig-third"))]);
    persist(&store, &session, &third);
    result(&store, &session, "c", "NOT_FOUND");
    let b = body(&store, &session);
    assert_eq!(
        b["messages"][4]["tool_calls"][0]["extra_content"]["google"]["thought_signature"],
        "sig-third"
    );
    assert_eq!(b["messages"][5]["content"], "NOT_FOUND");
}

#[test]
fn clarify_pending_restart_keeps_signature_without_creating_user_turn() {
    let (dir, store, session) = setup();
    store
        .add_message(&session, "user", Some("ask audience"), None, None, None)
        .unwrap();
    let mut question = call("q", r#"{"question":"audience?"}"#, Some("sig-clarify"));
    question["function"]["name"] = Value::String("clarify_ask".into());
    let question = turn(vec![question]);
    persist(&store, &session, &question);
    store
        .save_clarify_pending(&session, "t", "q", r#"{"question":"audience?"}"#)
        .unwrap();
    drop(store);
    let store = Store::open(dir.path().join("test.db")).unwrap();
    assert!(store.get_clarify_pending(&session).unwrap().is_some());
    result(&store, &session, "q", "management");
    store.delete_clarify_pending(&session).unwrap();
    let b = body(&store, &session);
    assert_eq!(b["messages"].as_array().unwrap().len(), 3);
    assert_eq!(b["messages"][2]["role"], "tool");
    assert_eq!(b["messages"][2]["tool_call_id"], "q");
    assert_eq!(
        b["messages"][1]["tool_calls"][0]["extra_content"]["google"]["thought_signature"],
        "sig-clarify"
    );
}

#[test]
fn plaintext_history_needs_no_extra_state() {
    let mut req = request();
    req.messages.push(message("assistant", "answer"));
    req.messages.push(message("user", "continue"));
    let b = build_body(&req, &extra_body_for(&req), true).unwrap();
    assert_eq!(b["messages"][1]["content"], "answer");
    assert!(b["messages"][1].get("extra_content").is_none());
}
