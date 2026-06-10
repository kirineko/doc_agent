use crate::core::sandbox::Sandbox;
use base64::{engine::general_purpose::STANDARD, Engine};
use boa_engine::js_string;
use boa_engine::native_function::NativeFunction;
use boa_engine::{Context, JsResult, JsValue};
use std::path::PathBuf;

const MAX_LOG_CHARS: usize = 2_000;

pub fn register(context: &mut Context, sandbox: &Sandbox) -> JsResult<()> {
    let root = sandbox.root().to_path_buf();
    register_read(context, root.clone())?;
    register_write(context, root)?;
    register_log(context)?;
    Ok(())
}

fn register_read(context: &mut Context, root: PathBuf) -> JsResult<()> {
    context.register_global_builtin_callable(
        js_string!("__doc_read"),
        1,
        NativeFunction::from_copy_closure_with_captures(
            |_this, args, root, ctx| {
                let path = args
                    .first()
                    .ok_or_else(|| {
                        boa_engine::error::JsNativeError::typ().with_message("path required")
                    })?
                    .to_string(ctx)?
                    .to_std_string_escaped();
                let sb = Sandbox::new(root).map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?;
                let resolved = sb.resolve(&path).map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?;
                let bytes = std::fs::read(resolved).map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?;
                Ok(JsValue::from(js_string!(STANDARD.encode(bytes))))
            },
            root,
        ),
    )
}

fn register_write(context: &mut Context, root: PathBuf) -> JsResult<()> {
    context.register_global_builtin_callable(
        js_string!("__doc_write"),
        2,
        NativeFunction::from_copy_closure_with_captures(
            |_this, args, root, ctx| {
                let path = args
                    .first()
                    .ok_or_else(|| {
                        boa_engine::error::JsNativeError::typ().with_message("path required")
                    })?
                    .to_string(ctx)?
                    .to_std_string_escaped();
                let data_b64 = args
                    .get(1)
                    .ok_or_else(|| {
                        boa_engine::error::JsNativeError::typ().with_message("data required")
                    })?
                    .to_string(ctx)?
                    .to_std_string_escaped();
                let bytes = STANDARD.decode(data_b64).map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?;
                let sb = Sandbox::new(root).map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?;
                let resolved = sb.resolve_for_write(&path).map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?;
                if let Some(parent) = resolved.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                    })?;
                }
                std::fs::write(resolved, bytes).map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?;
                Ok(JsValue::undefined())
            },
            root,
        ),
    )
}

fn register_log(context: &mut Context) -> JsResult<()> {
    context.register_global_builtin_callable(
        js_string!("__doc_log"),
        1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            if let Some(v) = args.first() {
                if let Ok(s) = v.to_string(ctx) {
                    let mut text = s.to_std_string_escaped();
                    if text.len() > MAX_LOG_CHARS {
                        text.truncate(MAX_LOG_CHARS);
                        text.push_str("...");
                    }
                    eprintln!("[skill_run] {text}");
                }
            }
            Ok(JsValue::undefined())
        }),
    )
}
