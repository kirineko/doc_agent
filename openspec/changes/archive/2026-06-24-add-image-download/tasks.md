## 1. 工具实现

- [x] 1.1 新增 `src-tauri/src/tools/image_download.rs`：`tool()` ToolSpec（占位 handler 返回 `NotImplemented`）+ JSON Schema（`urls` 必填、`dir` 可选默认 `images`）
- [x] 1.2 实现纯函数 helper：URL 校验 + SSRF 判定（http/https、环回/私网/链路本地/ULA/localhost/*.local）
- [x] 1.3 实现纯函数 helper：文件名 sanitize + 去重；格式→扩展名映射；按字节真实类型校验（`image::ImageReader`）
- [x] 1.4 实现 async `handler`：`reqwest::Client`（超时+UA）+ `futures_util::buffer_unordered(5)` 并发；单图 15 MiB 流式上限；写盘 + 按输入序返回 `downloaded[]` / `failed[]` / `count`

## 2. 注册与治理接线

- [x] 2.1 `tools/mod.rs` 导出 `pub mod image_download;`
- [x] 2.2 `registry.rs`：`default_tools()` 注册 `image_download::tool()`；`execute` 增加 `"image_download" => image_download::handler(ctx, args).await`（确认工具名不以 `web_` 开头 → 无条件可用）
- [x] 2.3 `io_plan.rs`：`image_download` 对 `dir`（缺省 `images`）申请 `SubtreeWrite` 锁；更新 `minimal_args_for` / `dummy_arg_value` 夹具，使 `all_default_tools_have_io_plan` 与 `io_plan_accepts_schema_required_args_only` 通过
- [x] 2.4 `changed_paths.rs`：`image_download` 读取 `result.downloaded[].path` 逐个上报 + 新增解析测试

## 3. 解耦指示

- [x] 3.1 `agent/loop_support.rs`：system prompt 追加一行"插入网络图片先 image_download 落地、再按本地路径引用"（无条件，不依赖 Key）
- [x] 3.2 `loop_support.rs` 或 `loop_runner_tests`：断言 system prompt 含该指示

## 4. 前端工具标签

- [x] 4.1 `src/lib/toolLabels.ts` 增加 `image_download: "下载图片"`
- [x] 4.2 `src/lib/toolLabels.test.ts` 的 `EXPECTED_TOOLS` 增加 `"image_download"`

## 5. 测试

- [x] 5.1 `image_download.rs` 单测：空 urls / 超 20 / 非 http(s) / 内网地址被拒；文件名 sanitize 与去重；格式→扩展名；真实字节校验（最小 PNG vs HTML 文本）；工具名已注册
- [x] 5.2 沙箱越界 `dir` 被拒单测

## 6. 验证

- [x] 6.1 `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [x] 6.2 `npm run typecheck && npm test && npm run build`

## 7. Review 后加固

- [x] 7.1 clippy `double_ended_iterator_last`（`filename_stem_from_url` 改 `next_back()`）
- [x] 7.2 重定向 SSRF：`redirect::Policy::custom` 对每次跳转目标复跑 `validate_url`，命中私网/环回即 `stop()`
- [x] 7.3 空/空白 `dir` 归一化到默认 `images`（避免对项目根申请过粗的 SubtreeWrite）
- [x] 7.4 补测：JPEG 分支 `format=jpg`；`reserve_name` 跳过磁盘已存在文件；重定向目标（含 ULA）被拒

## 8. 二轮 Review 加固

- [x] 8.1 [#3 一致性] 抽共享 helper `normalize_output_dir`，`handler` 与 `io_plan` 共用，消除空白/空串 `dir` 两侧口径不一致
- [x] 8.2 [#4/#5] `normalize_output_dir` 拒绝项目根（`.`）与 `.cache/` 首段（落盘但 `changed_paths` 静默过滤 → 产物不可见）
- [x] 8.3 [#2 回归] 补测 `blocks_noncanonical_ip_literals`：十进制/短写/十六进制/IPv4-mapped IPv6 全部被拒（实证：reqwest::Url 已归一化，CR 结论证伪）
- [x] 8.4 [#1 已知缺口] design D5 + spec 显式记录「公网域名 DNS 解析到私网 IP / rebinding 的首次连接」为不修的 MVP 已知限制
- [x] 8.5 [#9 已知项] design D7 记录 `urls` 原样进 `args_json` 持久化的隐私已知项（与 web_search/web_extract 同口径）
- [x] 8.6 [P2] `normalize_output_dir` 折叠冗余 `.` 段，堵住 `./.cache` 前缀绕过 `.cache` 检查
- [x] 8.7 [P1] 符号链接沙箱逃逸：`reserve_name` 用 `symlink_metadata`（不跟随）探测占用、跳过符号链接名；`fetch_one` 写盘改用 `OpenOptions::create_new(true)`（叠加防 TOCTOU、不覆盖既有文件/链接）；补测 `reserve_name_skips_dangling_symlink`
