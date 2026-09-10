## Context

见 `proposal.md - Why`。当前数据流中每一步都在 boa 里逐字节循环：

```
磁盘 JPEG 4.3MB
  | Rust fs::read -> base64 (原生, 快)
  v
__doc_read ==> 5.7M 字符 JS 字符串
  | HELPERS.atob   JS 循环 5.7M 次, out += String.fromCharCode  (boa JsString 不可变, 接近 O(n^2))
  | __b64ToBytes   JS 循环 4.3M 次
  v
Uint8Array  <- 模型为算宽高比再扫 SOF/IHDR
  | slide.addImage({ data: "image/jpeg;base64," + b64 })
  | pptxgenjs.write("base64") -> JSZip base64.decode + CRC32 + 整包 base64 encode (JS)
  v
__doc_write (Rust 解码, 快)
```

约束：

- 引擎 `boa_engine 0.21.1`，无中断钩子（`Context` 没有 interrupt handler），只有 `RuntimeLimits { loop_iteration, recursion, stack_size }`。
- 每次 `skill_run` 在独立 OS 线程（32MB 栈）跑一个 `Context`；`execute_script` 用 `recv_timeout` 等结果，超时不杀线程。
- 现有 native op 注册方式：`context.register_global_builtin_callable(js_string!(name), argc, NativeFunction::from_copy_closure_with_captures(...))`，位于 `tools/runtime/ops.rs`（217 行，接近上限，新 op 必须拆文件）。
- `image = "0.25"` 已是依赖（`tools/image_download.rs` 用 `ImageReader` 判断格式）。
- pptxgenjs bundle 内部 JSZip 仍在 JS 做 base64/CRC，本 change 不改 bundle。

## Goals / Non-Goals

**Goals:**

- 4MB 级图片进 PPT 的端到端 `skill_run` 在默认 30s 内完成（图片缩到 ≤1600px 长边后 JSZip 侧数据量 ≤ 500KB/张）。
- 超时错误让模型第一眼就知道「是慢不是 bug」，并给出正确修复方向。
- 既有脚本零修改。

**Non-Goals:**

- 替换 JSZip 的 base64/CRC 实现（需 patch bundle，另立 change）。
- boa heap 上限（BL-012）。
- 超时后强杀线程（boa 无安全中断；BL-010 另立）。
- 图片格式转换以外的图像处理（滤镜、裁剪等）。

## Decisions

### D1. 二进制 API 走 native op，JS 只做薄包装

**选择**：Rust 侧新增 `__doc_read_bytes(path) -> Uint8Array`、`__doc_b64_decode(str) -> Uint8Array`、`__doc_b64_encode(u8arr) -> string`、`__doc_utf8_decode(u8arr) -> string`、`__doc_utf8_encode(str) -> Uint8Array`；HELPERS 中 `atob/btoa/Buffer/TextEncoder/TextDecoder/fs.readFileSync` 改为调用这些 op。

**替代**：(a) 优化 JS polyfill（用数组 join 代替 `+=`）——仍是解释执行，只能提速常数倍，4MB 仍要几十秒；(b) 换 QuickJS/V8——依赖与体积剧变，超出范围。

**boa API 依据**（0.21.1 源码已核对）：

- 构造：`JsUint8Array::from_iter(bytes: Vec<u8>, ctx)`（`object/builtins/jstypedarray.rs:1028`）。
- 读取：`JsUint8Array::from_object(obj)?` → `.buffer(ctx)?` → `JsArrayBuffer::from_object(...)` → `.data()` 得 `GcRef<[u8]>`，配合 `.byte_offset(ctx)` / `.byte_length(ctx)` 切片。

```rust
// tools/runtime/ops_binary.rs（新文件）
use base64::{engine::general_purpose::STANDARD, Engine};
use boa_engine::object::builtins::{JsArrayBuffer, JsUint8Array};
use boa_engine::{js_string, Context, JsNativeError, JsResult, JsValue, NativeFunction};

pub fn register(context: &mut Context, root: std::path::PathBuf) -> JsResult<()> {
    register_read_bytes(context, root)?;
    context.register_global_builtin_callable(
        js_string!("__doc_b64_decode"), 1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            let s = arg_string(args, 0, ctx)?;
            let bytes = STANDARD.decode(s.trim()).map_err(|e| {
                JsNativeError::typ().with_message(format!("invalid base64: {e}"))
            })?;
            Ok(JsUint8Array::from_iter(bytes, ctx)?.into())
        }),
    )?;
    context.register_global_builtin_callable(
        js_string!("__doc_b64_encode"), 1,
        NativeFunction::from_copy_closure(|_this, args, ctx| {
            let bytes = arg_bytes(args, 0, ctx)?;
            Ok(JsValue::from(js_string!(STANDARD.encode(bytes))))
        }),
    )?;
    // __doc_utf8_decode / __doc_utf8_encode 同构，略
    Ok(())
}

/// 从 Uint8Array（或任何 TypedArray）参数取字节切片副本。
pub(crate) fn arg_bytes(args: &[JsValue], idx: usize, ctx: &mut Context) -> JsResult<Vec<u8>> {
    let obj = args.get(idx).and_then(JsValue::as_object).cloned()
        .ok_or_else(|| JsNativeError::typ().with_message("Uint8Array required"))?;
    let view = JsUint8Array::from_object(obj)?;
    let offset = view.byte_offset(ctx)?;
    let len = view.byte_length(ctx)?;
    let buf = JsArrayBuffer::from_object(
        view.buffer(ctx)?.as_object().cloned()
            .ok_or_else(|| JsNativeError::typ().with_message("detached buffer"))?,
    )?;
    let data = buf.data().ok_or_else(|| JsNativeError::typ().with_message("detached buffer"))?;
    Ok(data[offset..offset + len].to_vec())
}
```

HELPERS 对应改写（保持既有名字与语义）：

```javascript
var atob = (b64) => __doc_utf8_decode_latin1(__doc_b64_decode(b64)); // 见 D1 注 1
var btoa = (bin) => __doc_b64_encode(__doc_latin1_encode(bin));
class TextEncoder { encode(s) { return __doc_utf8_encode(s); } }
class TextDecoder { decode(buf) { return __doc_utf8_decode(__toU8(buf)); } }
var __toU8 = (d) => d instanceof Uint8Array ? d
  : d instanceof ArrayBuffer ? new Uint8Array(d)
  : ArrayBuffer.isView(d) ? new Uint8Array(d.buffer, d.byteOffset, d.byteLength)
  : new Uint8Array(d);
var __bytesToB64 = (data) => __doc_b64_encode(__toU8(data));
var __b64ToBytes = (b64) => __doc_b64_decode(b64);
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
var __wrapBuffer = (bytes) => Object.assign(bytes, {
  toString(enc) { return enc === "base64" ? __doc_b64_encode(this) : __doc_utf8_decode(this); }
});
var fs = { /* ... */
  readFileSync: (filePath, encoding) => {
    const enc = encoding && String(encoding).toLowerCase();
    if (enc === "base64") return doc_read(filePath);              // 仍走 __doc_read（Rust 编码）
    const bytes = __doc_read_bytes(filePath);                      // 新 op
    if (enc === "utf-8" || enc === "utf8") return __doc_utf8_decode(bytes);
    return __wrapBuffer(bytes);
  },
};
```

注 1：`atob` 语义是 base64 → **binary string**（每字符 0–255）。用 `String::from_utf8_lossy` 会破坏 ≥0x80 字节，需要 latin1 映射：Rust 侧 `bytes.iter().map(|&b| b as char).collect::<String>()` 再转 `JsString`（boa `JsString::from(&str)` 内部转 UTF-16，每字节一个 code unit，正确）。`btoa` 反向：取 UTF-16 code units，任一 >255 抛 `InvalidCharacterError`。

### D2. 图片 op 用 `image` crate 的 header-only 读取 + 缩放

```rust
// tools/runtime/ops_image.rs（新文件）
use image::{ImageReader, imageops::FilterType, ImageFormat};

#[derive(serde::Serialize)]
pub struct ImageInfo { pub width: u32, pub height: u32, pub mime: &'static str, pub bytes: u64 }

pub fn image_info(path: &std::path::Path) -> Result<ImageInfo, String> {
    let bytes = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    let reader = ImageReader::open(path).map_err(|e| e.to_string())?
        .with_guessed_format().map_err(|e| e.to_string())?;
    let mime = match reader.format() {
        Some(ImageFormat::Png) => "image/png",
        Some(ImageFormat::Jpeg) => "image/jpeg",
        Some(ImageFormat::Gif) => "image/gif",
        Some(ImageFormat::WebP) => "image/webp",
        other => return Err(format!("unsupported image format: {other:?}")),
    };
    let (width, height) = reader.into_dimensions().map_err(|e| e.to_string())?; // 只读 header
    Ok(ImageInfo { width, height, mime, bytes })
}

pub struct ResizeOpts { pub max_edge: u32, pub quality: u8, pub out: Option<String> }

/// 返回 (相对输出路径, 新宽, 新高, 输出字节数)。长边已 ≤ max_edge 且格式为 JPEG/PNG 时直接返回原路径。
pub fn image_resize(sandbox: &Sandbox, rel: &str, opts: ResizeOpts) -> Result<(String, u32, u32, u64), String> {
    let src = sandbox.resolve(rel).map_err(|e| e.to_string())?;
    let info = image_info(&src)?;
    let needs_resize = info.width.max(info.height) > opts.max_edge;
    let needs_reencode = !matches!(info.mime, "image/jpeg" | "image/png");
    if !needs_resize && !needs_reencode && opts.out.is_none() {
        return Ok((rel.to_string(), info.width, info.height, info.bytes));
    }
    let img = image::open(&src).map_err(|e| e.to_string())?;
    let img = if needs_resize { img.resize(opts.max_edge, opts.max_edge, FilterType::Triangle) } else { img };
    let out_rel = opts.out.unwrap_or_else(|| {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (rel, opts.max_edge, opts.quality).hash(&mut hasher);
        let hash = hasher.finish();
        format!(".cache/images/{}-{}-{hash:016x}.jpg", stem(rel), opts.max_edge)
    });
    let out_abs = sandbox.resolve_for_write(&out_rel).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(out_abs.parent().unwrap()).map_err(|e| e.to_string())?;
    let mut buf = std::io::Cursor::new(Vec::new());
    // PNG 保留透明；其余统一 JPEG（体积最小，pptxgenjs 支持）
    if info.mime == "image/png" && img.color().has_alpha() {
        img.write_to(&mut buf, ImageFormat::Png).map_err(|e| e.to_string())?;
    } else {
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, opts.quality);
        img.to_rgb8().write_with_encoder(enc).map_err(|e| e.to_string())?;
    }
    std::fs::write(&out_abs, buf.get_ref()).map_err(|e| e.to_string())?;
    Ok((out_rel, img.width(), img.height(), buf.get_ref().len() as u64))
}
```

- 写入必须走现有 write gate（`ops::set_runtime_write_gate` / `RuntimeWriteGate`）并记入 `WRITTEN_PATHS`，与 `__doc_write` 一致；`.cache/` 路径按 `project-cache-layout` spec 不出现在构建产物。
- JS 侧：`var doc_image_info = (p) => JSON.parse(__doc_image_info(p)); var doc_image_resize = (p, o) => JSON.parse(__doc_image_resize(p, JSON.stringify(o ?? {})));`，默认 `maxEdge: 1600, quality: 82`。
- **替代**：让模型继续手写 JPEG SOF 扫描——这正是本次故障的直接触发点；或新增独立 `image_info` 工具——留作 tasks 可选项（同一 Rust 函数复用，成本极低，但增加工具数）。

### D3. 超时分类、上限告知与运行时保护

```rust
// diagnostics.rs
fn classify_error(detail: &str) -> &'static str {
    let lower = detail.to_lowercase();
    if lower.contains("script timeout") { return "script timeout"; }
    if lower.contains("loop iteration limit") { return "loop iteration limit exceeded"; }
    if lower.contains("syntax") || lower.contains("unexpected") { return "JavaScript parse error"; }
    "JavaScript runtime error"
}

pub fn build_script_error(code: &str, detail: &str, script_path: Option<&str>, timeout_secs: u64) -> Value {
    // ...现有逻辑...
    if out["error"] == "script timeout" {
        out["hint"] = json!(format!(
            "脚本在 {timeout_secs}s 内未完成（上限 120s）。本运行时是解释器，对多 MB 二进制逐字节处理极慢；\
             这通常不是代码 bug。请：1) 用 doc_image_info(path) 取尺寸而非解析文件头；\
             2) 大图先 doc_image_resize(path, {{maxEdge: 1600}}) 再 addImage；\
             3) 减少单次脚本处理的文件数。加大 timeout_secs 通常无效。"
        ));
    }
    out
}
```

```rust
// skill.rs：schema 与 clamp 告知
"timeout_secs": { "type": "integer", "default": 30, "minimum": 1, "maximum": 120,
  "description": "Hard cap 120. Larger values are clamped and reported as timeout_clamped." }
// run_handler
let requested = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(30);
let timeout_secs = requested.clamp(1, 120);
// 成功/失败结果均附带 "timeout_clamped": {"requested": 180, "applied": 120}（当 requested != timeout_secs）
```

```rust
// runtime/mod.rs run_in_thread
let mut context = Context::default();
context.runtime_limits_mut().set_loop_iteration_limit(20_000_000); // 单循环上限；bundle 加载不受影响
// 取消标志：execute_script 创建 Arc<AtomicBool>，超时时 store(true)；
// 每个 native op 入口调用 ops::check_cancelled(ctx)? -> 已取消则抛 "script cancelled by host"
```

- `loop_iteration_limit` 语义是**单个循环**的迭代上限，2×10⁷ 足够覆盖合理业务循环（原本 4.3MB 逐字节循环 ≈ 4.3×10⁶，仍在限内——所以它保护的是死循环，不是慢循环）。
- 取消标志只能在脚本调用 native op 时生效；纯 JS 死算无法中断（boa 限制，记入 Risks）。

### D4. UI 展示 `detail`

`formatToolResultError` 改为返回 `{ headline: string; detail?: string }`，`ToolChainPanel` 在 headline 下渲染可折叠 `detail`（`<details>`，默认收起，`detail` 超 600 字符截断并保留尾部）。保留现有 `file_busy` 分支。

## Risks / Trade-offs

- [boa 无中断，纯 JS 死循环/慢循环仍占 CPU 直到自然结束] → `loop_iteration_limit` 兜底死循环；慢循环通过 native 热路径消除主要来源；日志记录僵尸线程数（`skill_run` 线程 name 便于 `ps -M` 观察）。
- [`from_iter` 逐字节 `flat_map(to_ne_bytes)` 有一次拷贝] → 4MB 级别 <10ms，可接受；如需零拷贝可改 `AlignedVec::from_iter(0, bytes)` + `JsArrayBuffer::from_byte_block` + `JsUint8Array::from_array_buffer`。
- [pptxgenjs 内部 JSZip 仍在 JS 里做 base64/CRC] → 缩图后单张 ≤500KB，3 张约 1.5MB，实测预算按 366KB≈11s 的既有数据外推约 45s——**仍可能超默认 30s**。缓解：pptxgenjs.md 指引默认 `maxEdge: 1280` 且多图 PPT 传 `timeout_secs: 90`；tasks 里加基准测试确定默认值；JSZip 原生化列为 follow-up。
- [`image` crate 解码 4MB JPEG 的内存峰值 ≈ 宽×高×3] → 12MP 约 36MB，主进程可承受；`doc_image_resize` 拒绝 >50MP 图片并返回明确错误。
- [`atob` latin1 语义与现有 polyfill 行为差异] → 现有 polyfill 同为 byte→charCode，语义一致；增加单测覆盖 0x00–0xFF 全字节往返。
- [既有脚本依赖 `Buffer.from(...)` 返回值上的 `Object.assign` 扩展属性] → `__wrapBuffer` 保留 `toString(enc)`；`isBuffer` 收窄为 `instanceof Uint8Array`（原实现把任何有 `.buffer` 的对象视作 Buffer，属修正）。

## Migration Plan

1. 先合并 D3（分类 + 上限告知 + UI detail）：纯诊断改动，零行为风险，立即改善模型自修方向。
2. 再合并 D1（native 二进制）：替换 polyfill，跑全部 `tools/tests.rs` 中 docx/pptx/xlsx 生成用例回归。
3. 最后 D2（图片 op）+ 文档更新 + 基准测试。
4. 回滚：三步彼此独立，任一步可单独 revert；HELPERS 常量替换是原子的。

## Open Questions

- 默认 `maxEdge`：**1600**。`cargo test --release pptx_three_large_images`（3 张 4000×3000 JPEG → resize → 3 页 PPTX）release 耗时 **8.2s**，低于默认 30s，无需把默认 `timeout_secs` 提到 90。多图或未缩图场景仍可按需传 `timeout_secs: 90`。
- 是否顺带注册独立 `image_info` 工具（非 skill_run 内）：已注册，schema `{paths: string[]}`，复用 `ops_image::image_info`。

### Review fix: 缩图文件名去冲突

默认文件名增加源相对路径、maxEdge、quality 的 64-bit 哈希后缀，保留 stem、尺寸与扩展名，不新增依赖。显式 out 和小图直通行为保持原样；相同输入参数重复调用返回相同路径。
