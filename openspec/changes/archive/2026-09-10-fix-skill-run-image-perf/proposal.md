## Why

用图片生成 PPT 时 `skill_run` 频繁失败，UI 显示「JavaScript runtime error」。本机 `tool_calls` 表证实真实原因是 **`script timeout`**：boa 是纯解释器，运行时的 `atob` / `Buffer.from(b64)` / `TextDecoder` 全是 JS polyfill，pptxgenjs 内部 JSZip 的 base64 解码、CRC32、再编码也全在 JS 逐字节循环。一张 366KB PNG 读取 + 解析要 11–21s，4.3MB 照片按线性外推超过 120s 硬上限。

三个放大因素让模型越修越偏：`timeout_secs` 上限 120 未告知（模型传 180 被静默截断后推断「>180s 仍不完成 → 死循环」）；超时被归类成 runtime error 且 UI 只显示 `error` 不显示 `detail`；超时后 boa 线程不会终止，持续占用 CPU，后续每次重试都在更差条件下运行。

## What Changes

- **原生二进制热路径**：`fs.readFileSync(path)`（无 encoding）直接返回 Rust 构造的 `Uint8Array`；`atob` / `btoa` / `Buffer.from(b64, 'base64')` / `buf.toString('base64')` / `TextEncoder` / `TextDecoder` 改为调用 Rust native op，JS 侧不再逐字节循环。
- **图片元数据与缩放 op**：新增 `doc_image_info(path)` 返回 `{width, height, mime, bytes}`；新增 `doc_image_resize(path, {maxEdge, quality, out})` 用已有 `image` crate 缩图并写入沙箱（默认 `.cache/images/`），返回新路径与尺寸。模型不再需要自己扫 JPEG SOF / PNG IHDR。
- **超时诊断修正**：`script timeout` 独立分类为 `"script timeout"`，`hint` 指向缩小数据规模 / 使用原生图片 op，而非 fs_patch 修代码；`timeout_secs` schema 与 SKILL.md 明示 `maximum: 120`，超出时结果中附 `timeout_clamped` 告知。
- **运行时保护**：启用 boa `loop_iteration_limit`（单循环上限，默认 2×10⁷）使真正的死循环确定性抛错；超时时通过 native op 检查取消标志，让仍在调用 op 的脚本尽早退出，减少僵尸线程。
- **UI**：工具链卡片在 `tool_result.ok=false` 时同时展示 `error` 与 `detail`（`detail` 可折叠）。
- **文档**：`runtime/SKILL.md` 新增「图片与二进制」章节；`pptx/pptxgenjs.md` Images 一节改为「先 `doc_image_info` 取尺寸 → 大图先 `doc_image_resize` → 再 `addImage`」的推荐流程。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `script-runtime`：新增原生二进制/base64 op、图片元数据与缩放 op、超时分类与上限告知、循环迭代上限；修改「运行时沙箱」requirement 明示超时上限 120s。
- `workspace-ui`：工具链卡片错误展示新增 `detail` 折叠区。

## Impact

- Rust：`src-tauri/src/tools/runtime/{mod.rs, ops.rs, diagnostics.rs}`（新增 `ops_binary.rs`、`ops_image.rs` 以守住 300 行上限）、`src-tauri/src/tools/skill.rs`（schema）。
- 前端：`src/lib/fileBusy.ts`（`formatToolResultError` 返回结构化对象）、`src/components/ToolChainPanel.tsx`。
- 文档：`src-tauri/assets/skills/runtime/SKILL.md`、`src-tauri/assets/skills/pptx/pptxgenjs.md`、`src-tauri/assets/skills/pptx/SKILL.md`。
- 依赖：**无新增 crate**。`image = "0.25"`、`base64 = "0.22"` 已在 `Cargo.toml`；`image` 需确认启用 `jpeg`/`png`/`webp`/`gif` feature（当前 `image_download.rs` 已用 `ImageReader` 解码这四种格式）。
- 兼容性：JS API 名称与返回类型保持不变（`fs.readFileSync(p)` 仍返回可 `.toString('base64')` 的 Uint8Array-like），既有脚本无需修改。
- 关联 backlog：BL-010（skill_run 可中断）、BL-012（boa heap 上限）部分缓解，不在此 change 内完全解决。
