use crate::agent::types::ModelId;
use crate::tools::vision_subcall::{parse_paths_arg, vision_subcall, mime_for_path};
use crate::tools::{ToolContext, ToolError};
use serde_json::{json, Value};

const DEFAULT_PROMPT: &str = "请详细描述图片内容";

pub fn tool() -> crate::tools::ToolSpec {
    crate::tools::ToolSpec {
        name: "image_read",
        description: "Read 1-4 image files via vision and return a text description. Use paths array (e.g. rendered PDF pages under .cache/pdf/ or chat attachments under .cache/attachments/).",
        parameters: json!({
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": 4,
                    "description": "Project-relative image paths (1-4)"
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional instruction for how to interpret the image(s)"
                }
            },
            "required": ["paths"],
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

    let paths = parse_paths_arg(&args)?;
    for path in &paths {
        if !crate::agent::provider::openai_compat::is_image_path(path) {
            return Err(ToolError::Execution(
                "image_read only supports image files (.png, .jpg, .jpeg, .webp, .gif); use pdf_read for PDFs"
                    .into(),
            ));
        }
    }

    let prompt = args
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_PROMPT)
        .to_string();

    let text = vision_subcall(ctx, model_id, &paths, &prompt).await?;

    Ok(json!({
        "text": text,
        "paths": paths,
        "count": paths.len(),
        "mime": paths.first().map(|p| mime_for_path(p)),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use tempfile::tempdir;

    #[tokio::test]
    async fn rejects_non_image_path() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(
            &ctx,
            json!({ "paths": ["notes.txt"] }),
            ModelId::KimiK26,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("image_read only supports"));
    }

    #[tokio::test]
    async fn rejects_missing_paths() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(&ctx, json!({}), ModelId::KimiK26)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("paths required"));
    }

    #[tokio::test]
    async fn rejects_too_many_paths() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(
            &ctx,
            json!({ "paths": ["a.png","b.png","c.png","d.png","e.png"] }),
            ModelId::KimiK26,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("paths must contain"));
    }

    #[test]
    fn tool_is_registered_name() {
        assert_eq!(tool().name, "image_read");
    }
}
