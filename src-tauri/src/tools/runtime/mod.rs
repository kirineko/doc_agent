mod diagnostics;
mod normalize;
mod ops;

use super::ToolError;
use crate::core::sandbox::Sandbox;
use boa_engine::builtins::promise::PromiseState;
use boa_engine::object::builtins::JsPromise;
use boa_engine::{Context, JsValue, Source};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

const HELPERS: &str = r#"
const doc_read = (path) => __doc_read(path);
const doc_write = (path, data) => __doc_write(path, data);
const doc_log = (...args) => __doc_log(JSON.stringify(args));
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
const __B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
var btoa = (bin) => {
  let out = "";
  for (let i = 0; i < bin.length; i += 3) {
    const a = bin.charCodeAt(i), b = bin.charCodeAt(i + 1), c = bin.charCodeAt(i + 2);
    out += __B64[a >> 2] + __B64[((a & 3) << 4) | (isNaN(b) ? 0 : b >> 4)]
      + (isNaN(b) ? "=" : __B64[((b & 15) << 2) | (isNaN(c) ? 0 : c >> 6)])
      + (isNaN(c) ? "=" : __B64[c & 63]);
  }
  return out;
};
var atob = (b64) => {
  const clean = b64.replace(/=+$/, "");
  let out = "", bits = 0, acc = 0;
  for (let i = 0; i < clean.length; i++) {
    acc = (acc << 6) | __B64.indexOf(clean[i]);
    bits += 6;
    if (bits >= 8) { bits -= 8; out += String.fromCharCode((acc >> bits) & 0xff); }
  }
  return out;
};
class TextEncoder {
  encode(s) {
    const out = [];
    for (let i = 0; i < s.length; i++) {
      let c = s.codePointAt(i);
      if (c > 0xffff) i++;
      if (c < 0x80) out.push(c);
      else if (c < 0x800) out.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
      else if (c < 0x10000) out.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
      else out.push(0xf0 | (c >> 18), 0x80 | ((c >> 12) & 0x3f), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
    }
    return new Uint8Array(out);
  }
}
class TextDecoder {
  decode(buf) {
    const b = new Uint8Array(buf.buffer ?? buf, buf.byteOffset ?? 0, buf.byteLength ?? buf.length);
    let s = "", i = 0;
    while (i < b.length) {
      const x = b[i];
      let c;
      if (x < 0x80) { c = x; i += 1; }
      else if (x < 0xe0) { c = ((x & 0x1f) << 6) | (b[i+1] & 0x3f); i += 2; }
      else if (x < 0xf0) { c = ((x & 0x0f) << 12) | ((b[i+1] & 0x3f) << 6) | (b[i+2] & 0x3f); i += 3; }
      else { c = ((x & 0x07) << 18) | ((b[i+1] & 0x3f) << 12) | ((b[i+2] & 0x3f) << 6) | (b[i+3] & 0x3f); i += 4; }
      s += String.fromCodePoint(c);
    }
    return s;
  }
}
var __bytesToB64 = (data) => {
  const bytes = data instanceof Uint8Array
    ? data
    : new Uint8Array(data && data.buffer ? data.buffer : data);
  let bin = "";
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  return btoa(bin);
};
var __b64ToBytes = (b64) => {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
};
var doc_write_bytes = (path, data) => __doc_write(path, __bytesToB64(data));
var Buffer = {
  from: (data, encoding) => {
    let bytes;
    if (typeof data === "string" && encoding === "base64") bytes = __b64ToBytes(data);
    else if (data instanceof ArrayBuffer) bytes = new Uint8Array(data);
    else if (ArrayBuffer.isView(data)) bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    else if (typeof data === "string") bytes = new TextEncoder().encode(data);
    else bytes = new Uint8Array(data);
    return Object.assign(bytes, {
      toString(enc) {
        return enc === "base64" ? __bytesToB64(this) : new TextDecoder().decode(this);
      }
    });
  },
  alloc: (n) => new Uint8Array(n),
  isBuffer: (x) => x instanceof Uint8Array || !!(x && x.buffer),
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
    const b64 = doc_read(filePath);
    const enc = encoding && String(encoding).toLowerCase();
    if (enc === "utf-8" || enc === "utf8") {
      return new TextDecoder().decode(__b64ToBytes(b64));
    }
    if (enc === "base64") return b64;
    // Node 语义：无 encoding 返回 Buffer（字节），适用于图片等二进制
    return Buffer.from(b64, "base64");
  },
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
) -> Result<(Value, Vec<String>), ToolError> {
    let root = sandbox.root().to_path_buf();
    let code = code.to_string();
    let code_for_error = code.clone();
    let script_path = script_path.map(str::to_string);
    let (tx, rx) = mpsc::channel();

    // boa 解析/执行大型 bundle（如 exceljs ~1MB）递归较深，需要加大栈。
    std::thread::Builder::new()
        .name("skill_run".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_in_thread(&root, &code, timeout)
            }))
            .unwrap_or_else(|_| Err("script panicked".into()));
            let _ = tx.send(result);
        })
        .map_err(|e| ToolError::Execution(format!("spawn runtime thread: {e}")))?;

    match rx.recv_timeout(timeout + Duration::from_secs(2)) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(ToolError::Structured(diagnostics::build_script_error(
            &code_for_error,
            &e,
            script_path.as_deref(),
        ))),
        Err(_) => Err(ToolError::Structured(diagnostics::build_script_error(
            &code_for_error,
            "script timeout",
            script_path.as_deref(),
        ))),
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
    let suspicious = msg.contains("not a callable function")
        || msg.contains("not a constructor")
        || msg.contains("is not defined")
        || msg.contains("not an object");
    if suspicious {
        format!(
            "{msg}\n提示：嵌入式 JS 运行时。require('fs'|'path'|'exceljs'|'pptxgenjs'|'docx'|'pdf-lib') 已映射；\
             编辑 docx XML：fs.readFileSync(path,'utf-8') + replace + fs.writeFileSync。\
             勿在末尾写 main()（运行时自动调用）。\
             保存 xlsx：await wb.xlsx.writeFile('out.xlsx')；pptx：await pptx.writeFile({{ fileName: 'out.pptx' }})。"
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
    if lower.contains("pptxgenjs") || lower.contains("pptx") {
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
