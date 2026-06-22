use crate::agent::loop_runner::TurnExecutionContext;
use crate::agent::loop_support::{emit, persist_clarify_pending, persist_tool_result};
use crate::agent::tool_args::{parse_tool_arguments, truncation_error};
use crate::agent::turn_control::CancelSignal;
use crate::agent::types::{AgentEvent, ChatMessage, ModelId, ToolCall};
use crate::core::sandbox::Sandbox;
use crate::state::AppState;
use crate::tools::io_plan::plan_tool_io;
use crate::tools::runtime::RuntimeWriteGate;
use crate::tools::ToolContext;
use futures_util::future::join_all;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Runtime};
use tokio::sync::Semaphore;

const PDF_READ_MAX_PARALLEL: usize = 3;

#[derive(Clone)]
struct ToolCallPlan {
    index: usize,
    call: ToolCall,
    args: Value,
    prebuilt_error: Option<Value>,
}

struct ExecOutcome {
    ok: bool,
    summary: String,
    changed_paths: Vec<String>,
    duration_ms: i64,
}

pub struct ToolBatchOutcome {
    pub has_pending_clarify: bool,
    pub cancelled: bool,
}

#[allow(clippy::too_many_arguments)]
fn finish_cancelled_clarify_batch<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    session_id: &str,
    turn_id: &str,
    plans: &[ToolCallPlan],
    working_messages: &mut Vec<ChatMessage>,
    pending_estimate: &mut u32,
    cancel: &CancelSignal,
) -> Result<Option<ToolBatchOutcome>, String> {
    if !cancel.is_cancelled() {
        return Ok(None);
    }
    persist_cancelled_clarify_remaining(
        app,
        state,
        session_id,
        turn_id,
        plans,
        working_messages,
        pending_estimate,
    )?;
    Ok(Some(ToolBatchOutcome {
        has_pending_clarify: false,
        cancelled: true,
    }))
}

#[allow(clippy::too_many_arguments)]
pub async fn run_tool_batch<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    sandbox: &Sandbox,
    turn_ctx: &TurnExecutionContext,
    model_id: ModelId,
    tool_calls: &[ToolCall],
    stream_indices: &[usize],
    truncated: bool,
    working_messages: &mut Vec<ChatMessage>,
    pending_estimate: &mut u32,
    cancel: &CancelSignal,
) -> Result<ToolBatchOutcome, String> {
    let session_id = &turn_ctx.session_id;
    let turn_id = &turn_ctx.turn_id;
    let plans = build_plans(tool_calls, stream_indices, truncated)?;
    let mut has_pending_clarify = false;

    for plan in &plans {
        if plan.call.function.name == "clarify_ask" {
            continue;
        }
        emit_tool_call(app, session_id, turn_id, plan, plan.args.clone(), "running");
    }

    let mut idx = 0;
    while idx < plans.len() {
        if cancel.is_cancelled() {
            persist_cancelled_remaining(
                app,
                state,
                session_id,
                turn_id,
                &plans[idx..],
                working_messages,
                pending_estimate,
            )?;
            return Ok(ToolBatchOutcome {
                has_pending_clarify: false,
                cancelled: true,
            });
        }
        if plans[idx].call.function.name == "clarify_ask" {
            idx += 1;
            continue;
        }
        if plans[idx].call.function.name == "pdf_read" {
            let start = idx;
            while idx < plans.len() && plans[idx].call.function.name == "pdf_read" {
                idx += 1;
            }
            let batch = &plans[start..idx];
            let outcomes =
                execute_pdf_read_batch(app, state, sandbox, turn_ctx, model_id, batch).await;
            for (plan, outcome) in batch.iter().zip(outcomes) {
                persist_tool_result(
                    state,
                    app,
                    session_id,
                    turn_id,
                    &plan.call,
                    outcome.ok,
                    outcome.summary,
                    outcome.duration_ms,
                    outcome.changed_paths,
                    working_messages,
                )?;
                bump_pending(working_messages, pending_estimate);
            }
        } else {
            let outcome = execute_one(app, state, sandbox, turn_ctx, model_id, &plans[idx]).await;
            persist_tool_result(
                state,
                app,
                session_id,
                turn_id,
                &plans[idx].call,
                outcome.ok,
                outcome.summary,
                outcome.duration_ms,
                outcome.changed_paths,
                working_messages,
            )?;
            bump_pending(working_messages, pending_estimate);
            idx += 1;
        }
    }

    for (offset, plan) in plans.iter().enumerate() {
        if let Some(outcome) = finish_cancelled_clarify_batch(
            app,
            state,
            session_id,
            turn_id,
            &plans[offset..],
            working_messages,
            pending_estimate,
            cancel,
        )? {
            return Ok(outcome);
        }
        if plan.call.function.name != "clarify_ask" {
            continue;
        }
        if plan.prebuilt_error.is_some() {
            emit_tool_call(app, session_id, turn_id, plan, plan.args.clone(), "running");
            let outcome = execute_one(app, state, sandbox, turn_ctx, model_id, plan).await;
            persist_tool_result(
                state,
                app,
                session_id,
                turn_id,
                &plan.call,
                outcome.ok,
                outcome.summary,
                outcome.duration_ms,
                outcome.changed_paths,
                working_messages,
            )?;
            bump_pending(working_messages, pending_estimate);
            continue;
        }
        match crate::tools::clarify::parse_question(plan.args.clone()) {
            Ok(question) if !has_pending_clarify => {
                if let Some(outcome) = finish_cancelled_clarify_batch(
                    app,
                    state,
                    session_id,
                    turn_id,
                    &plans[offset..],
                    working_messages,
                    pending_estimate,
                    cancel,
                )? {
                    return Ok(outcome);
                }
                has_pending_clarify = true;
                let normalized_args = serde_json::to_value(&question).map_err(|e| e.to_string())?;
                let question_json =
                    serde_json::to_string(&normalized_args).map_err(|e| e.to_string())?;
                persist_clarify_pending(state, session_id, turn_id, &plan.call.id, &question_json)?;
                emit_tool_call(
                    app,
                    session_id,
                    turn_id,
                    plan,
                    normalized_args,
                    "awaiting_user",
                );
                emit(
                    app,
                    AgentEvent::ClarifyQuestion {
                        session_id: session_id.to_string(),
                        turn_id: turn_id.to_string(),
                        tool_call_id: plan.call.id.clone(),
                        question,
                    },
                );
                if let Some(outcome) = finish_cancelled_clarify_batch(
                    app,
                    state,
                    session_id,
                    turn_id,
                    &plans[offset..],
                    working_messages,
                    pending_estimate,
                    cancel,
                )? {
                    return Ok(outcome);
                }
            }
            Ok(_) => {
                let value = json!({
                    "deferred": true,
                    "info": "本回合已有澄清问题待用户回答；每轮 assistant 仅允许一个 clarify_ask，请在用户回复后的下一轮单独提出此问题"
                });
                emit_tool_call(app, session_id, turn_id, plan, plan.args.clone(), "running");
                persist_tool_result(
                    state,
                    app,
                    session_id,
                    turn_id,
                    &plan.call,
                    true,
                    value.to_string(),
                    0,
                    Vec::new(),
                    working_messages,
                )?;
                bump_pending(working_messages, pending_estimate);
            }
            Err(err) => {
                let value = err.to_json_value();
                emit_tool_call(app, session_id, turn_id, plan, plan.args.clone(), "running");
                persist_tool_result(
                    state,
                    app,
                    session_id,
                    turn_id,
                    &plan.call,
                    false,
                    value.to_string(),
                    0,
                    Vec::new(),
                    working_messages,
                )?;
                bump_pending(working_messages, pending_estimate);
            }
        }
    }

    Ok(ToolBatchOutcome {
        has_pending_clarify,
        cancelled: false,
    })
}

fn persist_cancelled_remaining<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    session_id: &str,
    turn_id: &str,
    plans: &[ToolCallPlan],
    working_messages: &mut Vec<ChatMessage>,
    pending_estimate: &mut u32,
) -> Result<(), String> {
    let value = json!({ "cancelled": true });
    let summary = value.to_string();
    for plan in plans {
        emit_tool_call(app, session_id, turn_id, plan, plan.args.clone(), "running");
        persist_tool_result(
            state,
            app,
            session_id,
            turn_id,
            &plan.call,
            true,
            summary.clone(),
            0,
            Vec::new(),
            working_messages,
        )?;
        bump_pending(working_messages, pending_estimate);
    }
    Ok(())
}

fn persist_cancelled_clarify_remaining<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    session_id: &str,
    turn_id: &str,
    plans: &[ToolCallPlan],
    working_messages: &mut Vec<ChatMessage>,
    pending_estimate: &mut u32,
) -> Result<(), String> {
    let remaining: Vec<ToolCallPlan> = plans
        .iter()
        .filter(|plan| plan.call.function.name == "clarify_ask")
        .cloned()
        .collect();
    if remaining.is_empty() {
        return Ok(());
    }
    {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .delete_clarify_pending(session_id)
            .map_err(|e| e.to_string())?;
    }
    persist_cancelled_remaining(
        app,
        state,
        session_id,
        turn_id,
        &remaining,
        working_messages,
        pending_estimate,
    )
}

fn build_plans(
    tool_calls: &[ToolCall],
    stream_indices: &[usize],
    truncated: bool,
) -> Result<Vec<ToolCallPlan>, String> {
    if tool_calls.len() != stream_indices.len() {
        return Err("tool_calls and stream_indices length mismatch".into());
    }
    tool_calls
        .iter()
        .zip(stream_indices)
        .map(|(call, &index)| {
            let args_result =
                match parse_tool_arguments(&call.function.name, &call.function.arguments) {
                    Ok(args) => Ok(args),
                    Err(_) if truncated => Err(truncation_error(
                        &call.function.name,
                        &call.function.arguments,
                    )),
                    Err(err) => Err(err),
                };
            let (args, prebuilt_error) = match args_result {
                Ok(args) => (args, None),
                Err(err) => (Value::Object(Default::default()), Some(err)),
            };
            Ok(ToolCallPlan {
                index,
                call: call.clone(),
                args,
                prebuilt_error,
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

fn emit_tool_call<R: Runtime>(
    app: &AppHandle<R>,
    session_id: &str,
    turn_id: &str,
    plan: &ToolCallPlan,
    args: Value,
    status: &str,
) {
    emit(
        app,
        AgentEvent::ToolCall {
            session_id: session_id.to_string(),
            turn_id: turn_id.to_string(),
            id: plan.call.id.clone(),
            name: plan.call.function.name.clone(),
            args,
            status: status.to_string(),
            index: plan.index,
        },
    );
}

fn build_tool_context<'a>(
    sandbox: &'a Sandbox,
    state: &'a AppState,
    turn_ctx: &'a TurnExecutionContext,
    write_gate: Option<Arc<RuntimeWriteGate>>,
) -> ToolContext<'a> {
    ToolContext::for_turn(
        sandbox,
        Some(&state.secrets),
        &turn_ctx.project_id,
        &turn_ctx.session_id,
        &turn_ctx.turn_id,
        &turn_ctx.session_title,
        state.file_locks.clone(),
        write_gate,
        turn_ctx.profile_init,
        turn_ctx.agents_md_confirmed,
    )
}

async fn execute_one<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    sandbox: &Sandbox,
    turn_ctx: &TurnExecutionContext,
    model_id: ModelId,
    plan: &ToolCallPlan,
) -> ExecOutcome {
    let started = Instant::now();
    if let Some(err) = &plan.prebuilt_error {
        return ExecOutcome {
            ok: false,
            summary: err.to_string(),
            changed_paths: Vec::new(),
            duration_ms: started.elapsed().as_millis() as i64,
        };
    }

    let base_ctx = build_tool_context(sandbox, state, turn_ctx, None);
    let io_plan = match plan_tool_io(&base_ctx, &plan.call.function.name, &plan.args) {
        Ok(plan) => plan,
        Err(err) => {
            return ExecOutcome {
                ok: false,
                summary: err.to_json_value().to_string(),
                changed_paths: Vec::new(),
                duration_ms: started.elapsed().as_millis() as i64,
            };
        }
    };

    let lock_guard = match state.file_locks.acquire_many(
        &turn_ctx.project_id,
        &turn_ctx.session_id,
        &turn_ctx.turn_id,
        &turn_ctx.session_title,
        io_plan.locks,
    ) {
        Ok(guard) => guard,
        Err(err) => {
            return ExecOutcome {
                ok: false,
                summary: err.to_tool_json().to_string(),
                changed_paths: Vec::new(),
                duration_ms: started.elapsed().as_millis() as i64,
            };
        }
    };
    if let Err(err) = turn_ctx.file_locks.hold(lock_guard) {
        return ExecOutcome {
            ok: false,
            summary: json!({ "error": err }).to_string(),
            changed_paths: Vec::new(),
            duration_ms: started.elapsed().as_millis() as i64,
        };
    }

    let write_gate = if io_plan.dynamic_writes {
        Some(Arc::new(RuntimeWriteGate::new(
            state.file_locks.clone(),
            turn_ctx.file_locks.clone(),
            sandbox,
            turn_ctx.project_id.clone(),
            turn_ctx.session_id.clone(),
            turn_ctx.turn_id.clone(),
            turn_ctx.session_title.clone(),
            turn_ctx.profile_init,
            turn_ctx.agents_md_confirmed,
        )))
    } else {
        None
    };
    let ctx = base_ctx.with_write_gate(write_gate);

    match state
        .tools
        .execute(
            &ctx,
            app,
            model_id,
            &plan.call.function.name,
            plan.args.clone(),
        )
        .await
    {
        Ok(value) => {
            let paths = crate::tools::changed_paths::extract_changed_paths(
                &plan.call.function.name,
                &plan.args,
                &value,
            );
            ExecOutcome {
                ok: true,
                summary: value.to_string(),
                changed_paths: paths,
                duration_ms: started.elapsed().as_millis() as i64,
            }
        }
        Err(err) => {
            let value = err.to_json_value();
            ExecOutcome {
                ok: false,
                summary: value.to_string(),
                changed_paths: Vec::new(),
                duration_ms: started.elapsed().as_millis() as i64,
            }
        }
    }
}

async fn execute_pdf_read_batch<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    sandbox: &Sandbox,
    turn_ctx: &TurnExecutionContext,
    model_id: ModelId,
    batch: &[ToolCallPlan],
) -> Vec<ExecOutcome> {
    let sem = Arc::new(Semaphore::new(PDF_READ_MAX_PARALLEL));
    let futures: Vec<_> = batch
        .iter()
        .map(|plan| {
            let sem = sem.clone();
            async move {
                let _permit = sem.acquire().await.expect("pdf_read semaphore");
                execute_one(app, state, sandbox, turn_ctx, model_id, plan).await
            }
        })
        .collect();
    join_all(futures).await
}

fn bump_pending(working_messages: &[ChatMessage], pending_estimate: &mut u32) {
    if let Some(last) = working_messages.last() {
        *pending_estimate += crate::agent::compaction::estimate_chat_message_tokens(last);
    }
}

/// 将连续 `pdf_read` 名称分段，供单测验证并行分组。
pub fn pdf_read_run_segments(names: &[&str]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let mut idx = 0;
    while idx < names.len() {
        if names[idx] != "pdf_read" {
            segments.push((idx, idx + 1));
            idx += 1;
            continue;
        }
        let start = idx;
        while idx < names.len() && names[idx] == "pdf_read" {
            idx += 1;
        }
        segments.push((start, idx));
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::{FunctionCall, ModelId, ToolCall};
    use crate::core::sandbox::Sandbox;
    use crate::state::AppState;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn pdf_read_segments_group_consecutive_only() {
        let names = ["pdf_read", "pdf_read", "fs_list", "pdf_read"];
        let segments = pdf_read_run_segments(&names);
        assert_eq!(segments, [(0, 2), (2, 3), (3, 4)]);
    }

    #[test]
    fn pdf_read_max_parallel_is_three() {
        assert_eq!(PDF_READ_MAX_PARALLEL, 3);
    }

    #[test]
    fn build_plans_uses_stream_index_not_compact_position() {
        let calls = vec![sample_call("a", "pdf_read"), sample_call("b", "pdf_read")];
        let indices = [2, 4];
        let plans = build_plans(&calls, &indices, false).unwrap();
        assert_eq!(plans[0].index, 2);
        assert_eq!(plans[1].index, 4);
    }

    fn sample_call(id: &str, name: &str) -> ToolCall {
        ToolCall {
            id: id.to_string(),
            call_type: "function".into(),
            function: FunctionCall {
                name: name.to_string(),
                arguments: r#"{"path":"a.pdf"}"#.into(),
            },
        }
    }

    fn tool_call(id: &str, name: &str, args: serde_json::Value) -> ToolCall {
        ToolCall {
            id: id.to_string(),
            call_type: "function".into(),
            function: FunctionCall {
                name: name.to_string(),
                arguments: args.to_string(),
            },
        }
    }

    #[tokio::test]
    async fn write_lock_survives_execute_one_until_turn_context_drops() {
        let dir = tempdir().unwrap();
        let project_root = dir.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let sandbox = Sandbox::new(&project_root).unwrap();

        let calls_a = vec![tool_call(
            "call_a",
            "fs_write",
            json!({ "path": "out.txt", "content": "from-a" }),
        )];
        let plans_a = build_plans(&calls_a, &[0], false).unwrap();
        let turn_ctx_a = TurnExecutionContext {
            project_id: "p1".into(),
            session_id: "s-a".into(),
            turn_id: "t-a".into(),
            session_title: "A".into(),
            file_locks: crate::core::file_locks::TurnFileLockStore::new(),
            profile_init: false,
            agents_md_confirmed: false,
        };

        let outcome_a = execute_one(
            &app.handle(),
            &state,
            &sandbox,
            &turn_ctx_a,
            ModelId::Mock,
            &plans_a[0],
        )
        .await;
        assert!(
            outcome_a.ok,
            "first write should succeed: {}",
            outcome_a.summary
        );
        assert_eq!(turn_ctx_a.file_locks.guard_count(), 1);

        let calls_b = vec![tool_call(
            "call_b",
            "fs_write",
            json!({ "path": "out.txt", "content": "from-b" }),
        )];
        let plans_b = build_plans(&calls_b, &[0], false).unwrap();
        let turn_ctx_b = TurnExecutionContext {
            project_id: "p1".into(),
            session_id: "s-b".into(),
            turn_id: "t-b".into(),
            session_title: "B".into(),
            file_locks: crate::core::file_locks::TurnFileLockStore::new(),
            profile_init: false,
            agents_md_confirmed: false,
        };
        let blocked = execute_one(
            &app.handle(),
            &state,
            &sandbox,
            &turn_ctx_b,
            ModelId::Mock,
            &plans_b[0],
        )
        .await;
        assert!(!blocked.ok, "second turn should be blocked");
        assert!(blocked.summary.contains("file_busy"), "{}", blocked.summary);

        drop(turn_ctx_a);
        let unblocked = execute_one(
            &app.handle(),
            &state,
            &sandbox,
            &turn_ctx_b,
            ModelId::Mock,
            &plans_b[0],
        )
        .await;
        assert!(
            unblocked.ok,
            "lock should release with the first turn context: {}",
            unblocked.summary
        );
    }

    #[tokio::test]
    async fn cancelled_clarify_ask_receives_cancelled_tool_result() {
        let dir = tempdir().unwrap();
        let project_root = dir.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let sandbox = Sandbox::new(&project_root).unwrap();
        let session_id = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            let session = store
                .create_session(&project.id, "s1", "mock", true, "high")
                .unwrap();
            let assistant = store
                .add_message(&session.id, "assistant", Some(""), None, None, None)
                .unwrap();
            store
                .add_tool_call(
                    &assistant.id,
                    "call_mock_clarify_1",
                    "clarify_ask",
                    &json!({
                        "id": "mock_doc_type",
                        "kind": "single",
                        "prompt": "你想创建哪类文档？",
                        "options": [
                            { "id": "docx", "label": "Word 文档" },
                            { "id": "pptx", "label": "PPT 演示" }
                        ],
                        "allow_custom": true
                    })
                    .to_string(),
                )
                .unwrap();
            session.id
        };

        let tool_calls = vec![ToolCall {
            id: "call_mock_clarify_1".into(),
            call_type: "function".into(),
            function: FunctionCall {
                name: "clarify_ask".into(),
                arguments: json!({
                    "id": "mock_doc_type",
                    "kind": "single",
                    "prompt": "你想创建哪类文档？",
                    "options": [
                        { "id": "docx", "label": "Word 文档" },
                        { "id": "pptx", "label": "PPT 演示" }
                    ],
                    "allow_custom": true
                })
                .to_string(),
            },
        }];
        let cancel = CancelSignal::new();
        cancel.cancel();
        let mut working_messages = Vec::new();
        let mut pending_estimate = 0;

        let turn_ctx = TurnExecutionContext {
            project_id: "p1".into(),
            session_id: session_id.clone(),
            turn_id: "turn-1".into(),
            session_title: "Test".into(),
            file_locks: crate::core::file_locks::TurnFileLockStore::new(),
            profile_init: false,
            agents_md_confirmed: false,
        };

        let outcome = run_tool_batch(
            &app.handle(),
            &state,
            &sandbox,
            &turn_ctx,
            ModelId::Mock,
            &tool_calls,
            &[0],
            false,
            &mut working_messages,
            &mut pending_estimate,
            &cancel,
        )
        .await
        .unwrap();

        assert!(outcome.cancelled);
        assert!(!outcome.has_pending_clarify);
        let store = state.store.lock().unwrap();
        assert!(store.get_clarify_pending(&session_id).unwrap().is_none());
        let calls = store.list_tool_calls_for_session(&session_id).unwrap();
        assert_eq!(calls.len(), 1);
        assert!(calls[0]
            .result_json
            .as_deref()
            .is_some_and(|json| json.contains("cancelled")));
    }

    #[tokio::test]
    async fn clarify_phase_cancel_only_persists_remaining_clarify_tools() {
        let dir = tempdir().unwrap();
        let project_root = dir.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let sandbox = Sandbox::new(&project_root).unwrap();
        let session_id = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            let session = store
                .create_session(&project.id, "s1", "mock", true, "high")
                .unwrap();
            let assistant = store
                .add_message(&session.id, "assistant", Some(""), None, None, None)
                .unwrap();
            store
                .add_tool_call(
                    &assistant.id,
                    "call_mock_fs_1",
                    "fs_list",
                    r#"{"path":"."}"#,
                )
                .unwrap();
            store
                .add_tool_call(
                    &assistant.id,
                    "call_mock_clarify_1",
                    "clarify_ask",
                    &json!({
                        "id": "mock_doc_type",
                        "kind": "single",
                        "prompt": "你想创建哪类文档？",
                        "options": [
                            { "id": "docx", "label": "Word 文档" },
                            { "id": "pptx", "label": "PPT 演示" }
                        ],
                        "allow_custom": true
                    })
                    .to_string(),
                )
                .unwrap();
            session.id
        };

        let tool_calls = vec![
            ToolCall {
                id: "call_mock_fs_1".into(),
                call_type: "function".into(),
                function: FunctionCall {
                    name: "fs_list".into(),
                    arguments: r#"{"path":"."}"#.into(),
                },
            },
            ToolCall {
                id: "call_mock_clarify_1".into(),
                call_type: "function".into(),
                function: FunctionCall {
                    name: "clarify_ask".into(),
                    arguments: json!({
                        "id": "mock_doc_type",
                        "kind": "single",
                        "prompt": "你想创建哪类文档？",
                        "options": [
                            { "id": "docx", "label": "Word 文档" },
                            { "id": "pptx", "label": "PPT 演示" }
                        ],
                        "allow_custom": true
                    })
                    .to_string(),
                },
            },
        ];
        let plans = build_plans(&tool_calls, &[0, 1], false).unwrap();
        let mut working_messages = Vec::new();
        let mut pending_estimate = 0;

        let turn_ctx = TurnExecutionContext {
            project_id: "p1".into(),
            session_id: session_id.clone(),
            turn_id: "turn-1".into(),
            session_title: "Test".into(),
            file_locks: crate::core::file_locks::TurnFileLockStore::new(),
            profile_init: false,
            agents_md_confirmed: false,
        };

        let fs_outcome = execute_one(
            &app.handle(),
            &state,
            &sandbox,
            &turn_ctx,
            ModelId::Mock,
            &plans[0],
        )
        .await;
        persist_tool_result(
            &state,
            &app.handle(),
            &session_id,
            "turn-1",
            &plans[0].call,
            fs_outcome.ok,
            fs_outcome.summary.clone(),
            fs_outcome.duration_ms,
            fs_outcome.changed_paths,
            &mut working_messages,
        )
        .unwrap();

        let remaining: Vec<ToolCallPlan> = plans[0..]
            .iter()
            .filter(|plan| plan.call.function.name == "clarify_ask")
            .cloned()
            .collect();
        {
            let store = state.store.lock().unwrap();
            store
                .save_clarify_pending(
                    &session_id,
                    "turn-1",
                    "call_mock_clarify_1",
                    &tool_calls[1].function.arguments,
                )
                .unwrap();
        }
        persist_cancelled_clarify_remaining(
            &app.handle(),
            &state,
            &session_id,
            "turn-1",
            &remaining,
            &mut working_messages,
            &mut pending_estimate,
        )
        .unwrap();

        let store = state.store.lock().unwrap();
        assert!(store.get_clarify_pending(&session_id).unwrap().is_none());
        let calls = store.list_tool_calls_for_session(&session_id).unwrap();
        let fs_call = calls.iter().find(|call| call.name == "fs_list").unwrap();
        assert!(
            fs_call
                .result_json
                .as_deref()
                .is_some_and(|json| json.contains("entries")),
            "fs_list result must not be overwritten by clarify-phase cancel"
        );
        let clarify = calls
            .iter()
            .find(|call| call.name == "clarify_ask")
            .unwrap();
        assert!(clarify
            .result_json
            .as_deref()
            .is_some_and(|json| json.contains("cancelled")));
        let tool_messages: Vec<_> = store
            .list_messages(&session_id)
            .unwrap()
            .into_iter()
            .filter(|m| m.role == "tool")
            .collect();
        assert_eq!(tool_messages.len(), 2);
    }

    #[tokio::test]
    async fn second_clarify_in_same_batch_is_deferred_not_failed() {
        let dir = tempdir().unwrap();
        let project_root = dir.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();
        let state = AppState::new(dir.path().join("data")).unwrap();
        let app = tauri::test::mock_app();
        let sandbox = Sandbox::new(&project_root).unwrap();
        let session_id = {
            let store = state.store.lock().unwrap();
            let project = store
                .create_project("demo", project_root.to_str().unwrap())
                .unwrap();
            let session = store
                .create_session(&project.id, "s1", "mock", true, "high")
                .unwrap();
            let assistant = store
                .add_message(&session.id, "assistant", Some(""), None, None, None)
                .unwrap();
            let q1 = json!({
                "id": "q1",
                "kind": "single",
                "prompt": "问题一",
                "options": [
                    { "id": "a", "label": "A" },
                    { "id": "b", "label": "B" }
                ]
            });
            let q2 = json!({
                "id": "q2",
                "kind": "single",
                "prompt": "问题二",
                "options": [
                    { "id": "a", "label": "A" },
                    { "id": "b", "label": "B" }
                ]
            });
            store
                .add_tool_call(&assistant.id, "call_q1", "clarify_ask", &q1.to_string())
                .unwrap();
            store
                .add_tool_call(&assistant.id, "call_q2", "clarify_ask", &q2.to_string())
                .unwrap();
            session.id
        };
        let q = |id: &str, prompt: &str| {
            json!({
                "id": id,
                "kind": "single",
                "prompt": prompt,
                "options": [
                    { "id": "a", "label": "A" },
                    { "id": "b", "label": "B" }
                ]
            })
        };
        let tool_calls = vec![
            ToolCall {
                id: "call_q1".into(),
                call_type: "function".into(),
                function: FunctionCall {
                    name: "clarify_ask".into(),
                    arguments: q("q1", "问题一").to_string(),
                },
            },
            ToolCall {
                id: "call_q2".into(),
                call_type: "function".into(),
                function: FunctionCall {
                    name: "clarify_ask".into(),
                    arguments: q("q2", "问题二").to_string(),
                },
            },
        ];
        let turn_ctx = TurnExecutionContext {
            project_id: "p1".into(),
            session_id: session_id.clone(),
            turn_id: "turn-1".into(),
            session_title: "Test".into(),
            file_locks: crate::core::file_locks::TurnFileLockStore::new(),
            profile_init: false,
            agents_md_confirmed: false,
        };
        let mut working_messages = Vec::new();
        let mut pending_estimate = 0;
        let cancel = CancelSignal::new();

        let outcome = run_tool_batch(
            &app.handle(),
            &state,
            &sandbox,
            &turn_ctx,
            ModelId::Mock,
            &tool_calls,
            &[0, 1],
            false,
            &mut working_messages,
            &mut pending_estimate,
            &cancel,
        )
        .await
        .unwrap();

        assert!(outcome.has_pending_clarify);
        let store = state.store.lock().unwrap();
        let calls = store.list_tool_calls_for_session(&session_id).unwrap();
        assert_eq!(calls.len(), 2);
        let first = calls.iter().find(|c| c.id == "call_q1").unwrap();
        assert_eq!(first.status, "awaiting_user");
        let second = calls.iter().find(|c| c.id == "call_q2").unwrap();
        assert_eq!(second.status, "done");
        assert!(
            second
                .result_json
                .as_deref()
                .is_some_and(|json| json.contains("deferred")),
            "second clarify should be deferred, not failed: {:?}",
            second.result_json
        );
    }
}
