mod diagnostics;
mod normalize;
mod ops;
mod ops_binary;
mod ops_image;
pub mod write_gate;

pub(crate) use ops_image::image_info;
pub use write_gate::RuntimeWriteGate;

use super::ToolError;
use crate::core::sandbox::Sandbox;
use boa_engine::builtins::promise::PromiseState;
use boa_engine::object::builtins::JsPromise;
use boa_engine::{Context, JsValue, Source};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

const HELPERS: &str = r#"
const doc_read = (path) => __doc_read(path);
const doc_write = (path, data) => __doc_write(path, data);
const doc_log = (...args) => __doc_log(JSON.stringify(args));
const doc_exists = (path) => __doc_exists(path);
const doc_list = (path) => JSON.parse(__doc_list(path == null || path === "" ? "." : path));
const doc_image_info = (p) => JSON.parse(__doc_image_info(p));
const doc_image_resize = (p, o) => {
  const opts = Object.assign({ maxEdge: 1600, quality: 82 }, o == null ? {} : o);
  return JSON.parse(__doc_image_resize(p, JSON.stringify(opts)));
};
// 浏览器/Node 全局的最小 polyfill（boa 不内置 WebAPI）
var setTimeout = (fn, _ms, ...a) => { Promise.resolve().then(() => fn(...a)); return 0; };
var setImmediate = (fn, ...a) => { Promise.resolve().then(() => fn(...a)); return 0; };
var clearTimeout = () => {};
var clearImmediate = () => {};
var queueMicrotask = (fn) => { Promise.resolve().then(fn); };
var console = { log: doc_log, warn: doc_log, error: doc_log, info: doc_log, debug: () => {} };
var process = { nextTick: (fn, ...a) => queueMicrotask(() => fn(...a)), env: {}, browser: true };
var crypto = { getRandomValues: (arr) => { for (let i = 0; i < arr.length; i++) arr[i] = Math.floor(Math.random() * 256); return arr; } };
var self = globalThis;
var __toU8 = (d) => d instanceof Uint8Array ? d
  : d instanceof ArrayBuffer ? new Uint8Array(d)
  : ArrayBuffer.isView(d) ? new Uint8Array(d.buffer, d.byteOffset, d.byteLength)
  : new Uint8Array(d);
var __bytesToB64 = (data) => __doc_b64_encode(__toU8(data));
var __b64ToBytes = (b64) => __doc_b64_decode(b64);
var atob = (b64) => __doc_latin1_decode(__doc_b64_decode(String(b64)));
var btoa = (bin) => __doc_b64_encode(__doc_latin1_encode(String(bin)));
class TextEncoder { encode(s) { return __doc_utf8_encode(String(s)); } }
class TextDecoder { decode(buf) { return __doc_utf8_decode(__toU8(buf)); } }
var __wrapBuffer = (bytes) => Object.assign(bytes, {
  toString(enc) { return enc === "base64" ? __doc_b64_encode(this) : __doc_utf8_decode(this); }
});
var doc_write_bytes = (path, data) => __doc_write(path, __bytesToB64(data));
var Buffer = {
  from: (data, encoding) => {
    const bytes = (typeof data === "string" && encoding === "base64") ? __doc_b64_decode(data)
      : typeof data === "string" ? __doc_utf8_encode(data)
      : __toU8(data);
    return __wrapBuffer(bytes);
  },
  alloc: (n) => __wrapBuffer(new Uint8Array(n)),
  isBuffer: (x) => x instanceof Uint8Array,
};
var path = {
  join: (...parts) => parts.filter((p) => p != null && p !== "").join("/").replace(/\/+/g, "/"),
  dirname: (p) => { const i = String(p).lastIndexOf("/"); return i < 0 ? "." : String(p).slice(0, i); },
  basename: (p) => { const s = String(p); const i = s.lastIndexOf("/"); return i < 0 ? s : s.slice(i + 1); },
};
var fs = {
  writeFileSync: (filePath, data, encoding) => {
    if (typeof data === "string") {
      const enc = encoding && String(encoding).toLowerCase();
      if (enc && enc !== "utf-8" && enc !== "utf8") {
        throw new Error("fs.writeFileSync: only utf-8 text encoding is supported");
      }
      doc_write_bytes(filePath, new TextEncoder().encode(data));
    } else {
      doc_write_bytes(filePath, data);
    }
  },
  readFileSync: (filePath, encoding) => {
    const enc = encoding && String(encoding).toLowerCase();
    if (enc === "base64") return doc_read(filePath);
    const bytes = __doc_read_bytes(filePath);
    if (enc === "utf-8" || enc === "utf8") return __doc_utf8_decode(bytes);
    return __wrapBuffer(bytes);
  },
  existsSync: (filePath) => doc_exists(filePath),
  readdirSync: (filePath) => doc_list(filePath || ".").map((e) => e.name),
};
var require = (id) => {
  const key = String(id).toLowerCase();
  if (key === "fs") return fs;
  if (key === "path") return path;
  if (key === "exceljs" && typeof ExcelJS !== "undefined") return ExcelJS;
  if (key === "pptxgenjs" && typeof PptxGenJS !== "undefined") return PptxGenJS.default ?? PptxGenJS;
  if (key === "docx" && typeof docx !== "undefined") return docx;
  if ((key === "pdf-lib" || key === "pdflib") && typeof PDFLib !== "undefined") return PDFLib;
  throw new Error("Cannot find module '" + id + "'. Globals: fs, path, ExcelJS, PptxGenJS, PDFLib, docx");
};
"#;

/// 各 bundle 的 Node API 替身：writeFile 等在 boa 无 fs，统一改走沙箱写入。
const EXCELJS_SHIM: &str = r#"
(() => {
  if (typeof ExcelJS === "undefined") return;
  const wb = new ExcelJS.Workbook();
  for (const form of ["xlsx", "csv"]) {
    const proto = Object.getPrototypeOf(wb[form]);
    if (proto && typeof proto.writeBuffer === "function") {
      proto.writeFile = async function (filename) {
        const buf = await this.writeBuffer();
        __doc_write(String(filename), __bytesToB64(buf));
        return String(filename);
      };
    }
  }
})();
"#;

const PPTXGENJS_SHIM: &str = r#"
(() => {
  if (typeof PptxGenJS === "undefined") return;
  const Raw = PptxGenJS;
  const Ctor = Raw.default ?? Raw;
  if (!Ctor || typeof Ctor !== "function" || !Ctor.prototype) return;
  // bundle 导出的是模块对象；模型常写 new PptxGenJS()，将全局规范为可 new 的构造函数
  if (Raw !== Ctor) {
    for (const k in Raw) {
      if (k !== "default" && Raw[k] != null && Ctor[k] == null) Ctor[k] = Raw[k];
    }
    if (Raw.default && typeof Raw.default === "object") {
      for (const k in Raw.default) {
        if (Raw.default[k] != null && Ctor[k] == null) Ctor[k] = Raw.default[k];
      }
    }
  }
  globalThis.PptxGenJS = Ctor;
  const orig = Ctor.prototype.addSlide;
  if (typeof orig === "function") {
    Ctor.prototype.addSlide = function (...args) {
      const slide = orig.apply(this, args);
      if (Ctor.ShapeType && !this.ShapeType) this.ShapeType = Ctor.ShapeType;
      if (Ctor.ChartType && !this.ChartType) this.ChartType = Ctor.ChartType;
      return slide;
    };
  }
  Ctor.prototype.writeFile = async function (opts) {
    const name = typeof opts === "string"
      ? opts
      : (opts && opts.fileName) || "output.pptx";
    const b64 = await this.write({ outputType: "base64" });
    __doc_write(String(name), b64);
    return String(name);
  };
})();
"#;

const DOCX_SHIM: &str = r#"
(() => {
  if (typeof docx === "undefined" || !docx.Packer) return;
  if (typeof docx.Packer.toBase64String === "function") {
    docx.Packer.toBuffer = async (d) => __b64ToBytes(await docx.Packer.toBase64String(d));
  }
})();
"#;

/// 执行脚本，返回脚本结果与执行期间经 `__doc_write` 写入的相对路径。
pub fn execute_script(
    sandbox: &Sandbox,
    code: &str,
    timeout: Duration,
    script_path: Option<&str>,
    write_gate: Option<std::sync::Arc<write_gate::RuntimeWriteGate>>,
) -> Result<(Value, Vec<String>), ToolError> {
    let root = sandbox.root().to_path_buf();
    let code = code.to_string();
    let code_for_error = code.clone();
    let script_path = script_path.map(str::to_string);
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_for_thread = cancel.clone();

    // boa 解析/执行大型 bundle（如 exceljs ~1MB）递归较深，需要加大栈。
    std::thread::Builder::new()
        .name("skill_run".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ops::set_runtime_write_gate(write_gate);
                ops::set_cancel_flag(Some(cancel_for_thread));
                let result = run_in_thread(&root, &code, timeout);
                ops::set_cancel_flag(None);
                ops::set_runtime_write_gate(None);
                result
            }))
            .unwrap_or_else(|_| Err("script panicked".into()));
            let _ = tx.send(result);
        })
        .map_err(|e| ToolError::Execution(format!("spawn runtime thread: {e}")))?;

    match rx.recv_timeout(timeout) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(ToolError::Structured(diagnostics::build_script_error(
            &code_for_error,
            &e,
            script_path.as_deref(),
            timeout.as_secs(),
        ))),
        Err(_) => {
            cancel.store(true, Ordering::Relaxed);
            match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(Ok(v)) => Ok(v),
                Ok(Err(e)) => {
                    let detail = if e.to_lowercase().contains("cancelled by host") {
                        "script timeout".to_string()
                    } else {
                        e
                    };
                    Err(ToolError::Structured(diagnostics::build_script_error(
                        &code_for_error,
                        &detail,
                        script_path.as_deref(),
                        timeout.as_secs(),
                    )))
                }
                Err(_) => Err(ToolError::Structured(diagnostics::build_script_error(
                    &code_for_error,
                    "script timeout",
                    script_path.as_deref(),
                    timeout.as_secs(),
                ))),
            }
        }
    }
}

fn run_in_thread(
    root: &PathBuf,
    code: &str,
    timeout: Duration,
) -> Result<(Value, Vec<String>), String> {
    let started = std::time::Instant::now();
    let sandbox = Sandbox::new(root).map_err(|e| e.to_string())?;
    let mut context = Context::default();
    context
        .runtime_limits_mut()
        .set_loop_iteration_limit(20_000_000);
    ops::register(&mut context, &sandbox).map_err(|e| e.to_string())?;

    context
        .eval(Source::from_bytes(HELPERS))
        .map_err(|e| format!("failed to init helpers: {e}"))?;

    let normalized = normalize::normalize_script(code);

    for (name, source, shim) in bundles_for_code(&normalized) {
        if started.elapsed() > timeout {
            return Err("script timeout".into());
        }
        context
            .eval(Source::from_bytes(source))
            .map_err(|e| format!("failed to load bundle {name}: {e}"))?;
        if !shim.is_empty() {
            context
                .eval(Source::from_bytes(shim))
                .map_err(|e| format!("failed to init {name} shim: {e}"))?;
        }
    }

    let script = format!(
        r#"{normalized}
(() => {{
  if (typeof main !== "function") throw new Error("Script must define function main()");
  return main();
}})()
"#
    );
    let result = context
        .eval(Source::from_bytes(&script))
        .map_err(|e| with_runtime_hint(e.to_string()))?;
    let result = settle_promise(&mut context, result)?;
    let value = js_to_json(&mut context, &result)?;
    Ok((value, ops::take_written_paths()))
}

/// 常见 Node/浏览器 API 误用时，附加运行时环境提示，避免模型盲目试探。
fn with_runtime_hint(msg: String) -> String {
    let lower = msg.to_lowercase();
    if lower.contains("cannot find module") {
        return format!(
            "{msg}\n提示：require 白名单：fs、path、exceljs、pptxgenjs、docx、pdf-lib；库也可用全局 ExcelJS/PptxGenJS/docx/PDFLib。\
             先 skill_read {{\"skill\":\"runtime\"}} 查看完整 API。"
        );
    }
    if lower.contains("import ") || lower.contains("unexpected token 'export'") {
        return format!(
            "{msg}\n提示：不支持 ES module import；用全局变量或 require('…')。先 skill_read {{\"skill\":\"runtime\"}}。"
        );
    }
    if lower.contains("fetch") && lower.contains("not defined") {
        return format!(
            "{msg}\n提示：运行时无网络/fetch。先 skill_read {{\"skill\":\"runtime\"}}。"
        );
    }
    let suspicious = msg.contains("not a callable function")
        || msg.contains("not a constructor")
        || msg.contains("is not defined")
        || msg.contains("not an object");
    if suspicious {
        format!(
            "{msg}\n提示：嵌入式 boa 运行时（非 Node）。先 skill_read {{\"skill\":\"runtime\"}}。\
             require('fs'|'path'|'exceljs'|'pptxgenjs'|'docx'|'pdf-lib')；doc_exists/doc_list 可查路径。\
             勿在末尾写 main()。xlsx：await wb.xlsx.writeFile('out.xlsx')；pptx：await pptx.writeFile({{ fileName: 'out.pptx' }})。"
        )
    } else {
        msg
    }
}

/// async main() 返回 Promise 时，跑完微任务队列并取出结果。
fn settle_promise(context: &mut Context, value: JsValue) -> Result<JsValue, String> {
    let Some(obj) = value.as_object() else {
        return Ok(value);
    };
    let Ok(promise) = JsPromise::from_object(obj.clone()) else {
        return Ok(value);
    };
    context
        .run_jobs()
        .map_err(|e| format!("microtask queue failed: {e}"))?;
    match promise.state() {
        PromiseState::Fulfilled(v) => Ok(v),
        PromiseState::Rejected(e) => Err(with_runtime_hint(format!(
            "script rejected: {}",
            e.display()
        ))),
        PromiseState::Pending => {
            Err("script promise never settled (pending after run_jobs)".into())
        }
    }
}

fn bundles_for_code(code: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    let mut out = Vec::new();
    let lower = code.to_lowercase();
    if needs_exceljs(&lower) {
        out.push((
            "exceljs",
            include_str!("../../../assets/js/exceljs.bundle.js"),
            EXCELJS_SHIM,
        ));
    }
    if lower.contains("docx") {
        out.push((
            "docx",
            include_str!("../../../assets/js/docx.bundle.js"),
            DOCX_SHIM,
        ));
    }
    if needs_pptxgenjs(&lower) {
        out.push((
            "pptxgenjs",
            include_str!("../../../assets/js/pptxgenjs.bundle.js"),
            PPTXGENJS_SHIM,
        ));
    }
    if lower.contains("pdflib") || lower.contains("pdf-lib") {
        out.push((
            "pdf-lib",
            include_str!("../../../assets/js/pdf-lib.bundle.js"),
            "",
        ));
    }
    out
}

fn needs_exceljs(lower: &str) -> bool {
    lower.contains("exceljs")
        || lower.contains("exceljs.workbook")
        || (lower.contains("workbook") && lower.contains("addworksheet"))
        || lower.contains("xlsx.writefile")
        || lower.contains("xlsx.writebuffer")
}

/// `PptxGenJS` lowercases to `pptxgenjs`; bare `.pptx` path strings must not trigger load.
fn needs_pptxgenjs(lower: &str) -> bool {
    lower.contains("pptxgenjs")
}

#[cfg(test)]
mod bundle_tests {
    use super::*;

    #[test]
    fn pptx_path_alone_does_not_load_pptxgenjs() {
        let code = r#"async function main() {
  fs.writeFileSync("output.pptx", "x", "utf-8");
  return { ok: true };
}"#;
        assert!(!needs_pptxgenjs(
            &normalize::normalize_script(code).to_lowercase()
        ));
        assert!(bundles_for_code(&normalize::normalize_script(code))
            .iter()
            .all(|(name, _, _)| *name != "pptxgenjs"));
    }

    #[test]
    fn pptxgenjs_usage_loads_bundle() {
        let code = "async function main() { const p = new PptxGenJS(); return { ok: !!p }; }";
        let lower = code.to_lowercase();
        assert!(needs_pptxgenjs(&lower));
        assert!(bundles_for_code(code)
            .iter()
            .any(|(name, _, _)| *name == "pptxgenjs"));
    }
}

#[cfg(test)]
mod binary_helper_tests {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use base64::{engine::general_purpose::STANDARD, Engine};

    fn run(code: &str, dir: &std::path::Path) -> Value {
        let sandbox = Sandbox::new(dir).unwrap();
        execute_script(&sandbox, code, Duration::from_secs(10), None, None)
            .unwrap_or_else(|e| panic!("{}", e.to_json_value()))
            .0
    }

    #[test]
    fn read_file_sync_returns_uint8array_with_to_string_base64() {
        let dir = tempfile::tempdir().unwrap();
        let data = [0u8, 159, 146, 150];
        std::fs::write(dir.path().join("blob.bin"), data).unwrap();
        let value = run(
            r#"
function main() {
  const bytes = fs.readFileSync("blob.bin");
  const viaBuf = Buffer.from(bytes).toString("base64");
  const viaEnc = fs.readFileSync("blob.bin", "base64");
  return {
    isUint8: bytes instanceof Uint8Array,
    len: bytes.length,
    match: viaBuf === viaEnc,
    viaBuf
  };
}
"#,
            dir.path(),
        );
        assert_eq!(value["isUint8"], true);
        assert_eq!(value["len"], 4);
        assert_eq!(value["match"], true);
        assert_eq!(value["viaBuf"], STANDARD.encode(data));
    }

    #[test]
    fn atob_btoa_roundtrip_all_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let value = run(
            r#"
function main() {
  let bin = "";
  for (let i = 0; i < 256; i++) bin += String.fromCharCode(i);
  const back = atob(btoa(bin));
  let match = back.length === 256;
  for (let i = 0; i < 256; i++) if (back.charCodeAt(i) !== i) match = false;
  return { match };
}
"#,
            dir.path(),
        );
        assert_eq!(value["match"], true);
    }

    #[test]
    fn binary_hot_path_under_2s() {
        let dir = tempfile::tempdir().unwrap();
        let mut data = vec![0u8; 4 * 1024 * 1024];
        for (i, b) in data.iter_mut().enumerate() {
            *b = (i.wrapping_mul(1103515245).wrapping_add(12345) % 256) as u8;
        }
        std::fs::write(dir.path().join("blob.bin"), &data).unwrap();
        let started = std::time::Instant::now();
        let value = run(
            r#"
function main() {
  const bytes = fs.readFileSync("blob.bin");
  const b64 = Buffer.from(bytes).toString("base64");
  const round = new TextDecoder().decode(new TextEncoder().encode("ok"));
  return { len: bytes.length, b64len: b64.length, round };
}
"#,
            dir.path(),
        );
        let elapsed = started.elapsed();
        assert_eq!(value["len"], 4 * 1024 * 1024);
        assert_eq!(value["round"], "ok");
        assert!(
            elapsed.as_secs_f64() < 2.0,
            "binary hot path took {elapsed:?}"
        );
    }

    #[test]
    fn infinite_loop_hits_iteration_limit() {
        let dir = tempfile::tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let started = std::time::Instant::now();
        let err = execute_script(
            &sandbox,
            "function main() { while (true) {} }",
            Duration::from_secs(30),
            None,
            None,
        )
        .unwrap_err()
        .to_json_value();
        let elapsed = started.elapsed();
        assert_eq!(err["error"], "loop iteration limit exceeded");
        assert!(
            elapsed.as_secs() < 30,
            "dead loop waited for timeout ({elapsed:?})"
        );
    }

    #[test]
    fn op_after_timeout_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), b"hi").unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let mut context = Context::default();
        ops::register(&mut context, &sandbox).unwrap();
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        ops::set_cancel_flag(Some(flag));
        let err = context
            .eval(boa_engine::Source::from_bytes("__doc_read('a.txt')"))
            .unwrap_err()
            .to_string();
        ops::set_cancel_flag(None);
        assert!(err.contains("cancelled by host"), "err={err}");

        let started = std::time::Instant::now();
        let err = execute_script(
            &sandbox,
            r#"
function main() {
  const start = Date.now();
  while (Date.now() - start < 1500) { doc_exists("a.txt"); }
  return doc_read("a.txt");
}
"#,
            Duration::from_secs(1),
            None,
            None,
        )
        .unwrap_err()
        .to_json_value();
        let elapsed = started.elapsed();
        assert!(
            elapsed.as_secs_f64() < 3.0,
            "runtime thread did not exit in time ({elapsed:?})"
        );
        assert_eq!(err["error"], "script timeout");
    }
}

#[cfg(all(test, not(debug_assertions)))]
mod pptx_image_bench {
    use super::*;
    use crate::core::sandbox::Sandbox;
    use image::{Rgb, RgbImage};

    fn write_noisy_jpeg(path: &std::path::Path, w: u32, h: u32) {
        let mut img = RgbImage::new(w, h);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = Rgb([
                (x.wrapping_mul(37).wrapping_add(y) % 256) as u8,
                (y.wrapping_mul(17).wrapping_add(x) % 256) as u8,
                ((x + y) % 256) as u8,
            ]);
        }
        img.save(path).unwrap();
    }

    #[test]
    fn pptx_three_large_images() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["a.jpg", "b.jpg", "c.jpg"] {
            write_noisy_jpeg(&dir.path().join(name), 4000, 3000);
        }
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let code = r#"
async function main() {
  const pptx = new PptxGenJS();
  for (const src of ["a.jpg", "b.jpg", "c.jpg"]) {
    const r = doc_image_resize(src, { maxEdge: 1600 });
    const slide = pptx.addSlide();
    const b64 = fs.readFileSync(r.path, "base64");
    slide.addImage({ data: "image/jpeg;base64," + b64, x: 0.5, y: 0.5, w: 9, h: 5 });
  }
  await pptx.writeFile({ fileName: "out.pptx" });
  return { ok: true };
}
"#;
        let started = std::time::Instant::now();
        let (value, written) = execute_script(&sandbox, code, Duration::from_secs(90), None, None)
            .unwrap_or_else(|e| panic!("{}", e.to_json_value()));
        let elapsed = started.elapsed();
        eprintln!("pptx_three_large_images elapsed={elapsed:?}");
        assert_eq!(value["ok"], true);
        assert!(written.iter().any(|p| p == "out.pptx"), "{written:?}");
        crate::tools::ooxml::validate::roundtrip_check(&dir.path().join("out.pptx"))
            .unwrap_or_else(|e| panic!("invalid pptx: {e}"));
        assert!(elapsed.as_secs() < 90, "three-image pptx took {elapsed:?}");
    }
}

fn js_to_json(context: &mut Context, value: &JsValue) -> Result<Value, String> {
    if value.is_undefined() {
        return Err("script returned undefined".into());
    }
    let json = value
        .to_json(context)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "script returned a non-JSON value".to_string())?;
    serde_json::from_value(json).map_err(|e| e.to_string())
}
