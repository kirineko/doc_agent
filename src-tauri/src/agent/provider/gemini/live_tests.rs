//! Opt-in tests exercise the product provider and store using only synthetic content.
use super::{
    lifecycle_tests::{persist, reload, result, setup},
    tests::request,
    GeminiProvider,
};
use crate::agent::provider::LlmProvider;
use crate::agent::types::{AssistantTurn, ChatRequest, ThinkingEffort, ToolDefinition};
use crate::core::store::Store;
use serde_json::{json, Value};

async fn run(req: ChatRequest, key: &str) -> AssistantTurn {
    let turn = GeminiProvider
        .chat_stream(req, Some(key), &mut |_| {})
        .await
        .unwrap_or_else(|_| panic!("Google compatible product provider request failed"));
    assert_ne!(turn.finish_reason.as_deref(), Some("incomplete"));
    turn
}

fn key() -> Option<String> {
    if std::env::var("DOC_AGENT_GOOGLE_SMOKE").as_deref() != Ok("1") {
        eprintln!("SKIP Google live smoke: DOC_AGENT_GOOGLE_SMOKE is not 1");
        return None;
    }
    let key = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY required for opted-in smoke");
    assert!(!key.is_empty(), "GOOGLE_API_KEY must not be empty");
    Some(key)
}

fn tools() -> Vec<ToolDefinition> {
    vec![ToolDefinition {
        name: "lookup_probe".into(),
        description: "Look up a synthetic value by name.".into(),
        parameters: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
        strict: None,
    }]
}

#[tokio::test]
async fn live_google_parallel_store_restart_and_followup() {
    let Some(key) = key() else {
        return;
    };
    let (dir, store, session) = setup();
    store.add_message(&session,"user",Some("First call lookup_probe twice in parallel for alpha and beta. After receiving both results call it for gamma. Finally report all three values. Never guess values."),None,None,None).unwrap();
    let mut req = reload(&store, &session);
    req.tools = tools();
    req.thinking.effort = ThinkingEffort::Low;
    let mut first = run(req, &key).await;
    assert_eq!(first.tool_calls.len(), 2, "expected two parallel calls");
    let original_ids: Vec<_> = first.tool_calls.iter().map(|c| c.id.clone()).collect();
    crate::agent::loop_support::normalize_tool_call_ids(&mut first.tool_calls, |id| {
        original_ids.iter().any(|original| original == id)
    });
    persist(&store, &session, &first);
    for call in &first.tool_calls {
        let args: Value = serde_json::from_str(&call.function.arguments).unwrap();
        let text = match args["name"].as_str() {
            Some("alpha") => "A17",
            Some("beta") => "B29",
            _ => panic!("unexpected probe argument"),
        };
        result(&store, &session, &call.id, text);
    }
    drop(store);
    let store = Store::open(dir.path().join("test.db")).unwrap();
    let mut req = reload(&store, &session);
    req.tools = tools();
    req.thinking.effort = ThinkingEffort::Low;
    let third = run(req, &key).await;
    assert_eq!(third.tool_calls.len(), 1);
    assert_eq!(
        serde_json::from_str::<Value>(&third.tool_calls[0].function.arguments).unwrap()["name"],
        "gamma"
    );
    persist(&store, &session, &third);
    result(&store, &session, &third.tool_calls[0].id, "NOT_FOUND");
    let mut req = reload(&store, &session);
    req.tools = tools();
    req.thinking.effort = ThinkingEffort::Low;
    let final_turn = run(req, &key).await;
    assert!(final_turn.tool_calls.is_empty());
    for value in ["A17", "B29", "NOT_FOUND"] {
        assert!(
            final_turn.content.contains(value),
            "missing synthetic result"
        );
    }
    persist(&store, &session, &final_turn);
    store
        .add_message(
            &session,
            "user",
            Some("Repeat only the alpha value."),
            None,
            None,
            None,
        )
        .unwrap();
    let mut req = reload(&store, &session);
    req.thinking.effort = ThinkingEffort::Low;
    assert!(run(req, &key).await.content.contains("A17"));
}

#[tokio::test]
async fn live_google_thoughts_and_image() {
    let Some(key) = key() else { return };
    let mut req = request();
    req.max_tokens = Some(4096);
    req.thinking.effort = ThinkingEffort::High;
    req.messages[0].content=Some("A bag has 5 red, 4 blue and 3 green balls. Draw 4 without replacement. Calculate the probability all three colors appear. Give a brief calculation.".into());
    let turn = run(req, &key).await;
    assert!(!turn.content.is_empty());
    assert!(!turn.reasoning_content.is_empty());
    assert!(!turn.content.contains("<thought>"));
    assert!(!turn.reasoning_content.contains("</thought>"));
    let mut req = request();
    req.thinking.effort = ThinkingEffort::Low;
    req.messages[0].content = Some("What is the solid image color? Reply only red.".into());
    let pixel = image::RgbImage::from_pixel(32, 32, image::Rgb([255, 0, 0]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    pixel.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    use base64::Engine;
    req.messages[0]
        .image_urls
        .push(std::sync::Arc::from(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())
        )));
    assert!(run(req, &key).await.content.to_lowercase().contains("red"));
}
