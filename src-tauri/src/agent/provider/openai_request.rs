use super::{openai_compat::apply_output_token_limit, ProviderError};
use crate::agent::types::{ChatRequest, ModelId};
use serde_json::{json, Value};

pub fn build_body(
    request: &ChatRequest,
    extra: &Value,
    stream: bool,
) -> Result<Value, ProviderError> {
    let mut body = json!({
        "model": request.model.api_model(), "messages": request.messages, "stream": stream,
    });
    if stream || !request.tools.is_empty() {
        body["tools"] = json!(request.tools.iter().map(|tool| {
            let mut function = json!({"name":tool.name,"description":tool.description,"parameters":tool.parameters});
            if tool.strict == Some(true) { function["strict"] = json!(true); }
            json!({"type":"function","function":function})
        }).collect::<Vec<_>>());
    }
    if stream {
        body["stream_options"] = json!({"include_usage":true});
    }
    if let Some(format) = &request.response_format {
        body["response_format"] = format.clone();
    }
    if let Some(limit) = request.max_tokens {
        apply_output_token_limit(
            body.as_object_mut().unwrap(),
            request.model.provider_kind(),
            limit,
        );
    }
    if let Some(extra) = extra.as_object() {
        body.as_object_mut().unwrap().extend(extra.clone());
    }
    if request.model == ModelId::Gemini38Flash {
        super::gemini::request::adapt_request(&mut body, request)?;
    }
    Ok(body)
}
