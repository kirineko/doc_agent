use crate::core::sandbox::Sandbox;
use boa_engine::error::JsNativeError;
use boa_engine::js_string;
use boa_engine::native_function::NativeFunction;
use boa_engine::{Context, JsResult, JsValue};
use image::imageops::FilterType;
use image::{ImageFormat, ImageReader};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};

const MAX_PIXELS: u64 = 50_000_000;
const DEFAULT_MAX_EDGE: u32 = 1600;
const DEFAULT_QUALITY: u8 = 82;

#[derive(Debug, Serialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub mime: &'static str,
    pub bytes: u64,
}

pub struct ResizeOpts {
    pub max_edge: u32,
    pub quality: u8,
    pub out: Option<String>,
}

pub fn image_info(path: &Path) -> Result<ImageInfo, String> {
    let bytes = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    let reader = ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mime = match reader.format() {
        Some(ImageFormat::Png) => "image/png",
        Some(ImageFormat::Jpeg) => "image/jpeg",
        Some(ImageFormat::Gif) => "image/gif",
        Some(ImageFormat::WebP) => "image/webp",
        other => return Err(format!("unsupported image format: {other:?}")),
    };
    let (width, height) = reader.into_dimensions().map_err(|e| e.to_string())?;
    Ok(ImageInfo {
        width,
        height,
        mime,
        bytes,
    })
}

/// 返回 (相对输出路径, 新宽, 新高, 输出字节数)。长边已 ≤ max_edge 且格式为 JPEG/PNG 时直接返回原路径。
pub fn image_resize(
    sandbox: &Sandbox,
    rel: &str,
    opts: ResizeOpts,
) -> Result<(String, u32, u32, u64), String> {
    let src = sandbox.resolve(rel).map_err(|e| e.to_string())?;
    let info = image_info(&src)?;
    let needs_resize = info.width.max(info.height) > opts.max_edge;
    let needs_reencode = !matches!(info.mime, "image/jpeg" | "image/png");
    if !needs_resize && !needs_reencode && opts.out.is_none() {
        return Ok((rel.to_string(), info.width, info.height, info.bytes));
    }
    if info.width as u64 * info.height as u64 > MAX_PIXELS {
        return Err(format!(
            "image too large: {}x{} exceeds 50 megapixels",
            info.width, info.height
        ));
    }
    let img = image::open(&src).map_err(|e| e.to_string())?;
    let img = if needs_resize {
        img.resize(opts.max_edge, opts.max_edge, FilterType::Triangle)
    } else {
        img
    };
    let keep_png = info.mime == "image/png" && img.color().has_alpha();
    let out_rel = opts.out.unwrap_or_else(|| {
        let ext = if keep_png { "png" } else { "jpg" };
        let mut hasher = DefaultHasher::new();
        (rel, opts.max_edge, opts.quality).hash(&mut hasher);
        let hash = hasher.finish();
        format!(
            ".cache/images/{}-{}-{hash:016x}.{ext}",
            stem(rel),
            opts.max_edge
        )
    });
    ensure_relative_out(&out_rel)?;
    super::ops::apply_write_gate(&out_rel)?;
    let out_abs = sandbox
        .resolve_for_write(&out_rel)
        .map_err(|e| e.to_string())?;
    if let Some(parent) = out_abs.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut buf = Cursor::new(Vec::new());
    if keep_png {
        img.write_to(&mut buf, ImageFormat::Png)
            .map_err(|e| e.to_string())?;
    } else {
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, opts.quality);
        img.to_rgb8()
            .write_with_encoder(enc)
            .map_err(|e| e.to_string())?;
    }
    std::fs::write(&out_abs, buf.get_ref()).map_err(|e| e.to_string())?;
    Ok((
        out_rel,
        img.width(),
        img.height(),
        buf.get_ref().len() as u64,
    ))
}

pub fn register(context: &mut Context, root: PathBuf) -> JsResult<()> {
    context.register_global_builtin_callable(
        js_string!("__doc_image_info"),
        1,
        NativeFunction::from_copy_closure_with_captures(
            |_this, args, root, ctx| {
                super::ops::check_cancelled(ctx)?;
                let rel = arg_string(args, 0, ctx)?;
                let sb = Sandbox::new(root).map_err(js_err)?;
                let src = sb.resolve(&rel).map_err(js_err)?;
                let info = image_info(&src).map_err(js_err)?;
                let payload = serde_json::to_string(&info).map_err(js_err)?;
                Ok(JsValue::from(js_string!(payload)))
            },
            root.clone(),
        ),
    )?;
    context.register_global_builtin_callable(
        js_string!("__doc_image_resize"),
        2,
        NativeFunction::from_copy_closure_with_captures(
            |_this, args, root, ctx| {
                super::ops::check_cancelled(ctx)?;
                let rel = arg_string(args, 0, ctx)?;
                let opts_json = args
                    .get(1)
                    .map(|v| v.to_string(ctx))
                    .transpose()?
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_else(|| "{}".into());
                let opts = parse_opts(&opts_json);
                let sb = Sandbox::new(root).map_err(js_err)?;
                let (path, width, height, bytes) = image_resize(&sb, &rel, opts).map_err(js_err)?;
                if path != rel {
                    super::ops::record_written_path(&path);
                }
                Ok(JsValue::from(js_string!(serde_json::to_string(&json!({
                    "path": path,
                    "width": width,
                    "height": height,
                    "bytes": bytes
                }))
                .map_err(js_err)?)))
            },
            root,
        ),
    )?;
    Ok(())
}

fn parse_opts(raw: &str) -> ResizeOpts {
    let v: Value = serde_json::from_str(raw).unwrap_or_else(|_| json!({}));
    let max_edge = v
        .get("maxEdge")
        .and_then(Value::as_u64)
        .unwrap_or(DEFAULT_MAX_EDGE as u64)
        .clamp(1, 10_000) as u32;
    let quality = v
        .get("quality")
        .and_then(Value::as_u64)
        .unwrap_or(DEFAULT_QUALITY as u64)
        .clamp(1, 100) as u8;
    let out = v
        .get("out")
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|s| !s.is_empty());
    ResizeOpts {
        max_edge,
        quality,
        out,
    }
}

fn ensure_relative_out(rel: &str) -> Result<(), String> {
    let path = Path::new(rel);
    if path.is_absolute()
        || path.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err("path escapes project sandbox".into());
    }
    Ok(())
}

fn stem(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .replace(['\\', '/'], "_")
}

fn arg_string(args: &[JsValue], idx: usize, ctx: &mut Context) -> JsResult<String> {
    let v = args
        .get(idx)
        .ok_or_else(|| JsNativeError::typ().with_message("path required"))?;
    Ok(v.to_string(ctx)?.to_std_string_escaped())
}

fn js_err(e: impl ToString) -> boa_engine::JsError {
    JsNativeError::typ().with_message(e.to_string()).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use image::{Rgb, RgbImage, Rgba, RgbaImage};

    fn sandbox_dir() -> (tempfile::TempDir, Sandbox) {
        let dir = tempfile::tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        (dir, sandbox)
    }

    fn write_jpeg(path: &Path, w: u32, h: u32) {
        RgbImage::from_pixel(w, h, Rgb([180, 40, 90]))
            .save(path)
            .unwrap();
    }

    fn write_png_alpha(path: &Path, w: u32, h: u32) {
        RgbaImage::from_pixel(w, h, Rgba([10, 20, 30, 128]))
            .save(path)
            .unwrap();
    }

    #[test]
    fn image_info_reads_jpeg_header() {
        let (dir, _) = sandbox_dir();
        let path = dir.path().join("photo.jpg");
        write_jpeg(&path, 4000, 3000);
        let started = std::time::Instant::now();
        let info = image_info(&path).unwrap();
        assert!(started.elapsed().as_millis() < 100);
        assert_eq!(info.width, 4000);
        assert_eq!(info.height, 3000);
        assert_eq!(info.mime, "image/jpeg");
        assert!(info.bytes > 0);
    }

    #[test]
    fn image_resize_scales_large_jpeg() {
        let (dir, sandbox) = sandbox_dir();
        write_jpeg(&dir.path().join("photo.jpg"), 4000, 3000);
        let (out, w, h, bytes) = image_resize(
            &sandbox,
            "photo.jpg",
            ResizeOpts {
                max_edge: 1600,
                quality: 82,
                out: None,
            },
        )
        .unwrap();
        assert!(out.starts_with(".cache/images/"), "{out}");
        assert_eq!(w, 1600);
        assert_eq!(h, 1200);
        assert!(bytes > 0);
        assert!(dir.path().join(&out).exists());
    }

    #[test]
    fn default_resize_paths_distinguish_sources_and_quality() {
        let (dir, sandbox) = sandbox_dir();
        for folder in ["a", "b"] {
            std::fs::create_dir(dir.path().join(folder)).unwrap();
        }
        write_jpeg(&dir.path().join("a/photo.jpg"), 64, 48);
        write_jpeg(&dir.path().join("b/photo.jpg"), 48, 64);
        RgbImage::from_pixel(64, 48, Rgb([10, 200, 30]))
            .save(dir.path().join("a/photo.png"))
            .unwrap();
        let resize = |rel, quality| {
            image_resize(
                &sandbox,
                rel,
                ResizeOpts {
                    max_edge: 32,
                    quality,
                    out: None,
                },
            )
            .unwrap()
            .0
        };
        let first = resize("a/photo.jpg", 82);
        let original = std::fs::read(dir.path().join(&first)).unwrap();
        let other_dir = resize("b/photo.jpg", 82);
        let other_ext = resize("a/photo.png", 82);
        let other_quality = resize("a/photo.jpg", 60);
        let paths = [&first, &other_dir, &other_ext, &other_quality];
        assert_eq!(
            paths
                .into_iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            4
        );
        assert_eq!(std::fs::read(dir.path().join(&first)).unwrap(), original);
        assert_eq!(resize("a/photo.jpg", 82), first);
        assert!(first.starts_with(".cache/images/photo-32-"));
        assert!(first.ends_with(".jpg"));
    }

    #[test]
    fn small_png_passthrough() {
        let (dir, sandbox) = sandbox_dir();
        let path = dir.path().join("icon.png");
        write_png_alpha(&path, 800, 600);
        let before = std::fs::read(&path).unwrap();
        let (out, w, h, _) = image_resize(
            &sandbox,
            "icon.png",
            ResizeOpts {
                max_edge: 1600,
                quality: 82,
                out: None,
            },
        )
        .unwrap();
        assert_eq!(out, "icon.png");
        assert_eq!((w, h), (800, 600));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn non_image_is_rejected() {
        let (dir, _) = sandbox_dir();
        std::fs::write(dir.path().join("doc.docx"), b"PK\x03\x04 not an image").unwrap();
        let err = image_info(&dir.path().join("doc.docx")).unwrap_err();
        assert!(err.contains("unsupported image format"), "err={err}");
    }

    #[test]
    fn resize_out_escape_is_rejected() {
        let (dir, sandbox) = sandbox_dir();
        write_jpeg(&dir.path().join("photo.jpg"), 64, 64);
        let escaped = dir.path().join("..").join("doc-agent-resize-escape.jpg");
        let _ = std::fs::remove_file(&escaped);
        let err = image_resize(
            &sandbox,
            "photo.jpg",
            ResizeOpts {
                max_edge: 32,
                quality: 82,
                out: Some("../doc-agent-resize-escape.jpg".into()),
            },
        )
        .unwrap_err();
        assert!(
            err.to_lowercase().contains("sandbox") || err.contains("escapes"),
            "err={err}"
        );
        assert!(!escaped.exists(), "escaped file was created at {escaped:?}");
    }

    #[test]
    fn doc_image_resize_reread_and_written_paths() {
        let (dir, sandbox) = sandbox_dir();
        write_jpeg(&dir.path().join("photo.jpg"), 4000, 3000);
        let (value, written) = crate::tools::runtime::execute_script(
            &sandbox,
            r#"
function main() {
  const r = doc_image_resize("photo.jpg", { maxEdge: 1600 });
  const info = doc_image_info(r.path);
  return { path: r.path, width: r.width, height: r.height, infoW: info.width, infoH: info.height };
}
"#,
            std::time::Duration::from_secs(30),
            None,
            None,
        )
        .unwrap_or_else(|e| panic!("{}", e.to_json_value()));
        assert_eq!(value["width"], 1600);
        assert_eq!(value["height"], 1200);
        assert_eq!(value["infoW"], 1600);
        assert_eq!(value["infoH"], 1200);
        let path = value["path"].as_str().unwrap();
        assert!(
            written.iter().any(|p| p == path),
            "written_paths={written:?} path={path}"
        );
    }
}
