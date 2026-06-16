use crate::agent::provider::{openai_compat::model_from_str, provider_for};
use crate::agent::session_title::truncate_for_storage;
use crate::agent::types::{
    AgentEvent, ChatMessage, ChatRequest, ModelId, ThinkingConfig, ThinkingEffort,
};
use crate::core::store::Message;
use crate::state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};
use tokio::time::timeout;

const MSG_SNIPPET_CHARS: usize = 500;
const LLM_TIMEOUT: Duration = Duration::from_secs(15);
const SUGGESTED_TITLE_CHARS: usize = 40;

pub fn build_title_prompt(snippets: &str) -> String {
    format!(
        "根据以下两轮对话，生成一个简短的会话标题（单行，不超过 {SUGGESTED_TITLE_CHARS} 个字符）。\
         概括用户意图，不要引号、不要 Markdown、不要句号结尾。只输出标题本身，不要其他文字。\n\n\
         对话：{snippets}"
    )
}

pub fn snippets_from_first_two_rounds(messages: &[Message]) -> String {
    let chat: Vec<&Message> = messages
        .iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .collect();

    let mut output = String::new();
    let mut round = 0usize;
    let mut i = 0usize;

    while i < chat.len() && round < 2 {
        while i < chat.len() && chat[i].role != "user" {
            i += 1;
        }
        if i >= chat.len() {
            break;
        }
        round += 1;
        let user = chat[i].content.as_deref().unwrap_or("");
        output.push_str(&format!(
            "\n[用户] {}",
            truncate_snippet(user, MSG_SNIPPET_CHARS)
        ));
        i += 1;

        while i < chat.len() && chat[i].role != "assistant" {
            i += 1;
        }
        if i < chat.len() {
            let assistant = chat[i].content.as_deref().unwrap_or("");
            output.push_str(&format!(
                "\n[助手] {}",
                truncate_snippet(assistant, MSG_SNIPPET_CHARS)
            ));
            i += 1;
        }
    }

    output
}

pub fn clean_generated_title(text: &str) -> Option<String> {
    let mut s = text.trim().to_string();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('"') && s.ends_with('"') && s.chars().count() >= 2 {
        s = s[1..s.len() - 1].trim().to_string();
    }
    if s.starts_with('\'') && s.ends_with('\'') && s.chars().count() >= 2 {
        s = s[1..s.len() - 1].trim().to_string();
    }
    s = s
        .trim_end_matches(['。', '.', '！', '!', '？', '?'])
        .to_string();
    s = s.replace('\n', " ").trim().to_string();
    if s.is_empty() || s == "新会话" {
        return None;
    }
    let title = truncate_for_storage(&s);
    if title == "新会话" {
        return None;
    }
    Some(title)
}

fn truncate_snippet(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    format!("{}…", text.chars().take(max).collect::<String>())
}

pub async fn generate_session_title(
    state: &AppState,
    session_id: &str,
    model: &str,
) -> Option<String> {
    let model_id = model_from_str(model);
    let api_key = if model_id == ModelId::Mock {
        None
    } else {
        state
            .secrets
            .get_api_key(model_id.provider_key())
            .ok()
            .flatten()
    };
    if model_id != ModelId::Mock && api_key.is_none() {
        return None;
    }

    let snippets = {
        let store = state.store.lock().ok()?;
        let messages = store.list_messages(session_id).ok()?;
        snippets_from_first_two_rounds(&messages)
    };
    if snippets.trim().is_empty() {
        return None;
    }

    let request = ChatRequest {
        session_id: session_id.to_string(),
        turn_id: "autotitle".into(),
        model: model_id,
        messages: vec![
            ChatMessage {
                role: "system".into(),
                content: Some("你是会话标题生成器。根据对话摘要输出一行简短中文标题。".into()),
                image_urls: vec![],
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: "user".into(),
                content: Some(build_title_prompt(&snippets)),
                image_urls: vec![],
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        tools: vec![],
        thinking: ThinkingConfig {
            enabled: false,
            effort: ThinkingEffort::High,
        },
        response_format: None,
        max_tokens: Some(64),
    };

    let provider = provider_for(model_id);
    let mut on_event = |_event| {};
    let result = timeout(
        LLM_TIMEOUT,
        provider.chat_stream(request, api_key.as_deref(), &mut on_event),
    )
    .await;

    let turn = match result {
        Ok(Ok(turn)) => turn,
        _ => return None,
    };

    clean_generated_title(&turn.content)
}

pub fn spawn_llm_session_title<R: Runtime>(app: AppHandle<R>, state: AppState, session_id: String) {
    tokio::spawn(async move {
        let (model, claimed) = {
            let store = state.store.lock().map_err(|e| e.to_string());
            let Ok(store) = store else {
                return;
            };
            let Some(session) = store.get_session(&session_id).ok().flatten() else {
                return;
            };
            let claimed = store.claim_autotitle_llm(&session_id).unwrap_or(false);
            (session.model, claimed)
        };
        if !claimed {
            return;
        }

        let Some(title) = generate_session_title(&state, &session_id, &model).await else {
            return;
        };

        let updated = {
            let store = state.store.lock().map_err(|e| e.to_string());
            let Ok(store) = store else {
                return;
            };
            store
                .finish_llm_autotitle(&session_id, &title)
                .unwrap_or(false)
        };
        if !updated {
            return;
        }

        let _ = app.emit(
            "agent-event",
            AgentEvent::SessionTitleUpdated {
                session_id: session_id.clone(),
                title,
            },
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::session_title::MAX_STORED_TITLE_CHARS;

    #[test]
    fn cleans_quotes_and_whitespace() {
        assert_eq!(
            clean_generated_title("\"分析课程资料\"").as_deref(),
            Some("分析课程资料")
        );
        assert_eq!(clean_generated_title("  "), None);
    }

    #[test]
    fn builds_snippets_from_two_rounds() {
        let messages = vec![
            Message {
                id: "1".into(),
                session_id: "s".into(),
                role: "user".into(),
                content: Some("你好".into()),
                reasoning_content: None,
                tool_call_id: None,
                seq: 1,
                created_at: String::new(),
                archived: false,
                attachments_json: None,
            },
            Message {
                id: "2".into(),
                session_id: "s".into(),
                role: "assistant".into(),
                content: Some("你好，有什么可以帮你？".into()),
                reasoning_content: None,
                tool_call_id: None,
                seq: 2,
                created_at: String::new(),
                archived: false,
                attachments_json: None,
            },
            Message {
                id: "3".into(),
                session_id: "s".into(),
                role: "user".into(),
                content: Some("分析 SK1002 归档".into()),
                reasoning_content: None,
                tool_call_id: None,
                seq: 3,
                created_at: String::new(),
                archived: false,
                attachments_json: None,
            },
            Message {
                id: "4".into(),
                session_id: "s".into(),
                role: "assistant".into(),
                content: Some("好的，我来分析。".into()),
                reasoning_content: None,
                tool_call_id: None,
                seq: 4,
                created_at: String::new(),
                archived: false,
                attachments_json: None,
            },
        ];
        let snippets = snippets_from_first_two_rounds(&messages);
        assert!(snippets.contains("SK1002"));
        assert!(snippets.contains("[用户]"));
        assert!(snippets.contains("[助手]"));
    }

    #[test]
    fn truncates_long_generated_titles() {
        let long = "标".repeat(MAX_STORED_TITLE_CHARS + 10);
        let title = clean_generated_title(&long).expect("title");
        assert!(title.chars().count() <= MAX_STORED_TITLE_CHARS + 1);
    }
}
