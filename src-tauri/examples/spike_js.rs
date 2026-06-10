//! Spike: exceljs / pptxgenjs + boa_engine in embedded runtime (IIFE bundles).
use boa_engine::builtins::promise::PromiseState;
use boa_engine::object::builtins::JsPromise;
use boa_engine::{Context, JsValue, Source};
use std::fs;
use std::path::PathBuf;

fn eval_async(ctx: &mut Context, script: &str) -> Result<JsValue, Box<dyn std::error::Error>> {
    let value = ctx
        .eval(Source::from_bytes(script))
        .map_err(|e| e.to_string())?;
    let Some(obj) = value.as_object() else {
        return Ok(value);
    };
    let Ok(promise) = JsPromise::from_object(obj.clone()) else {
        return Ok(value);
    };
    ctx.run_jobs().map_err(|e| e.to_string())?;
    match promise.state() {
        PromiseState::Fulfilled(v) => Ok(v),
        PromiseState::Rejected(e) => Err(format!("promise rejected: {}", e.display()).into()),
        PromiseState::Pending => Err("promise still pending after run_jobs".into()),
    }
}

fn bytes_from(ctx: &mut Context, value: &JsValue) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let json = value
        .to_json(ctx)
        .map_err(|e| e.to_string())?
        .ok_or("non-JSON value")?;
    let arr = json.as_array().ok_or("expected byte array")?;
    Ok(arr
        .iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8))
        .collect())
}

const POLYFILLS: &str = r#"
var setTimeout = (fn, _ms, ...a) => { Promise.resolve().then(() => fn(...a)); return 0; };
var setImmediate = (fn, ...a) => { Promise.resolve().then(() => fn(...a)); return 0; };
var clearTimeout = () => {};
var clearImmediate = () => {};
var console = { log: () => {}, warn: () => {}, error: () => {} };
"#;

fn spike_exceljs() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from("/tmp/spike-exceljs.xlsx");
    let bundle = include_str!("../assets/js/exceljs.bundle.js");
    let script = format!(
        r#"
{POLYFILLS}
{bundle}
(async () => {{
  try {{
    const wb = new ExcelJS.Workbook();
    const ws = wb.addWorksheet("S1");
    ws.getCell("A1").value = "标题";
    ws.getCell("A1").font = {{ name: "Arial", bold: true }};
    const buf = await wb.xlsx.writeBuffer();
    return Array.from(new Uint8Array(buf));
  }} catch (e) {{
    throw new Error("step failed: " + e.message + "\n" + (e.stack ?? ""));
  }}
}})()
"#
    );
    let mut ctx = Context::default();
    let value = eval_async(&mut ctx, &script)?;
    let bytes = bytes_from(&mut ctx, &value)?;
    fs::write(&out, &bytes)?;
    println!("exceljs: wrote {} ({} bytes)", out.display(), bytes.len());
    Ok(())
}

fn spike_pptxgenjs() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from("/tmp/spike-pptxgenjs.pptx");
    let bundle = include_str!("../assets/js/pptxgenjs.bundle.js");
    let script = format!(
        r#"
{POLYFILLS}
{bundle}
(async () => {{
  const Pptx = PptxGenJS.default ?? PptxGenJS;
  const pres = new Pptx();
  const slide = pres.addSlide();
  slide.addText("Spike 标题", {{ x: 1, y: 1, fontSize: 24, bold: true }});
  const buf = await pres.write({{ outputType: "arraybuffer" }});
  return Array.from(new Uint8Array(buf));
}})()
"#
    );
    let mut ctx = Context::default();
    let value = eval_async(&mut ctx, &script)?;
    let bytes = bytes_from(&mut ctx, &value)?;
    fs::write(&out, &bytes)?;
    println!("pptxgenjs: wrote {} ({} bytes)", out.display(), bytes.len());
    Ok(())
}

fn main() {
    match spike_exceljs() {
        Ok(()) => {}
        Err(e) => println!("exceljs FAILED: {e}"),
    }
    match spike_pptxgenjs() {
        Ok(()) => {}
        Err(e) => println!("pptxgenjs FAILED: {e}"),
    }
}
