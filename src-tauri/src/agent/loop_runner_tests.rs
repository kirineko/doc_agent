use super::*;
use crate::agent::types::AgentEvent;
use crate::core::store::{Message, Store};
use crate::state::AppState;
use tempfile::tempdir;

#[test]
fn assistant_step_done_event_serializes() {
    let event = AgentEvent::AssistantStepDone {
        session_id: "s1".into(),
        turn_id: "t1".into(),
        message: Message {
            id: "m1".into(),
            session_id: "s1".into(),
            role: "assistant".into(),
            content: Some("answer".into()),
            reasoning_content: Some("thought".into()),
            tool_call_id: None,
            seq: 1,
            created_at: "2026-01-01".into(),
        },
    };
    let value = serde_json::to_value(&event).unwrap();
    assert_eq!(value["kind"], "assistant_step_done");
    assert_eq!(value["message"]["id"], "m1");
}

#[test]
fn reasoning_content_is_persisted_with_assistant() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("test.db")).unwrap();
    let project = store
        .create_project("demo", dir.path().to_str().unwrap())
        .unwrap();
    let session = store
        .create_session(&project.id, "s1", "mock", true, "high")
        .unwrap();
    store
        .add_message(
            &session.id,
            "assistant",
            Some("answer"),
            Some("thought"),
            None,
        )
        .unwrap();
    let messages = store.list_messages(&session.id).unwrap();
    assert_eq!(messages[0].reasoning_content.as_deref(), Some("thought"));
}

#[test]
fn system_prompt_includes_clarify_trigger() {
    let messages =
        crate::agent::loop_support::build_working_messages(&[], &[], Some("帮我做一份 PPT"), false);
    let system = messages[0].content.as_ref().unwrap();
    assert!(system.contains("skill_read clarify"));
    assert!(system.contains("clarify_ask"));
}

#[test]
fn mock_clarify_pause_submit_and_resume() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let handle = app.handle().clone();
        let project_root = dir.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();
        let session_id = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            store
                .create_session(&project.id, "s1", "mock", true, "high")
                .unwrap()
                .id
        };

        run_turn(
            handle.clone(),
            state.clone(),
            session_id.clone(),
            "请澄清需求".into(),
        )
        .await
        .unwrap();
        {
            let store = state.store.lock().unwrap();
            let pending = store.get_clarify_pending(&session_id).unwrap().unwrap();
            assert_eq!(pending.tool_call_id, "call_mock_clarify_1");
            let calls = store.list_tool_calls_for_session(&session_id).unwrap();
            assert_eq!(calls[0].status, "awaiting_user");
        }

        let blocked = run_turn(
            handle.clone(),
            state.clone(),
            session_id.clone(),
            "新消息".into(),
        )
        .await
        .unwrap_err();
        assert!(blocked.contains("请先回答当前澄清问题"));

        crate::agent::clarify_interaction::submit_clarify_answer(
            handle.clone(),
            state.clone(),
            crate::agent::clarify_interaction::SubmitClarifyAnswer {
                session_id: session_id.clone(),
                question_id: "mock_doc_type".into(),
                selected: vec!["pptx".into()],
                custom: None,
            },
        )
        .await
        .unwrap();

        {
            let store = state.store.lock().unwrap();
            assert!(store.get_clarify_pending(&session_id).unwrap().is_none());
            let calls = store.list_tool_calls_for_session(&session_id).unwrap();
            assert_eq!(calls[0].status, "done");
            assert!(calls[0]
                .result_json
                .as_deref()
                .unwrap()
                .contains("PPT 演示"));
            let messages = store.list_messages(&session_id).unwrap();
            assert_eq!(messages.iter().filter(|m| m.role == "user").count(), 1);
            assert!(messages.iter().any(|m| m.role == "assistant"
                && m.content
                    .as_deref()
                    .unwrap_or("")
                    .contains("已收到澄清答案")));
        }

        let second = crate::agent::clarify_interaction::submit_clarify_answer(
            handle,
            state,
            crate::agent::clarify_interaction::SubmitClarifyAnswer {
                session_id,
                question_id: "mock_doc_type".into(),
                selected: vec!["pptx".into()],
                custom: None,
            },
        )
        .await
        .unwrap_err();
        assert!(second.contains("已处理或不存在"));
    });
}

#[test]
fn mock_mixed_tools_run_before_clarify_pause() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let handle = app.handle().clone();
        let project_root = dir.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();
        let session_id = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            store
                .create_session(&project.id, "s1", "mock", true, "high")
                .unwrap()
                .id
        };

        run_turn(
            handle.clone(),
            state.clone(),
            session_id.clone(),
            "列出目录并澄清需求".into(),
        )
        .await
        .unwrap();

        {
            let store = state.store.lock().unwrap();
            let calls = store.list_tool_calls_for_session(&session_id).unwrap();
            let fs_call = calls.iter().find(|c| c.name == "fs_list").unwrap();
            assert_eq!(fs_call.status, "done");
            assert!(fs_call.result_json.is_some());
            let clarify_call = calls.iter().find(|c| c.name == "clarify_ask").unwrap();
            assert_eq!(clarify_call.status, "awaiting_user");
            assert!(clarify_call.result_json.is_none());
            let pending = store.get_clarify_pending(&session_id).unwrap().unwrap();
            assert_eq!(pending.tool_call_id, clarify_call.id);
        }

        crate::agent::clarify_interaction::submit_clarify_answer(
            handle,
            state.clone(),
            crate::agent::clarify_interaction::SubmitClarifyAnswer {
                session_id: session_id.clone(),
                question_id: "mock_doc_type".into(),
                selected: vec!["docx".into()],
                custom: None,
            },
        )
        .await
        .unwrap();

        {
            let store = state.store.lock().unwrap();
            assert!(store.get_clarify_pending(&session_id).unwrap().is_none());
            let calls = store.list_tool_calls_for_session(&session_id).unwrap();
            assert!(calls.iter().all(|c| c.status == "done"));
            let messages = store.list_messages(&session_id).unwrap();
            assert!(messages.iter().any(|m| m.role == "assistant"
                && m.content
                    .as_deref()
                    .unwrap_or("")
                    .contains("已收到澄清答案")));
        }
    });
}
