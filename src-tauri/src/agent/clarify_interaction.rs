use crate::agent::loop_runner;
use crate::agent::types::{AgentEvent, ClarifyAnswer, ClarifyQuestion};
use crate::state::AppState;
use serde_json::json;
use tauri::{AppHandle, Emitter, Runtime};

#[derive(Debug, Clone)]
pub struct SubmitClarifyAnswer {
    pub session_id: String,
    pub question_id: String,
    pub selected: Vec<String>,
    pub custom: Option<String>,
}

pub async fn submit_clarify_answer<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    answer: SubmitClarifyAnswer,
) -> Result<(), String> {
    // 删 pending 与写 tool result 必须在同一锁块内完成：
    // 中间释放锁会留下「pending 已删但 tool_call 无 result」的窗口，
    // 期间 send_message 的 pending 检查会放行，插入的 user 消息将打断
    // assistant(tool_calls) → tool 序列导致 API 400。
    let (pending, result_json) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let pending = store
            .get_clarify_pending(&answer.session_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "澄清问题已处理或不存在".to_string())?;
        let question: ClarifyQuestion =
            serde_json::from_str(&pending.question_json).map_err(|e| e.to_string())?;
        if question.id != answer.question_id {
            return Err("澄清问题不匹配".into());
        }
        validate_clarify_answer(&question, &answer)?;
        let deleted = store
            .delete_clarify_pending(&answer.session_id)
            .map_err(|e| e.to_string())?;
        if deleted == 0 {
            return Err("澄清问题已处理或不存在".into());
        }

        let display_text = clarify_display_text(&question, &answer);
        let confirmed = answer.selected.iter().any(|v| v == "confirm");
        let brief = if question.kind == "confirm_brief" && confirmed {
            question.brief.clone()
        } else {
            None
        };
        let result = ClarifyAnswer {
            question_id: answer.question_id,
            selected: answer.selected,
            custom: answer.custom,
            display_text,
            brief,
        };
        let result_json = serde_json::to_string(&result).map_err(|e| e.to_string())?;
        store
            .finish_tool_call(&pending.tool_call_id, &result_json, "done", 0)
            .map_err(|e| e.to_string())?;
        store
            .add_message(
                &pending.session_id,
                "tool",
                Some(&result_json),
                None,
                Some(&pending.tool_call_id),
            )
            .map_err(|e| e.to_string())?;
        (pending, result_json)
    };
    emit_tool_result(
        &app,
        &pending.session_id,
        &pending.turn_id,
        &pending.tool_call_id,
        result_json,
    );

    loop_runner::resume_turn(app, state, pending.session_id, pending.turn_id).await
}

pub async fn cancel_clarify<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    session_id: String,
) -> Result<(), String> {
    // 与 submit 相同：删 pending 与写 tool result 在同一锁块内，避免竞态窗口
    let result_json = json!({ "cancelled": true }).to_string();
    let pending = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let pending = store
            .get_clarify_pending(&session_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "澄清问题已处理或不存在".to_string())?;
        let deleted = store
            .delete_clarify_pending(&session_id)
            .map_err(|e| e.to_string())?;
        if deleted == 0 {
            return Err("澄清问题已处理或不存在".into());
        }
        store
            .finish_tool_call(&pending.tool_call_id, &result_json, "done", 0)
            .map_err(|e| e.to_string())?;
        store
            .add_message(
                &pending.session_id,
                "tool",
                Some(&result_json),
                None,
                Some(&pending.tool_call_id),
            )
            .map_err(|e| e.to_string())?;
        pending
    };
    emit_tool_result(
        &app,
        &pending.session_id,
        &pending.turn_id,
        &pending.tool_call_id,
        result_json,
    );
    loop_runner::resume_turn(app, state, pending.session_id, pending.turn_id).await
}

fn validate_clarify_answer(
    question: &ClarifyQuestion,
    answer: &SubmitClarifyAnswer,
) -> Result<(), String> {
    let custom = answer.custom.as_deref().map(str::trim).unwrap_or("");
    let option_ids: std::collections::HashSet<_> =
        question.options.iter().map(|o| o.id.as_str()).collect();
    for selected in &answer.selected {
        if !option_ids.contains(selected.as_str()) && selected != "confirm" {
            return Err(format!("未知选项: {selected}"));
        }
    }
    match question.kind.as_str() {
        "single" => {
            if answer.selected.len() > 1 {
                return Err("单选题只能选择一个选项".into());
            }
            if answer.selected.is_empty() && custom.is_empty() {
                return Err("请选择一个选项或填写自定义内容".into());
            }
        }
        "multi" => {
            let count = answer.selected.len() + usize::from(!custom.is_empty());
            if let Some(min) = question.min_selections {
                if count < min {
                    return Err(format!("至少选择 {min} 项"));
                }
            }
            if let Some(max) = question.max_selections {
                if count > max {
                    return Err(format!("最多选择 {max} 项"));
                }
            }
            if count == 0 {
                return Err("请至少选择或填写一项".into());
            }
        }
        "text" => {
            if custom.is_empty() {
                return Err("请填写回答内容".into());
            }
        }
        "confirm_brief" => {
            if answer.selected.iter().any(|v| v == "confirm") {
                return Ok(());
            }
            if custom.is_empty() {
                return Err("请确认或填写修改意见".into());
            }
        }
        _ => return Err("未知澄清题型".into()),
    }
    Ok(())
}

fn clarify_display_text(question: &ClarifyQuestion, answer: &SubmitClarifyAnswer) -> String {
    let mut parts: Vec<String> = answer
        .selected
        .iter()
        .filter_map(|id| {
            if id == "confirm" {
                Some("确认继续".to_string())
            } else {
                question
                    .options
                    .iter()
                    .find(|option| option.id == *id)
                    .map(|option| option.label.clone())
            }
        })
        .collect();
    if let Some(custom) = answer
        .custom
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(custom.to_string());
    }
    parts.join("、")
}

fn emit_tool_result<R: Runtime>(
    app: &AppHandle<R>,
    session_id: &str,
    turn_id: &str,
    tool_call_id: &str,
    summary: String,
) {
    let _ = app.emit(
        "agent-event",
        AgentEvent::ToolResult {
            session_id: session_id.to_string(),
            turn_id: turn_id.to_string(),
            id: tool_call_id.to_string(),
            ok: true,
            summary,
            duration_ms: 0,
            changed_paths: Vec::new(),
        },
    );
}
