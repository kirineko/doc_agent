use crate::core::sandbox::Sandbox;
use base64::{engine::general_purpose::STANDARD, Engine};
use boa_engine::error::JsNativeError;
use boa_engine::js_string;
use boa_engine::native_function::NativeFunction;
use boa_engine::object::builtins::{JsArrayBuffer, JsUint8Array};
use boa_engine::{Context, JsResult, JsString, JsValue};
use std::path::PathBuf;

pub fn register(context: &mut Context, root: PathBuf) -> JsResult<()> {
    register_read_bytes(context, root)?;
    context.register_global_builtin_callable(
        js_string!("__doc_b64_decode"),
        1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            super::ops::check_cancelled(ctx)?;
            let s = arg_string(args, 0, ctx)?;
            let bytes = decode_b64(&s)?;
            Ok(JsUint8Array::from_iter(bytes, ctx)?.into())
        }),
    )?;
    context.register_global_builtin_callable(
        js_string!("__doc_b64_encode"),
        1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            super::ops::check_cancelled(ctx)?;
            let bytes = arg_bytes(args, 0, ctx)?;
            Ok(JsValue::from(js_string!(STANDARD.encode(bytes))))
        }),
    )?;
    context.register_global_builtin_callable(
        js_string!("__doc_utf8_encode"),
        1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            super::ops::check_cancelled(ctx)?;
            let s = arg_js_string(args, 0, ctx)?;
            let bytes = s.to_std_string_lossy().into_bytes();
            Ok(JsUint8Array::from_iter(bytes, ctx)?.into())
        }),
    )?;
    context.register_global_builtin_callable(
        js_string!("__doc_utf8_decode"),
        1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            super::ops::check_cancelled(ctx)?;
            let bytes = arg_bytes(args, 0, ctx)?;
            let text = String::from_utf8_lossy(&bytes);
            Ok(JsValue::from(js_string!(text.as_ref())))
        }),
    )?;
    context.register_global_builtin_callable(
        js_string!("__doc_latin1_encode"),
        1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            super::ops::check_cancelled(ctx)?;
            let s = arg_js_string(args, 0, ctx)?;
            let mut bytes = Vec::with_capacity(s.len());
            for unit in s.iter() {
                if unit > 0xFF {
                    return Err(JsNativeError::typ()
                        .with_message("InvalidCharacterError")
                        .into());
                }
                bytes.push(unit as u8);
            }
            Ok(JsUint8Array::from_iter(bytes, ctx)?.into())
        }),
    )?;
    context.register_global_builtin_callable(
        js_string!("__doc_latin1_decode"),
        1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            super::ops::check_cancelled(ctx)?;
            let bytes = arg_bytes(args, 0, ctx)?;
            let text: String = bytes.iter().map(|&b| char::from(b)).collect();
            Ok(JsValue::from(js_string!(text.as_str())))
        }),
    )?;
    Ok(())
}

/// 从 Uint8Array 参数取字节切片副本。
pub(crate) fn arg_bytes(args: &[JsValue], idx: usize, ctx: &mut Context) -> JsResult<Vec<u8>> {
    let obj = args
        .get(idx)
        .and_then(JsValue::as_object)
        .ok_or_else(|| JsNativeError::typ().with_message("Uint8Array required"))?;
    let view = JsUint8Array::from_object(obj)?;
    let offset = view.byte_offset(ctx)?;
    let len = view.byte_length(ctx)?;
    let buf_obj = view
        .buffer(ctx)?
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("detached buffer"))?;
    let buf = JsArrayBuffer::from_object(buf_obj)?;
    let data = buf
        .data()
        .ok_or_else(|| JsNativeError::typ().with_message("detached buffer"))?;
    Ok(data.get(offset..offset + len).unwrap_or(&[]).to_vec())
}

fn arg_js_string(args: &[JsValue], idx: usize, ctx: &mut Context) -> JsResult<JsString> {
    let v = args
        .get(idx)
        .ok_or_else(|| JsNativeError::typ().with_message("string required"))?;
    v.to_string(ctx)
}

fn arg_string(args: &[JsValue], idx: usize, ctx: &mut Context) -> JsResult<String> {
    Ok(arg_js_string(args, idx, ctx)?.to_std_string_escaped())
}

fn decode_b64(input: &str) -> JsResult<Vec<u8>> {
    let trimmed = input.trim();
    let stripped = trimmed.trim_end_matches('=');
    if stripped.is_empty() {
        return Ok(Vec::new());
    }
    let pad = (4 - stripped.len() % 4) % 4;
    let mut padded = String::with_capacity(stripped.len() + pad);
    padded.push_str(stripped);
    for _ in 0..pad {
        padded.push('=');
    }
    STANDARD.decode(padded).map_err(|e| {
        JsNativeError::typ()
            .with_message(format!("invalid base64: {e}"))
            .into()
    })
}

fn register_read_bytes(context: &mut Context, root: PathBuf) -> JsResult<()> {
    context.register_global_builtin_callable(
        js_string!("__doc_read_bytes"),
        1,
        NativeFunction::from_copy_closure_with_captures(
            |_this, args, root, ctx| {
                super::ops::check_cancelled(ctx)?;
                let path = arg_string(args, 0, ctx)?;
                let sb = Sandbox::new(root)
                    .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
                let resolved = sb
                    .resolve(&path)
                    .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
                let bytes = std::fs::read(resolved)
                    .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
                Ok(JsUint8Array::from_iter(bytes, ctx)?.into())
            },
            root,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    fn eval(code: &str) -> Result<JsValue, String> {
        let mut context = Context::default();
        register(&mut context, PathBuf::from(".")).map_err(|e| e.to_string())?;
        context
            .eval(Source::from_bytes(code))
            .map_err(|e| e.to_string())
    }

    fn eval_json(code: &str) -> serde_json::Value {
        let mut context = Context::default();
        register(&mut context, PathBuf::from(".")).unwrap();
        let v = context
            .eval(Source::from_bytes(code))
            .unwrap_or_else(|e| panic!("{e}"));
        let json = v
            .to_json(&mut context)
            .unwrap_or_else(|e| panic!("{e}"))
            .expect("json");
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn latin1_and_b64_roundtrip_all_bytes() {
        let out = eval_json(
            r#"
(() => {
  const bytes = new Uint8Array(256);
  for (let i = 0; i < 256; i++) bytes[i] = i;
  const latin = __doc_latin1_decode(bytes);
  const back = __doc_latin1_encode(latin);
  const b64 = __doc_b64_encode(bytes);
  const decoded = __doc_b64_decode(b64);
  let match = back.length === 256 && decoded.length === 256;
  for (let i = 0; i < 256; i++) {
    if (back[i] !== i || decoded[i] !== i) match = false;
  }
  return { match, b64len: b64.length, empty: __doc_b64_encode(new Uint8Array(0)) };
})()
"#,
        );
        assert_eq!(out["match"], true);
        assert_eq!(out["empty"], "");
    }

    #[test]
    fn utf8_roundtrip() {
        let out = eval_json(
            r#"
(() => {
  const s = "中文 emoji 😀";
  const encoded = __doc_utf8_encode(s);
  return { back: __doc_utf8_decode(encoded), len: encoded.length };
})()
"#,
        );
        assert_eq!(out["back"], "中文 emoji 😀");
        assert!(out["len"].as_u64().unwrap() > 8);
    }

    #[test]
    fn invalid_base64_throws() {
        let err = eval(r#"__doc_b64_decode("!!!")"#).unwrap_err();
        assert!(err.contains("invalid base64"), "err={err}");
    }

    #[test]
    fn latin1_encode_rejects_high_code_unit() {
        let err = eval(r#"__doc_latin1_encode("\u0100")"#).unwrap_err();
        assert!(err.contains("InvalidCharacterError"), "err={err}");
    }

    #[test]
    fn empty_inputs() {
        let out = eval_json(
            r#"
(() => {
  const empty = new Uint8Array(0);
  return {
    b64: __doc_b64_encode(empty),
    decodedLen: __doc_b64_decode("").length,
    latin: __doc_latin1_decode(empty),
    utf: __doc_utf8_decode(empty)
  };
})()
"#,
        );
        assert_eq!(out["b64"], "");
        assert_eq!(out["decodedLen"], 0);
        assert_eq!(out["latin"], "");
        assert_eq!(out["utf"], "");
    }
}
