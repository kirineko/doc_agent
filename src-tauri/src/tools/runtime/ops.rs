use crate::core::project_files::list_project_dir;
use crate::core::sandbox::{Sandbox, SandboxError};
use base64::{engine::general_purpose::STANDARD, Engine};
use boa_engine::js_string;
use boa_engine::native_function::NativeFunction;
use boa_engine::{Context, JsResult, JsValue};
use serde_json::json;
use std::cell::RefCell;
use std::path::PathBuf;

const MAX_LOG_CHARS: usize = 2_000;

thread_local! {
    // execute_script 每次 spawn 独立线程，写入记录天然按脚本隔离。
    static WRITTEN_PATHS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub fn register(context: &mut Context, sandbox: &Sandbox) -> JsResult<()> {
    WRITTEN_PATHS.with(|cell| cell.borrow_mut().clear());
    let root = sandbox.root().to_path_buf();
    register_read(context, root.clone())?;
    register_write(context, root.clone())?;
    register_exists(context, root.clone())?;
    register_list(context, root)?;
    register_log(context)?;
    Ok(())
}

/// 取出当前线程脚本执行期间经 `__doc_write` 写入的相对路径。
pub fn take_written_paths() -> Vec<String> {
    WRITTEN_PATHS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
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
                WRITTEN_PATHS.with(|cell| {
                    cell.borrow_mut().push(path.replace('\\', "/"));
                });
                Ok(JsValue::undefined())
            },
            root,
        ),
    )
}

fn register_exists(context: &mut Context, root: PathBuf) -> JsResult<()> {
    context.register_global_builtin_callable(
        js_string!("__doc_exists"),
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
                match sb.resolve_for_write(&path) {
                    Ok(candidate) => Ok(JsValue::from(candidate.exists())),
                    Err(SandboxError::EscapesSandbox | SandboxError::NotFound) => {
                        Ok(JsValue::from(false))
                    }
                    Err(e) => Err(boa_engine::error::JsNativeError::typ()
                        .with_message(e.to_string())
                        .into()),
                }
            },
            root,
        ),
    )
}

fn register_list(context: &mut Context, root: PathBuf) -> JsResult<()> {
    context.register_global_builtin_callable(
        js_string!("__doc_list"),
        1,
        NativeFunction::from_copy_closure_with_captures(
            |_this, args, root, ctx| {
                let rel = args
                    .first()
                    .map(|v| v.to_string(ctx))
                    .transpose()?
                    .map(|s| s.to_std_string_escaped())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| ".".to_string());
                let listing = list_project_dir(root, &rel).map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?;
                let entries: Vec<_> = listing
                    .entries
                    .iter()
                    .map(|e| json!({ "name": e.name, "is_dir": e.is_dir }))
                    .collect();
                Ok(JsValue::from(js_string!(serde_json::to_string(&entries)
                    .map_err(|e| {
                    boa_engine::error::JsNativeError::typ().with_message(e.to_string())
                })?)))
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
