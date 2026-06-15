use crate::agent::provider::openai_compat::{encode_attachment_data_url, is_image_path};
use crate::agent::provider::provider_for;
use crate::agent::provider::ProviderError;
use crate::agent::provider::openai_compat::MAX_ATTACHMENTS_PER_MESSAGE;
use crate::agent::types::{
    ChatMessage, ChatRequest, MessageAttachment, ModelId, ThinkingConfig, ThinkingEffort,
};
use crate::tools::{ToolContext, ToolError};
use serde_json::Value;
use std::sync::Arc;

pub async fn vision_subcall(
    ctx: &ToolContext<'_>,
    model_id: ModelId,
    paths: &[String],
    prompt: &str,
) -> Result<String, ToolError> {
    if !model_id.supports_vision() {
        return Err(ToolError::Execution(
            "vision subcall requires a vision-capable model".into(),
        ));
    }
    if paths.is_empty() || paths.len() > MAX_ATTACHMENTS_PER_MESSAGE {
        return Err(ToolError::InvalidArgs(format!(
            "paths must contain 1..={MAX_ATTACHMENTS_PER_MESSAGE} images"
        )));
    }

    let mut image_urls = Vec::with_capacity(paths.len());
    for path in paths {
        if !is_image_path(path) {
            return Err(ToolError::Execution(format!(
                "not an image file: {path}"
            )));
        }
        let attachment = MessageAttachment {
            path: path.clone(),
            mime: mime_for_path(path),
        };
        let data_url = encode_attachment_data_url(ctx.sandbox, &attachment)
            .map_err(ToolError::Execution)?;
        image_urls.push(Arc::from(data_url));
    }

    let api_key = ctx
        .secrets
        .and_then(|secrets| secrets.get_api_key(model_id.provider_key()).ok().flatten())
        .ok_or(ProviderError::MissingApiKey)
        .map_err(|e| ToolError::Execution(e.to_string()))?;

    let request = ChatRequest {
        session_id: "vision-subcall".into(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        model: model_id,
        messages: vec![ChatMessage {
            role: "user".into(),
            content: Some(prompt.to_string()),
            image_urls,
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: vec![],
        thinking: ThinkingConfig {
            enabled: false,
            effort: ThinkingEffort::High,
        },
        response_format: None,
        max_tokens: None,
    };

    let provider = provider_for(model_id);
    let turn = provider
        .chat_stream(request, Some(&api_key), &mut |_| {})
        .await
        .map_err(|e| ToolError::Execution(e.to_string()))?;

    Ok(turn.content)
}

pub fn parse_paths_arg(args: &Value) -> Result<Vec<String>, ToolError> {
    let Some(arr) = args.get("paths").and_then(|v| v.as_array()) else {
        return Err(ToolError::InvalidArgs("paths required".into()));
    };
    if arr.is_empty() || arr.len() > MAX_ATTACHMENTS_PER_MESSAGE {
        return Err(ToolError::InvalidArgs(format!(
            "paths must contain 1..={MAX_ATTACHMENTS_PER_MESSAGE} items"
        )));
    }
    arr.iter()
        .map(|v| {
            v.as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .ok_or_else(|| ToolError::InvalidArgs("paths must be strings".into()))
        })
        .collect()
}

pub fn mime_for_path(path: &str) -> String {
    match std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png".into(),
        Some("jpg") | Some("jpeg") => "image/jpeg".into(),
        Some("webp") => "image/webp".into(),
        Some("gif") => "image/gif".into(),
        _ => "image/png".into(),
    }
}
