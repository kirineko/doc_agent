use crate::agent::provider::openai_compat::{
    encode_attachment_data_url, is_image_path,
};
use crate::agent::provider::provider_for;
use crate::agent::provider::ProviderError;
use crate::agent::types::{
    ChatMessage, ChatRequest, MessageAttachment, ModelId, ThinkingConfig, ThinkingEffort,
};
use crate::tools::{required_str_arg, ToolContext, ToolError};
use serde_json::{json, Value};
use std::sync::Arc;

const DEFAULT_PROMPT: &str = "请详细描述图片内容";

pub fn tool() -> crate::tools::ToolSpec {
    crate::tools::ToolSpec {
        name: "image_read",
        description: "Read an image file in the project sandbox and return a text description via vision.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project-relative path to an image file"
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional instruction for how to interpret the image"
                }
            },
            "required": ["path"],
            "additionalProperties": false
        }),
        handler: |_ctx, _args| Err(ToolError::NotImplemented),
    }
}

pub async fn handler(
    ctx: &ToolContext<'_>,
    args: Value,
    model_id: ModelId,
) -> Result<Value, ToolError> {
    if !model_id.supports_vision() {
        return Err(ToolError::Execution(
            "image_read requires a vision-capable model".into(),
        ));
    }

    let path = required_str_arg(&args, "path")?;
    if !is_image_path(&path) {
        return Err(ToolError::Execution(
            "image_read only supports image files (.png, .jpg, .jpeg, .webp, .gif); use office tools for documents"
                .into(),
        ));
    }

    let prompt = args
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_PROMPT)
        .to_string();

    let mime = mime_for_path(&path);
    let attachment = MessageAttachment {
        path: path.clone(),
        mime,
    };
    let data_url = encode_attachment_data_url(ctx.sandbox, &attachment)
        .map_err(ToolError::Execution)?;

    let api_key = ctx
        .secrets
        .and_then(|secrets| secrets.get_api_key(model_id.provider_key()).ok().flatten())
        .ok_or(ProviderError::MissingApiKey)
        .map_err(|e| ToolError::Execution(e.to_string()))?;

    let request = ChatRequest {
        session_id: "image-read".into(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        model: model_id,
        messages: vec![ChatMessage {
            role: "user".into(),
            content: Some(prompt),
            image_urls: vec![Arc::from(data_url)],
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

    Ok(json!({
        "text": turn.content,
        "path": path,
        "mime": attachment.mime,
    }))
}

fn mime_for_path(path: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn rejects_non_image_path() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(
            &ctx,
            json!({ "path": "notes.txt" }),
            ModelId::KimiK26,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("image_read only supports"));
    }

    #[tokio::test]
    async fn rejects_missing_file() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(
            &ctx,
            json!({ "path": "missing.png" }),
            ModelId::KimiK26,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("attachment path error"));
    }

    #[test]
    fn tool_is_registered_name() {
        assert_eq!(tool().name, "image_read");
    }
}
