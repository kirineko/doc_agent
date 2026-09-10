## 1. 诊断修正（先行，零行为风险）

- [x] 1.1 `tools/runtime/diagnostics.rs`：`classify_error` 增加 `"script timeout"` 与 `"loop iteration limit exceeded"` 分支；`build_script_error` 新增 `timeout_secs: u64` 参数并为超时生成指向 `doc_image_info` / `doc_image_resize` 的 `hint`。验证：新增单测 `timeout_is_classified_separately`、`timeout_hint_mentions_resize`，`cargo test diagnostics` 通过。
- [x] 1.2 `tools/skill.rs`：schema `timeout_secs` 加 `minimum: 1`、`maximum: 120`、description；`run_handler` 记录 `requested` 与 `applied`，二者不等时在成功与失败结果均注入 `timeout_clamped: {requested, applied}`。验证：单测 `timeout_180_is_clamped_and_reported`（Mock 脚本 `return 1`，断言结果含 `timeout_clamped`）与 `timeout_60_has_no_clamp_field`。
- [x] 1.3 `runtime/mod.rs`：`execute_script` 把 `timeout.as_secs()` 传给 `build_script_error`；超时分支 detail 保持 `"script timeout"`。验证：现有测试通过，`cargo clippy -- -D warnings` 无告警。
- [x] 1.4 `src/lib/fileBusy.ts`：`formatToolResultError` 返回 `{ headline, detail?, hint? }`（新函数 `parseToolResultError`，旧函数保留为 `headline` 的薄包装以兼容调用点）；`detail` 超 600 字符按「前 400 + … + 后 200」截断。验证：`fileBusy.test.ts` 新增 3 个用例（超时 JSON、runtime error 带堆栈、非 JSON 回退），`npm test` 通过。
- [x] 1.5 `src/components/ToolChainPanel.tsx`：错误区改为标题行 + `<details>` 折叠 `detail` + 次级 `hint`；组件保持 ≤250 行（必要时抽 `ToolErrorBlock.tsx`）。验证：Testing Library 用例断言 `script timeout` 标题可见、点击展开后 `hint` 文本可见；`npm run typecheck` 通过。

## 2. 原生二进制热路径

- [x] 2.1 新建 `tools/runtime/ops_binary.rs`：实现 `arg_bytes` 辅助函数与 `__doc_read_bytes`、`__doc_b64_decode`、`__doc_b64_encode`、`__doc_utf8_encode`、`__doc_utf8_decode`、`__doc_latin1_encode`、`__doc_latin1_decode` 七个 op（后两个供 `atob`/`btoa`）；`ops::register` 调用 `ops_binary::register`。验证：Rust 单测直接 `context.eval` 调用各 op：0x00–0xFF 全字节往返、空输入、非法 base64 抛 `invalid base64`、`btoa` 遇 >0xFF 字符抛 `InvalidCharacterError`。
- [x] 2.2 `runtime/mod.rs` HELPERS：按 design D1 重写 `atob`/`btoa`/`TextEncoder`/`TextDecoder`/`__bytesToB64`/`__b64ToBytes`/`Buffer`/`fs.readFileSync`，删除 `__B64` 常量与手写循环；`Buffer.isBuffer` 收窄为 `instanceof Uint8Array`。验证：`tools/tests.rs` 全部 docx/pptx/xlsx/pdf-lib 生成用例通过（无脚本改动）；新增 `read_file_sync_returns_uint8array_with_to_string_base64`。
- [x] 2.3 性能回归测试：在 `tools/runtime` 测试中用 tempdir 生成 4MB 随机字节文件，脚本 `fs.readFileSync(p)` + `Buffer.from(bytes).toString('base64')` + `new TextDecoder().decode(new TextEncoder().encode(s))`，断言总耗时 <2s（`#[ignore]` 之外，CI 可跑）。验证：`cargo test binary_hot_path_under_2s` 通过。
- [x] 2.4 `runtime/mod.rs`：`Context::default()` 后 `context.runtime_limits_mut().set_loop_iteration_limit(20_000_000)`；`execute_script` 创建 `Arc<AtomicBool>` 取消标志，超时分支 `store(true)`；`ops.rs` 新增线程局部 `CANCEL_FLAG` 与 `check_cancelled(ctx) -> JsResult<()>`，所有 native op 入口首行调用。验证：单测 `infinite_loop_hits_iteration_limit`（`while(true){}`，断言 error 为 `loop iteration limit exceeded` 且耗时 < timeout）与 `op_after_timeout_is_rejected`（1s timeout，脚本 busy-loop 1.5s 后 `doc_read`，断言线程在 3s 内退出且错误含 `cancelled by host`）。

## 3. 图片 op

- [x] 3.1 确认 `Cargo.toml` 中 `image` feature 含 `jpeg`、`png`、`gif`、`webp`（默认 feature 已含；若被 `default-features = false` 覆盖则显式列出）。验证：`cargo tree -e features -i image | rg "jpeg|png|webp|gif"` 四项均出现。
- [x] 3.2 新建 `tools/runtime/ops_image.rs`：实现 `image_info`、`image_resize`（design D2），并注册 `__doc_image_info(path) -> JSON string`、`__doc_image_resize(path, optsJson) -> JSON string`；缩放写入走 `write_gate` 校验并 push `WRITTEN_PATHS`；>50MP 拒绝。验证：单测用 `image` crate 生成 4000×3000 JPEG / 800×600 PNG(alpha) / 非图片文件，覆盖 spec 五个 scenario。
- [x] 3.3 HELPERS 增加 `doc_image_info` / `doc_image_resize`（默认 `maxEdge: 1600, quality: 82`）。验证：脚本级测试 `doc_image_resize` 输出经 `doc_image_info` 复读尺寸正确，`written_paths` 含输出路径。
- [x] 3.4 端到端基准：测试脚本用 pptxgenjs 把 3 张 4000×3000 JPEG（各 ≈4MB）经 `doc_image_resize({maxEdge: 1600})` 插入 3 页并 `writeFile`；记录耗时，产物经 `ooxml` 校验为合法 PPTX。验证：`cargo test --release pptx_three_large_images -- --nocapture` 输出耗时；据结果在 design.md「Open Questions」填入最终默认 `maxEdge` 与推荐 `timeout_secs`（若 >30s 则文档推荐 `timeout_secs: 90`）。
- [x] 3.5（可选）注册独立工具 `image_info`（`tools/image_info.rs`，复用 `ops_image::image_info`，schema `{paths: string[]}`），中文标签「读取图片信息」加入 `src/lib/toolLabels.ts`。验证：handler 单测 + `toolLabels.test.ts`。

## 4. 文档

- [x] 4.1 `assets/skills/runtime/SKILL.md`：新增「图片与二进制」章节（`doc_image_info` / `doc_image_resize` / `fs.readFileSync` 三态 / 大图直接 base64 会超时的警告）；超时一节改为「默认 30s，上限 120s，超出截断并返回 `timeout_clamped`」。验证：`skill_read runtime` 单测断言包含 `doc_image_resize` 与 `maxEdge`。
- [x] 4.2 `assets/skills/pptx/pptxgenjs.md` Images 一节重写为推荐流程并给出保持宽高比的完整示例（见下）；`assets/skills/pptx/SKILL.md` 第 207 行「图片」条目同步。验证：`rg -n "0xFF|IHDR|SOF" assets/skills/pptx/` 无结果；`skill_read pptx doc=pptxgenjs.md` 单测断言含 `doc_image_resize`。

  ```javascript
  // 推荐：先取尺寸并缩图，再按宽高比放入指定区域
  function fitImage(path, box /* {x,y,w,h} inches */) {
    const r = doc_image_resize(path, { maxEdge: 1600 });      // 大图缩到 1600px 长边
    const ratio = r.width / r.height;
    let w = box.w, h = w / ratio;
    if (h > box.h) { h = box.h; w = h * ratio; }
    const b64 = fs.readFileSync(r.path, "base64");
    const mime = r.path.endsWith(".png") ? "image/png" : "image/jpeg";
    return { data: `${mime};base64,${b64}`, x: box.x + (box.w - w) / 2, y: box.y + (box.h - h) / 2, w, h };
  }
  slide.addImage(fitImage("assets/photo.jpg", { x: 0.5, y: 1.2, w: 9, h: 4 }));
  ```

- [x] 4.3 `tools/skill.rs` 工具 description 追加一句：`Images: use doc_image_info/doc_image_resize before addImage; never parse image headers in JS.`。验证：schema 单测断言 description 含 `doc_image_resize`。

## 5. 收尾

- [x] 5.1 `openspec/specs/project-backlog/spec.md`：BL-010、BL-012 条目追加「2026-09 `fix-skill-run-image-perf` 部分缓解（loop limit + op 取消标志），根治仍待办」。验证：文件 diff 仅两处追加。
- [x] 5.2 全量门禁：`cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`；`npm run typecheck && npm test && npm run build`。验证：全部通过。
- [ ] 5.3 手动验证：在真实项目放 3 张 >3MB 照片，让 Agent 生成含图 PPT；观察工具链卡片无 `script timeout`，PPT 可用 PowerPoint/WPS 打开，`.cache/images/` 出现缩图。验证：截图附 PR。

- [x] 5.4 Review fix：默认缩图文件名加入来源与参数哈希；回归测试覆盖同名图片、不同质量和重复调用。
