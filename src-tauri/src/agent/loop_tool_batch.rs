use crate::agent::loop_support::{emit, persist_clarify_pending, persist_tool_result};
use crate::agent::tool_args::{parse_tool_arguments, truncation_error};
use crate::agent::types::{AgentEvent, ChatMessage, ModelId, ToolCall};
use crate::core::sandbox::Sandbox;
use crate::state::AppState;
use crate::tools::ToolContext;
use futures_util::future::join_all;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Runtime};
use tokio::sync::Semaphore;

const PDF_READ_MAX_PARALLEL: usize = 3;

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
}

#[allow(clippy::too_many_arguments)]
pub async fn run_tool_batch<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    sandbox: &Sandbox,
    session_id: &str,
    turn_id: &str,
    model_id: ModelId,
    tool_calls: &[ToolCall],
    stream_indices: &[usize],
    truncated: bool,
    working_messages: &mut Vec<ChatMessage>,
    pending_estimate: &mut u32,
) -> Result<ToolBatchOutcome, String> {
    let ctx = ToolContext::with_secrets(sandbox, &state.secrets);
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
            let outcomes = execute_pdf_read_batch(app, state, &ctx, model_id, batch).await;
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
            let outcome = execute_one(app, state, &ctx, model_id, &plans[idx]).await;
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

    for plan in &plans {
        if plan.call.function.name != "clarify_ask" {
            continue;
        }
        if plan.prebuilt_error.is_some() {
            emit_tool_call(app, session_id, turn_id, plan, plan.args.clone(), "running");
            let outcome = execute_one(app, state, &ctx, model_id, plan).await;
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
            }
            Ok(_) => {
                let value = json!({ "error": "一次只允许一个澄清问题" });
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
    })
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

async fn execute_one<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    ctx: &ToolContext<'_>,
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
    match state
        .tools
        .execute(
            ctx,
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
    ctx: &ToolContext<'_>,
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
                execute_one(app, state, ctx, model_id, plan).await
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
            function: crate::agent::types::FunctionCall {
                name: name.to_string(),
                arguments: r#"{"path":"a.pdf"}"#.into(),
            },
        }
    }
}
