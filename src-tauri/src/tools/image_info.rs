use crate::tools::runtime::image_info as read_image_info;
use crate::tools::{ToolContext, ToolError};
use serde_json::{json, Value};

const MAX_PATHS: usize = 20;

pub fn tool() -> crate::tools::ToolSpec {
    crate::tools::ToolSpec {
        name: "image_info",
        description: "Read image metadata (width, height, mime, bytes) from local project files \
            by inspecting file headers only. Use before inserting images into documents. \
            Supports PNG, JPEG, GIF, and WebP. paths is a project-relative array.",
        parameters: json!({
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": MAX_PATHS,
                    "description": "Project-relative image paths (1-20)"
                }
            },
            "required": ["paths"],
            "additionalProperties": false
        }),
        handler,
    }
}

fn handler(ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
    let paths = parse_paths(&args)?;
    let mut images = Vec::with_capacity(paths.len());
    for rel in &paths {
        let resolved = ctx.sandbox.resolve(rel).map_err(ToolError::Sandbox)?;
        let info = read_image_info(&resolved).map_err(ToolError::Execution)?;
        images.push(json!({
            "path": rel,
            "width": info.width,
            "height": info.height,
            "mime": info.mime,
            "bytes": info.bytes,
        }));
    }
    Ok(json!({ "images": images }))
}

fn parse_paths(args: &Value) -> Result<Vec<String>, ToolError> {
    let Some(arr) = args.get("paths").and_then(|v| v.as_array()) else {
        return Err(ToolError::InvalidArgs("paths required".into()));
    };
    if arr.is_empty() || arr.len() > MAX_PATHS {
        return Err(ToolError::InvalidArgs(format!(
            "paths must contain 1..={MAX_PATHS} items"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use image::{Rgb, RgbImage};
    use tempfile::tempdir;

    fn write_jpeg(path: &std::path::Path, w: u32, h: u32) {
        RgbImage::from_pixel(w, h, Rgb([20, 80, 160]))
            .save(path)
            .unwrap();
    }

    #[test]
    fn reads_jpeg_metadata() {
        let dir = tempdir().unwrap();
        write_jpeg(&dir.path().join("photo.jpg"), 320, 240);
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let out = handler(&ctx, json!({ "paths": ["photo.jpg"] })).unwrap();
        assert_eq!(out["images"][0]["path"], "photo.jpg");
        assert_eq!(out["images"][0]["width"], 320);
        assert_eq!(out["images"][0]["height"], 240);
        assert_eq!(out["images"][0]["mime"], "image/jpeg");
        assert!(out["images"][0]["bytes"].as_u64().unwrap() > 0);
    }

    #[test]
    fn rejects_non_image() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("notes.txt"), b"hello").unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(&ctx, json!({ "paths": ["notes.txt"] })).unwrap_err();
        assert!(
            err.to_string().contains("unsupported image format"),
            "{err}"
        );
    }

    #[test]
    fn rejects_missing_paths() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let ctx = ToolContext::new(&sandbox);
        let err = handler(&ctx, json!({})).unwrap_err();
        assert!(err.to_string().contains("paths required"));
    }

    #[test]
    fn tool_is_registered_name() {
        assert_eq!(tool().name, "image_info");
    }
}
