use super::request::{GOOGLE_CHAT_URL, MAX_REQUEST_BYTES};
use super::state::GoogleReplayState;
use crate::agent::provider::{
    openai_compat::extra_body_for, openai_request::build_body, openai_stream::Accumulator, sse,
};
use crate::agent::types::{ChatMessage, ChatRequest, ModelId, ThinkingConfig, ThinkingEffort};
use serde_json::{json, Value};

pub(super) fn request() -> ChatRequest {
    ChatRequest {
        session_id: "s".into(),
        turn_id: "t".into(),
        model: ModelId::Gemini38Flash,
        messages: vec![message("user", "hello")],
        tools: vec![],
        thinking: ThinkingConfig {
            enabled: true,
            effort: ThinkingEffort::Medium,
        },
        response_format: None,
        max_tokens: None,
        cancel: None,
    }
}

pub(super) fn message(role: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: role.into(),
        content: Some(content.into()),
        image_urls: vec![],
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        provider_state: None,
    }
}

pub(super) fn call(id: &str, args: &str, signature: Option<&str>) -> Value {
    let mut value =
        json!({"id":id,"type":"function","function":{"name":"lookup_probe","arguments":args}});
    if let Some(signature) = signature {
        value["extra_content"] = json!({"google":{"thought_signature":signature}});
    }
    value
}

pub(super) fn chunk(delta: Value, reason: Option<&str>) -> Value {
    json!({"choices":[{"index":0,"delta":delta,"finish_reason":reason}]})
}

pub(super) fn turn(calls: Vec<Value>) -> crate::agent::types::AssistantTurn {
    let mut acc = Accumulator::new(Some("gemini-3.8-flash"));
    acc.apply(&chunk(
        json!({"content":"before","tool_calls":calls}),
        Some("stop"),
    ))
    .unwrap();
    acc.finish().unwrap()
}

#[test]
fn request_uses_compat_parameters_and_all_catalog_schemas_unchanged() {
    let mut req = request();
    req.tools =
        crate::tools::ToolRegistry::default_tools().tools_for_model(ModelId::Gemini38Flash, true);
    req.max_tokens = Some(1024);
    req.response_format = Some(json!({"type":"json_object"}));
    let body = build_body(&req, &extra_body_for(&req), true).unwrap();
    assert_eq!(
        GOOGLE_CHAT_URL,
        "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
    );
    assert_eq!(body["messages"][0]["role"], "user");
    assert_eq!(body["max_tokens"], 1024);
    assert_eq!(
        body["extra_body"]["google"]["thinking_config"]["thinking_level"],
        "medium"
    );
    assert_eq!(body["response_format"]["type"], "json_object");
    for key in [
        "input",
        "steps",
        "store",
        "generation_config",
        "reasoning_effort",
    ] {
        assert!(body.get(key).is_none());
    }
    for (tool, wire) in req.tools.iter().zip(body["tools"].as_array().unwrap()) {
        assert_eq!(wire["function"]["parameters"], tool.parameters);
    }
    let non_stream = build_body(&req, &extra_body_for(&req), false).unwrap();
    assert_eq!(non_stream["messages"], body["messages"]);
    assert_eq!(non_stream["max_tokens"], body["max_tokens"]);
}

#[test]
fn images_and_limits_use_complete_serialized_body() {
    let mut req = request();
    req.messages[0].image_urls = vec![std::sync::Arc::from("data:image/png;base64,cGl4ZWw=")];
    let body = build_body(&req, &extra_body_for(&req), true).unwrap();
    assert_eq!(body["messages"][0]["content"][1]["type"], "image_url");
    assert!(body.get("max_tokens").is_none());
    req.messages
        .push(message("user", &"x".repeat(MAX_REQUEST_BYTES)));
    assert!(build_body(&req, &extra_body_for(&req), true).is_err());
    req.messages.truncate(1);
    req.thinking.enabled = false;
    assert!(build_body(&req, &extra_body_for(&req), true).is_err());
    req.thinking.enabled = true;
    req.thinking.effort = ThinkingEffort::Max;
    assert!(build_body(&req, &extra_body_for(&req), true).is_err());
    req.thinking.effort = ThinkingEffort::Low;
    req.max_tokens = Some(65_537);
    assert!(build_body(&req, &extra_body_for(&req), true).is_err());
}

#[test]
fn metadata_is_small_and_opaque_and_only_google_replays_it() {
    let turn = turn(vec![call("wire", "{}", Some("secret-signature"))]);
    assert!(!format!("{turn:?}").contains("secret-signature"));
    let raw = turn.provider_state.clone().unwrap();
    let state: Value = serde_json::from_str(&raw).unwrap();
    assert!(state.get("steps").is_none());
    assert!(!raw.contains("before"));
    assert!(!raw.contains("wire"));
    let mut req = request();
    let mut msg = message("assistant", &turn.content);
    msg.tool_calls = Some(turn.tool_calls);
    msg.provider_state = Some(raw);
    msg.reasoning_content = Some("visible thought".into());
    assert!(!format!("{msg:?}").contains("secret-signature"));
    req.messages.push(msg);
    let body = build_body(&req, &extra_body_for(&req), true).unwrap();
    assert_eq!(
        body["messages"][1]["tool_calls"][0]["extra_content"]["google"]["thought_signature"],
        "secret-signature"
    );
    assert!(body["messages"][1].get("reasoning_content").is_none());
    req.model = ModelId::DeepSeekV4Flash;
    let old = build_body(&req, &extra_body_for(&req), true).unwrap();
    assert!(!old.to_string().contains("secret-signature"));
    assert_eq!(old["messages"][1]["reasoning_content"], "visible thought");
}

#[test]
fn metadata_missing_malformed_and_wrong_version_fail_without_echo() {
    let mut req = request();
    req.messages.push(message("assistant", "plain answer"));
    assert!(build_body(&req, &extra_body_for(&req), true).is_ok());
    let turn = turn(vec![call("wire", "{}", Some("secret"))]);
    req.messages[1].tool_calls = Some(turn.tool_calls);
    assert!(build_body(&req, &extra_body_for(&req), true).is_err());
    for raw in ["secret", r#"{"protocol":"secret","version":99}"#] {
        req.messages[1].provider_state = Some(raw.into());
        let error = build_body(&req, &extra_body_for(&req), true)
            .unwrap_err()
            .to_string();
        assert!(!error.contains("secret"));
    }
    let empty = GoogleReplayState::new("gemini-3.8-flash");
    req.messages[1].provider_state = Some(empty.to_json().unwrap());
    assert!(build_body(&req, &extra_body_for(&req), true).is_err());
}

#[tokio::test]
async fn byte_stream_normalizes_parallel_calls_thoughts_usage_and_progress() {
    let values = [
        chunk(
            json!({"content":"<thou","extra_content":{"google":{"thought":true}}}),
            None,
        ),
        chunk(
            json!({"content":"ght>思考</thou","extra_content":{"google":{"thought":true}}}),
            None,
        ),
        chunk(json!({"content":"ght>正文"}), None),
        chunk(
            json!({"tool_calls":[call("a", "{\"name\":", Some("sig-a"))]}),
            None,
        ),
        chunk(
            json!({"tool_calls":[call("b", "{\"name\":\"beta\"}", None)]}),
            None,
        ),
        chunk(
            json!({"tool_calls":[{"id":"a","function":{"arguments":"\"alpha\"}"}}]}),
            Some("stop"),
        ),
        json!({"choices":[],"usage":{"prompt_tokens":38,"completion_tokens":345,"total_tokens":1353}}),
    ];
    let data = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            format!(
                ": comment\n{}{}{}",
                if i % 2 == 0 { "data:" } else { "data: " },
                v,
                if i % 2 == 0 { "\r\n\r\n" } else { "\n\n" }
            )
        })
        .collect::<String>()
        + "data: [DONE]\n\n";
    let stream = futures_util::stream::iter(
        data.as_bytes()
            .chunks(1)
            .map(|b| Ok::<_, String>(b.to_vec())),
    );
    let mut reasoning = String::new();
    let mut content = String::new();
    let mut progress = Vec::new();
    let turn = sse::consume_stream(stream, None, Some("gemini-3.8-flash"), |r, c, t| {
        reasoning.push_str(r.unwrap_or_default());
        content.push_str(c.unwrap_or_default());
        if let Some(t) = t {
            progress.extend_from_slice(t);
        }
    })
    .await
    .unwrap();
    assert_eq!(turn.reasoning_content, "思考");
    assert_eq!(turn.content, "正文");
    assert_eq!(reasoning, "思考");
    assert_eq!(content, "正文");
    assert_eq!(turn.tool_calls.len(), 2);
    assert_eq!(turn.tool_calls[0].function.arguments, r#"{"name":"alpha"}"#);
    assert_eq!(turn.usage.unwrap().completion, 1315);
    assert_eq!(
        progress
            .iter()
            .map(|p| p["index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![0, 1, 0]
    );
    assert!(!format!("{progress:?}").contains("sig-a"));
}

#[test]
fn truncation_drops_complete_and_partial_tools_before_validation() {
    for args in ["{}", "{\"partial\":"] {
        let mut acc = Accumulator::new(Some("gemini-3.8-flash"));
        acc.apply(&chunk(
            json!({"content":"partial answer","tool_calls":[call("a", args, None)]}),
            Some("length"),
        ))
        .unwrap();
        let turn = acc.finish().unwrap();
        assert!(turn.content.contains("partial answer"));
        assert!(turn.content.contains("截断"));
        assert_eq!(turn.finish_reason.as_deref(), Some("incomplete"));
        assert!(turn.tool_calls.is_empty());
        let state =
            GoogleReplayState::parse(turn.provider_state.as_deref().unwrap(), "gemini-3.8-flash")
                .unwrap();
        assert!(state.tool_extra_content.is_empty());
    }
}

#[tokio::test]
async fn missing_terminal_errors_and_stalled_cancellation() {
    let raw = format!("data: {}\n\n", chunk(json!({"content":"unfinished"}), None));
    let result = sse::consume_stream(
        futures_util::stream::iter([Ok::<_, String>(raw)]),
        None,
        Some("gemini-3.8-flash"),
        |_, _, _| {},
    )
    .await;
    assert!(result.is_err());
    let cancel = crate::agent::turn_control::CancelSignal::new();
    let stop = async {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        cancel.cancel();
    };
    let read = sse::consume_stream(
        futures_util::stream::pending::<Result<Vec<u8>, String>>(),
        Some(&cancel),
        Some("gemini-3.8-flash"),
        |_, _, _| {},
    );
    let (result, ()) = tokio::time::timeout(std::time::Duration::from_secs(1), async {
        tokio::join!(read, stop)
    })
    .await
    .unwrap();
    assert!(matches!(result, Err(sse::SseError::Cancelled)));
}

#[test]
fn complete_response_separates_summary_and_keeps_signature() {
    let turn = crate::agent::provider::openai_stream::parse_complete(&json!({
        "choices":[{"message":{"role":"assistant","content":"<thought>reason</thought>answer","extra_content":{"google":{"thought":true,"thought_signature":"opaque"}}},"finish_reason":"stop"}]
    }), Some("gemini-3.8-flash")).unwrap();
    assert_eq!(turn.content, "answer");
    assert!(turn.is_complete_text());
    let mut rejected = turn.clone();
    for reason in [None, Some("length"), Some("incomplete"), Some("tool_calls")] {
        rejected.finish_reason = reason.map(str::to_string);
        assert!(!rejected.is_complete_text());
    }
    assert_eq!(turn.reasoning_content, "reason");
    assert!(turn.provider_state.unwrap().contains("opaque"));
    let mut mixed = super::stream::ThoughtText::default();
    assert_eq!(
        mixed.push("<thought>reason</thought>Use <thought> literally.", true),
        ("reason".into(), "Use <thought> literally.".into())
    );
    let mut text = super::stream::ThoughtText::default();
    assert_eq!(
        text.push("Use <thought> as a literal tag.", false).1,
        "Use <thought> as a literal tag."
    );
}

#[test]
fn invalid_calls_and_conflicting_signatures_are_rejected() {
    for calls in [
        vec![call("a", "{}", None)],
        vec![call("a", "{", Some("sig"))],
        vec![call("a", "[]", Some("sig"))],
    ] {
        let mut acc = Accumulator::new(Some("gemini-3.8-flash"));
        acc.apply(&chunk(json!({"tool_calls":calls}), Some("stop")))
            .unwrap();
        assert!(acc.finish().is_err());
    }
    let mut acc = Accumulator::new(Some("gemini-3.8-flash"));
    acc.apply(&chunk(
        json!({"tool_calls":[call("a","{}",Some("sig")),call("b","{}",None)]}),
        None,
    ))
    .unwrap();
    assert!(acc
        .apply(&chunk(
            json!({"tool_calls":[{"function":{"arguments":"more"}}]}),
            None
        ))
        .is_err());
    assert!(acc
        .apply(&chunk(
            json!({"tool_calls":[call("a","",Some("different"))]}),
            None
        ))
        .is_err());
    let mut acc = Accumulator::new(Some("gemini-3.8-flash"));
    assert!(acc
        .apply(&json!({"error":{"message":"secret"}}))
        .unwrap_err()
        .to_string()
        .contains("error"));
}

#[tokio::test]
async fn multiline_frames_and_done_without_finish_are_handled() {
    let valid = "data:{\"choices\":\n\ndata: [DONE]\n\n";
    let result = sse::consume_stream(
        futures_util::stream::iter([Ok::<_, String>(valid)]),
        None,
        None,
        |_, _, _| {},
    )
    .await;
    assert!(result.is_err());
    let valid="data:{\"choices\":\ndata:[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":\"stop\"}]}\r\n\r\n";
    let result = sse::consume_stream(
        futures_util::stream::iter([Ok::<_, String>(valid)]),
        None,
        None,
        |_, _, _| {},
    )
    .await
    .unwrap();
    assert_eq!(result.content, "ok");
    let result = sse::consume_stream(
        futures_util::stream::iter([Ok::<_, String>("data: [DONE]\n\n")]),
        None,
        Some("gemini-3.8-flash"),
        |_, _, _| {},
    )
    .await;
    assert!(result.is_err());
}
