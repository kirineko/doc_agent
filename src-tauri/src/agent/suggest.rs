use crate::agent::provider::provider_for;
use crate::agent::types::{ChatMessage, ChatRequest, ModelId, ThinkingConfig, ThinkingEffort};
use crate::core::project_files::{list_project_files, recent_document_paths};
use crate::state::AppState;
use crate::tools::office::read_document_text;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;

const STARTER_MAX: usize = 4;
const FOLLOWUP_MAX: usize = 3;
const SUGGESTION_MAX_CHARS: usize = 80;
const STARTER_MAX_CHARS: usize = SUGGESTION_MAX_CHARS;
const FOLLOWUP_MAX_CHARS: usize = SUGGESTION_MAX_CHARS;
const DOC_SNIPPET_CHARS: usize = 2000;
const MSG_SNIPPET_CHARS: usize = 1000;
const FILE_LIST_MAX: usize = 50;
const LLM_TIMEOUT: Duration = Duration::from_secs(20);

pub async fn generate_suggestions(
    state: AppState,
    session_id: String,
    kind: &str,
) -> Result<Vec<String>, String> {
    let api_key = state
        .secrets
        .get_api_key("deepseek")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "suggestions disabled: DeepSeek API key not configured".to_string())?;

    let user_prompt = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let session = store
            .get_session(&session_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "session not found".to_string())?;
        let project = store
            .get_project(&session.project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "project not found".to_string())?;
        let root = PathBuf::from(&project.root_path);

        match kind {
            "starter" => build_starter_prompt(&root)?,
            "followup" => build_followup_prompt(&store, &session_id)?,
            other => return Err(format!("unknown suggestion kind: {other}")),
        }
    };

    let max_count = if kind == "starter" {
        STARTER_MAX
    } else {
        FOLLOWUP_MAX
    };

    let request = ChatRequest {
        session_id: session_id.clone(),
        turn_id: format!("suggest-{kind}"),
        model: ModelId::DeepSeekV4Flash,
        messages: vec![
            ChatMessage {
                role: "system".into(),
                content: Some(
                    "你是办公文档助手的推荐问生成器。必须输出合法 json。\
                     JSON 格式示例：{\"suggestions\":[\"分析 @报告.docx 的结构\", \"汇总 @数据.xlsx 的关键指标\"]}。\
                     不要输出除 JSON 对象以外的任何文字。"
                        .into(),
                ),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: "user".into(),
                content: Some(user_prompt),
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
        response_format: Some(json!({ "type": "json_object" })),
        max_tokens: Some(512),
    };

    let provider = provider_for(ModelId::DeepSeekV4Flash);
    let mut on_event = |_event| {};
    let result = timeout(
        LLM_TIMEOUT,
        provider.chat_stream(request, Some(&api_key), &mut on_event),
    )
    .await;

    let turn = match result {
        Ok(Ok(turn)) => turn,
        _ => return Ok(vec![]),
    };

    Ok(trim_suggestions(
        parse_suggestions_json(&turn.content),
        max_count,
        if kind == "starter" {
            STARTER_MAX_CHARS
        } else {
            FOLLOWUP_MAX_CHARS
        },
    ))
}

fn build_starter_prompt(root: &Path) -> Result<String, String> {
    let file_list = list_project_files(root);
    let paths: Vec<String> = file_list
        .entries
        .iter()
        .take(FILE_LIST_MAX)
        .map(|e| e.path.clone())
        .collect();

    let doc_paths = recent_document_paths(root, 3);

    let mut snippets = String::new();
    for path in &doc_paths {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        if let Ok(text) = read_document_text(path) {
            let clipped = truncate_chars(&text, DOC_SNIPPET_CHARS);
            snippets.push_str(&format!("\n\n### {rel}\n{clipped}"));
        }
    }

    let paths_text = paths.join("\n");

    Ok(format!(
        "基于以下项目文件清单与文档内容摘要，生成 {STARTER_MAX} 条用户最可能提出的、围绕 Word/Excel/PPT/PDF 文档分析或生成的具体问题。\n\
         要求：每条须可直接执行、尽量提及具体文件名；每条不超过 {STARTER_MAX_CHARS} 个字符（含标点与空格），超出须压缩表述；\
         必须输出 json 对象，格式为 {{\"suggestions\":[\"问题1\",\"问题2\"]}}。\n\n\
         文件清单（最多 {FILE_LIST_MAX} 条）：\n{paths_text}\n\n\
         文档摘要：{snippets}"
    ))
}

fn build_followup_prompt(
    store: &crate::core::store::Store,
    session_id: &str,
) -> Result<String, String> {
    let messages = store.list_messages(session_id).map_err(|e| e.to_string())?;
    let tool_calls = store
        .list_tool_calls_for_session(session_id)
        .map_err(|e| e.to_string())?;

    let mut history = String::new();
    for msg in messages
        .iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let content = msg.content.as_deref().unwrap_or("");
        history.push_str(&format!(
            "\n[{role}] {content}",
            role = msg.role,
            content = truncate_chars(content, MSG_SNIPPET_CHARS)
        ));
    }

    let tools: Vec<String> = tool_calls.iter().map(|t| t.name.clone()).collect();

    Ok(format!(
        "基于以下对话与工具调用足迹，生成 {FOLLOWUP_MAX} 条用户最可能继续追问的「下一步」问题。\n\
         要求：与当前上下文相关、可直接执行；每条不超过 {FOLLOWUP_MAX_CHARS} 个字符（含标点与空格），超出须压缩表述；\
         必须输出 json 对象，格式为 {{\"suggestions\":[\"问题1\",\"问题2\"]}}。\n\n\
         对话：{history}\n\n\
         工具：{tools:?}"
    ))
}

pub fn parse_suggestions_json(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    if let Ok(items) = serde_json::from_str::<Vec<String>>(trimmed) {
        return items;
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(items) = strings_from_json_value(&value) {
            return items;
        }
    }
    if let Some(inner) = strip_code_fence(trimmed) {
        if let Ok(items) = serde_json::from_str::<Vec<String>>(&inner) {
            return items;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&inner) {
            if let Some(items) = strings_from_json_value(&value) {
                return items;
            }
        }
    }
    if let Some(array_text) = extract_first_json_array(trimmed) {
        if let Ok(items) = serde_json::from_str::<Vec<String>>(&array_text) {
            return items;
        }
    }
    vec![]
}

fn strip_code_fence(text: &str) -> Option<String> {
    let start = text.find("```")?;
    let rest = &text[start + 3..];
    let rest = rest.trim_start();
    let rest = if rest.len() >= 4 && rest[..4].eq_ignore_ascii_case("json") {
        &rest[4..]
    } else {
        rest
    };
    let end = rest.find("```")?;
    Some(rest[..end].trim().to_string())
}

fn strings_from_json_value(value: &serde_json::Value) -> Option<Vec<String>> {
    if let Some(items) = value.as_array() {
        return Some(
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect(),
        );
    }
    for key in ["suggestions", "questions", "items"] {
        if let Some(items) = value.get(key).and_then(|v| v.as_array()) {
            return Some(
                items
                    .iter()
                    .filter_map(|item| {
                        item.as_str().map(str::to_string).or_else(|| {
                            item.get("text")
                                .and_then(|v| v.as_str())
                                .map(str::to_string)
                        })
                    })
                    .collect(),
            );
        }
    }
    None
}

fn extract_first_json_array(text: &str) -> Option<String> {
    let start = text.find('[')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    for (offset, ch) in text[start..].char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => depth += 1,
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(text[start..start + offset + ch.len_utf8()].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn trim_suggestions(items: Vec<String>, max_count: usize, max_chars: usize) -> Vec<String> {
    items
        .into_iter()
        .map(|s| truncate_chars(s.trim(), max_chars))
        .filter(|s| !s.is_empty())
        .take(max_count)
        .collect()
}

fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    format!("{}…", text.chars().take(max).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_array_directly() {
        let items = parse_suggestions_json(r#"["分析 a.docx", "汇总 b.xlsx"]"#);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn parses_json_inside_fence() {
        let items = parse_suggestions_json("```json\n[\"创建 PPT\"]\n```");
        assert_eq!(items, vec!["创建 PPT"]);
    }

    #[test]
    fn parses_array_embedded_in_prose() {
        let items =
            parse_suggestions_json("以下是推荐：\n[\"分析 a.docx\", \"生成汇报 PPT\"]\n希望有帮助");
        assert_eq!(items, vec!["分析 a.docx", "生成汇报 PPT"]);
    }

    #[test]
    fn parses_object_wrapped_suggestions() {
        let items = parse_suggestions_json(r#"{"suggestions":["分析 a.docx","汇总 b.xlsx"]}"#);
        assert_eq!(items, vec!["分析 a.docx", "汇总 b.xlsx"]);
    }

    #[test]
    fn bad_output_returns_empty() {
        assert!(parse_suggestions_json("not json").is_empty());
    }

    #[test]
    fn trims_length_and_count() {
        let long = "这是一段非常非常非常非常非常非常非常非常非常非常长的推荐问题";
        let out = trim_suggestions(vec![long.to_string(), "短".into()], 1, STARTER_MAX_CHARS);
        assert_eq!(out.len(), 1);
        assert!(out[0].chars().count() <= STARTER_MAX_CHARS + 1);
    }
}
