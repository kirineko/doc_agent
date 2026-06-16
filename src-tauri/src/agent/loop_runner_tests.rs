use super::*;
use crate::agent::compaction::MAX_TOOL_STEPS;
use crate::agent::types::AgentEvent;
use crate::core::store::{Message, Store};
use crate::state::AppState;
use std::time::Duration;
use tempfile::tempdir;

fn seed_bulky_history(store: &Store, session_id: &str, pairs: usize) {
    for index in 0..pairs {
        store
            .add_message(
                session_id,
                "user",
                Some(&format!("old user {index} {}", "a".repeat(8_000))),
                None,
                None,
                None,
            )
            .unwrap();
        store
            .add_message(
                session_id,
                "assistant",
                Some(&format!("old assistant {index} {}", "b".repeat(8_000))),
                None,
                None,
                None,
            )
            .unwrap();
    }
}

#[test]
fn max_tool_steps_is_64() {
    assert_eq!(MAX_TOOL_STEPS, 64);
}

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
            archived: false,
            attachments_json: None,
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
            None,
        )
        .unwrap();
    let messages = store.list_messages(&session.id).unwrap();
    assert_eq!(messages[0].reasoning_content.as_deref(), Some("thought"));
}

#[test]
fn system_prompt_includes_clarify_trigger() {
    let messages = crate::agent::loop_support::build_working_messages(
        &[],
        &[],
        Some("帮我做一份 PPT"),
        &[],
        false,
        None,
    )
    .unwrap();
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
            vec![],
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
            vec![],
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
            vec![],
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

#[test]
fn mock_clarify_first_still_runs_non_clarify_before_pause() {
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
            handle,
            state.clone(),
            session_id.clone(),
            "先澄清再列出".into(),
            vec![],
        )
        .await
        .unwrap();

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
    });
}

#[test]
fn mock_pdf_reads_finish_before_clarify_pause() {
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
        // The files are intentionally invalid PDFs: this test asserts ordering and
        // result persistence before clarify pause, not PDF extraction success.
        std::fs::write(project_root.join("a.pdf"), b"not-a-pdf").unwrap();
        std::fs::write(project_root.join("b.pdf"), b"not-a-pdf").unwrap();
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
            handle,
            state.clone(),
            session_id.clone(),
            "读取PDF并澄清".into(),
            vec![],
        )
        .await
        .unwrap();

        let store = state.store.lock().unwrap();
        let calls = store.list_tool_calls_for_session(&session_id).unwrap();
        let pdf_calls: Vec<_> = calls.iter().filter(|c| c.name == "pdf_read").collect();
        assert_eq!(pdf_calls.len(), 2);
        assert!(pdf_calls
            .iter()
            .all(|c| c.status == "done" || c.status == "error"));
        assert!(pdf_calls.iter().all(|c| c.result_json.is_some()));
        let clarify_call = calls.iter().find(|c| c.name == "clarify_ask").unwrap();
        assert_eq!(clarify_call.status, "awaiting_user");
        assert!(clarify_call.result_json.is_none());
        let pending = store.get_clarify_pending(&session_id).unwrap().unwrap();
        assert_eq!(pending.tool_call_id, clarify_call.id);
    });
}

#[test]
fn mock_slow_turn_cancelled_without_turn_complete() {
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

        let shared = state.clone();
        let sid = session_id.clone();
        let run =
            tokio::spawn(
                async move { run_turn(handle, shared, sid, "慢工具".into(), vec![]).await },
            );

        tokio::time::sleep(Duration::from_millis(150)).await;
        state.turns.cancel(&session_id).unwrap();
        run.await.unwrap().unwrap();

        assert!(!state.turns.is_session_active(&session_id));
        let store = state.store.lock().unwrap();
        let calls = store.list_tool_calls_for_session(&session_id).unwrap();
        assert!(calls.is_empty(), "cancel during SSE should not run tools");
    });
}

#[test]
fn project_rejects_second_session_while_first_is_running() {
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
        let (session_a, session_b) = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            let a = store
                .create_session(&project.id, "会话 A", "mock", true, "high")
                .unwrap()
                .id;
            let b = store
                .create_session(&project.id, "会话 B", "mock", true, "high")
                .unwrap()
                .id;
            (a, b)
        };

        let shared = state.clone();
        let sid = session_a.clone();
        let run =
            tokio::spawn(
                async move { run_turn(handle, shared, sid, "慢工具".into(), vec![]).await },
            );

        tokio::time::sleep(Duration::from_millis(50)).await;
        let blocked = run_turn(
            app.handle().clone(),
            state.clone(),
            session_b.clone(),
            "hello".into(),
            vec![],
        )
        .await
        .unwrap_err();
        assert!(blocked.contains("项目内有其他会话正在执行任务"));
        {
            let store = state.store.lock().unwrap();
            let messages = store.list_messages(&session_b).unwrap();
            assert!(
                messages
                    .iter()
                    .all(|m| m.role != "user" || m.content.as_deref() != Some("hello")),
                "rejected send must not persist user message"
            );
        }

        state.turns.cancel(&session_a).unwrap();
        run.await.unwrap().unwrap();
    });
}

#[test]
fn clarify_pending_allows_other_session_in_same_project() {
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
        let (session_a, session_b) = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            let a = store
                .create_session(&project.id, "会话 A", "mock", true, "high")
                .unwrap()
                .id;
            let b = store
                .create_session(&project.id, "会话 B", "mock", true, "high")
                .unwrap()
                .id;
            (a, b)
        };

        run_turn(
            handle.clone(),
            state.clone(),
            session_a.clone(),
            "请澄清需求".into(),
            vec![],
        )
        .await
        .unwrap();

        {
            let store = state.store.lock().unwrap();
            assert!(store.get_clarify_pending(&session_a).unwrap().is_some());
            assert!(!state.turns.is_session_active(&session_a));
        }

        run_turn(
            handle,
            state.clone(),
            session_b.clone(),
            "另一条消息".into(),
            vec![],
        )
        .await
        .expect("clarify pending on A should not block B in same project");
    });
}

#[test]
fn clarify_submit_rejected_while_other_session_running() {
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
        let (session_a, session_b) = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            let a = store
                .create_session(&project.id, "会话 A", "mock", true, "high")
                .unwrap()
                .id;
            let b = store
                .create_session(&project.id, "会话 B", "mock", true, "high")
                .unwrap()
                .id;
            (a, b)
        };

        run_turn(
            handle.clone(),
            state.clone(),
            session_a.clone(),
            "请澄清需求".into(),
            vec![],
        )
        .await
        .unwrap();

        let shared = state.clone();
        let sid = session_b.clone();
        let run =
            tokio::spawn(
                async move { run_turn(handle, shared, sid, "慢工具".into(), vec![]).await },
            );
        tokio::time::sleep(Duration::from_millis(50)).await;

        let blocked = crate::agent::clarify_interaction::submit_clarify_answer(
            app.handle().clone(),
            state.clone(),
            crate::agent::clarify_interaction::SubmitClarifyAnswer {
                session_id: session_a.clone(),
                question_id: "mock_doc_type".into(),
                selected: vec!["pptx".into()],
                custom: None,
            },
        )
        .await
        .unwrap_err();
        assert!(blocked.contains("项目内有其他会话正在执行任务"));
        {
            let store = state.store.lock().unwrap();
            assert!(store.get_clarify_pending(&session_a).unwrap().is_some());
        }

        state.turns.cancel(&session_b).unwrap();
        run.await.unwrap().unwrap();
    });
}

#[test]
fn clarify_cancel_allowed_while_other_session_running() {
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
        let (session_a, session_b) = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            let a = store
                .create_session(&project.id, "会话 A", "mock", true, "high")
                .unwrap()
                .id;
            let b = store
                .create_session(&project.id, "会话 B", "mock", true, "high")
                .unwrap()
                .id;
            (a, b)
        };

        run_turn(
            handle.clone(),
            state.clone(),
            session_a.clone(),
            "请澄清需求".into(),
            vec![],
        )
        .await
        .unwrap();

        let shared = state.clone();
        let sid = session_b.clone();
        let run =
            tokio::spawn(
                async move { run_turn(handle, shared, sid, "慢工具".into(), vec![]).await },
            );
        tokio::time::sleep(Duration::from_millis(50)).await;

        crate::agent::clarify_interaction::cancel_clarify(
            app.handle().clone(),
            state.clone(),
            session_a.clone(),
        )
        .await
        .unwrap();

        {
            let store = state.store.lock().unwrap();
            assert!(store.get_clarify_pending(&session_a).unwrap().is_none());
            let calls = store.list_tool_calls_for_session(&session_a).unwrap();
            let clarify = calls
                .iter()
                .find(|call| call.name == "clarify_ask")
                .unwrap();
            assert!(clarify
                .result_json
                .as_deref()
                .is_some_and(|json| json.contains("cancelled")));
        }

        state.turns.cancel(&session_b).unwrap();
        run.await.unwrap().unwrap();
    });
}

#[test]
fn mock_turn_compacts_near_context_limit_before_llm() {
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
            let session = store
                .create_session(&project.id, "s1", "mock", true, "high")
                .unwrap();
            seed_bulky_history(&store, &session.id, 8);
            store.set_session_token_count(&session.id, 88_000).unwrap();
            session.id
        };

        run_turn(
            handle,
            state.clone(),
            session_id.clone(),
            "继续写文档".into(),
            vec![],
        )
        .await
        .unwrap();

        let store = state.store.lock().unwrap();
        let all = store.list_all_messages(&session_id).unwrap();
        let archived = all.iter().filter(|m| m.archived).count();
        assert!(
            archived >= 2,
            "expected archived history after compaction, got {archived}"
        );

        let active = store.list_active_messages(&session_id).unwrap();
        assert!(
            active.iter().any(|m| {
                m.content
                    .as_deref()
                    .unwrap_or("")
                    .starts_with("Previous context has been compacted.")
            }),
            "expected compaction summary in active messages"
        );
        assert!(
            active
                .first()
                .and_then(|m| m.content.as_deref())
                .unwrap_or("")
                .starts_with("Previous context has been compacted."),
            "summary should precede preserved messages"
        );
        assert_eq!(
            active
                .iter()
                .filter(|m| m.role == "user" && m.content.as_deref() == Some("继续写文档"))
                .count(),
            1,
            "current turn user message must not be duplicated after compaction rebuild"
        );

        let token_count = store
            .get_session_token_count(&session_id)
            .unwrap()
            .unwrap_or(0);
        assert!(
            token_count < 88_000,
            "expected token baseline to drop after compaction, got {token_count}"
        );
        assert!(
            active.len() < all.len(),
            "active context should be smaller than full history"
        );
    });
}
