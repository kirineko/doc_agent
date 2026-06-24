# 设计：Agent 图片下载（image_download）

## Context

- `skill_run` 用 boa_engine（非 Node）执行 docx-js / pptxgenjs / PDFLib，**无 fetch、无网络**，只能引用本地文件；`typst_to_pdf` / `html_to_pdf` 亦只读沙箱内文件。
- 已有异步工具范式：handler 占位返回 `NotImplemented`，真正实现挂在 `ToolRegistry::execute` 的 async 特判分支（见 `web_search` / `html_to_pdf` / `pdf_read`）。
- 文件写入工具经 `Sandbox::resolve_for_write` 限定项目根内；执行前 `plan_tool_io` 申请文件锁；成功后 `extract_changed_paths` 把产物相对路径上报 UI（`@` 索引、文件浏览、产物面板）。`changed_paths` 会过滤 `.cache/` 下中间产物。
- 依赖现状：`reqwest = { features = ["json","stream","rustls-tls"] }`、`image = "0.25"`、`futures-util = "0.3"` 均已在 `Cargo.toml`。
- `is_web_tool(name) = name.starts_with("web_")`；`tools_for_model` 按 `include_web` 过滤 web 工具。Tavily Key 决定 `include_web`。

## Goals / Non-Goals

**Goals：**

- 一个 `image_download` 工具：批量下载公网图片到项目目录，返回本地相对路径供文档引用。
- 与 Word/PPT/PDF 生成解耦：工具不感知文档格式，文档侧只引用本地路径。
- 始终可用（无需任何 Key），健壮（部分失败、类型/体量校验），安全（http(s) + SSRF 防护 + 沙箱内输出）。

**Non-Goals：**

- 不做图片处理（裁剪/缩放/压缩/格式转换）——仅原样落地。
- 不抓取网页里的 `<img>`（那是 `web_extract` 之后模型自行决定 URL）。
- 不做断点续传、不做下载历史 UI、不做缓存去重跨会话复用。
- 不在 `skill_run` 运行时内提供 fetch（保持运行时无网络的安全边界）。

## Decisions

### D1：独立工具 `image_download`，而非塞进文档 skill

下载与"生成 docx/pptx/pdf"是两件正交的事：

| 关注点 | 归属 |
|---|---|
| 取图落地（网络 I/O、类型校验、命名、写盘） | `image_download` |
| 按本地路径插图（docx-js `ImageRun` / pptxgenjs `addImage` / typst `image()` / html `<img>`） | 各文档 skill（`skill_run` / `typst_to_pdf` / `html_to_pdf`） |

解耦的结构性保证：`image_download` 是独立 `tools/image_download.rs`，不被任何文档工具调用；文档工具只读本地文件。命名与 `image_read` 同族（`image_*`），但**不**以 `web_` 开头 → 不受 `include_web` 门禁，无条件可用。

### D2：工具 I/O 契约

入参：

```jsonc
{
  "urls": ["https://.../a.png", "https://.../b.jpg"],   // 必填，1–20 个 http(s) URL
  "dir": "images"                                         // 可选，项目相对输出目录，默认 "images"，自动创建
}
```

出参：

```jsonc
{
  "dir": "images",
  "downloaded": [
    { "url": "...", "path": "images/a.png", "bytes": 20480, "width": 800, "height": 600, "format": "png" }
  ],
  "failed": [
    { "url": "...", "error": "not an image (detected: text/html error page)" }
  ],
  "count": 1            // 成功数
}
```

- 始终返回 `Ok`（除入参非法外），把逐 URL 失败放进 `failed[]`，让模型据此重试/换图，而非中断 loop。
- `changed_paths` 取 `downloaded[].path`（逐文件上报，便于 `@` 引用具体图片）；默认 `images/` 非 `.cache/`，不会被过滤。

为何不支持自定义文件名/`{url,filename}` 对象：模型直接使用返回的 `path` 即可引用，无须预先指定名字；保持 schema 简单、降低误用。

### D3：限额与默认值

| 项 | 值 | 理由 |
|---|---|---|
| `urls` 数量 | 1–20 | 批量够用，约束内存与请求量 |
| 单图字节上限 | 15 MiB | 文档插图足够；流式累计字节超限即中断该 URL |
| 单请求超时 | 30 s | 与 `html_to_pdf` 同量级 |
| 并发 | 5 | 限制峰值内存/连接（峰值 ≈ 5 × 15 MiB） |
| 默认目录 | `images` | 用户可见、可 `@` 引用 |

### D4：下载与命名流程

1. 校验入参：`urls` 非空、≤20；逐个校验 URL（D5）。
2. 构造单个 `reqwest::Client`（超时 + UA），用 `futures_util` `buffer_unordered(5)` 并发抓取。
3. 每个任务：`GET` → 流式读 body，累计字节，超 15 MiB 即中断标记失败 → 得到 `bytes`。
4. **按字节真实类型校验**：`image::ImageReader::with_guessed_format()` 取 `format` 与 `(width,height)`；解析失败 → 失败（防 HTML/JSON 错误页伪装）。扩展名由**检测到的格式**决定（png/jpg/jpeg/gif/webp/bmp/tiff），忽略 URL 原扩展名。
5. 文件名：取 URL 末段路径 stem，sanitize（仅 `[A-Za-z0-9._-]`，去查询串/非法字符），空则 `image`；扩展名用检测格式。并发写盘时用共享 `Mutex<HashSet>` + 现存文件检测去重（`name`、`name-1`、`name-2`…）。写入在任务内完成，写后即释放 `bytes`（限制内存）。
6. 结果按输入序排序后返回。

### D5：外部地址安全约束（SSRF 防护）

从用户机器直发任意 URL 有 SSRF 风险（探测内网、云元数据 169.254.169.254 等）。MVP 采取**尽力而为**的拒绝策略：

- scheme 必须是 `http` / `https`，否则拒绝（`data:`、`file:`、`ftp:` 等一律拒）。
- host 为 IP 字面量时拒绝：环回（127/8、::1）、私网（10/8、172.16/12、192.168/16）、链路本地（169.254/16、fe80::/10）、未指定（0.0.0.0、::）、ULA（fc00::/7）。
- **非标准 IP 写法也被阻断**：`reqwest::Url` 会把十进制（`2130706433`）、短写（`127.1`）、十六进制（`0x7f000001`）、`0`、IPv4-mapped IPv6（`[::ffff:7f00:1]`）等形式**归一化**为标准 IP 后再判段，故这些绕过手法对 `is_blocked_host` 无效（有回归测试 `blocks_noncanonical_ip_literals` 锁死）。
- host 为域名时拒绝 `localhost` 与 `*.local`。
- **重定向目标同样校验**：`reqwest` 默认跟随至多 10 次重定向，一个公网 URL 可被 `302` 引向内网目标，绕过上面的 host 校验。为此 client 设置 `redirect::Policy::custom`，对**每次重定向的目标 URL** 重新跑同一套 `validate_url`（scheme + 私网/环回/链路本地/ULA/localhost/`.local` 拒绝）；命中即 `attempt.stop()`（把 30x 响应当作终态返回，对该 URL 计入 `failed[]`），未命中才 `follow()`。保留对重定向的跟随，避免误伤依赖跳转的合法图片 CDN。
- **已知缺口（不修，记为 future）**：`validate_url` **不做 DNS 解析**，只校验 URL 里的 host 字符串。因此「公网域名 DNS 解析到私网/环回/云元数据 IP」（含 DNS rebinding）的**首次连接**无法在 MVP 内阻断。彻底修复需要引入 DNS resolver 依赖并对 resolved IP 复校验 + 连接时防 rebinding，超出 MVP 边界，与其它网络工具（`web_search`/`web_extract`）的约束口径一致。

被拒 URL（含重定向目标被拒）计入 `failed[]`。

### D6：与文档生成解耦的 system prompt 指示

`loop_support.rs` 组装 system prompt 时追加一行（无条件，不依赖任何 Key）：

```text
插入网络图片：先用 image_download(urls, dir) 下载到项目目录，再在 skill_run / typst / html 中按返回的本地相对路径引用（skill_run 运行时无法联网取图）。
```

不修改任何文档 skill 的职责；仅提示"先下载再按本地路径引用"。

### D7：沙箱、文件锁与 changed_paths

- 输出目录经 `resolve_for_write` 限定项目根内（`..`、越界拒绝），父目录自动 `create_dir_all`。
- **`dir` 规范化由共享 helper `normalize_output_dir` 统一处理**，`handler` 与 `io_plan` 共用，保证两侧一致：trim 后空/缺省 → `images`；反斜杠归 `/`、去尾 `/`、折叠冗余 `.` 段（`./.cache` → `.cache`，防 `./` 前缀绕过）；额外拒绝三类——绝对路径 / 含 `..` 越界、项目根（`.`，会锁住整个项目且图片散落根目录）、`.cache` 首段（该目录被 `changed_paths` 静默过滤，文件落盘但 UI/`@`/产物面板不可见，与"产物可见"语义冲突）。
- **符号链接防护**：写盘目标名若已存在（含断链/有效符号链接），MUST 被跳过——否则 `fs::write` 会沿符号链接写出项目外（沙箱逃逸）。`reserve_name` 用 `symlink_metadata`（不跟随）探测占用，`fetch_one` 用 `OpenOptions::create_new(true)` 写盘（与探测叠加堵 TOCTOU，且不覆盖既有文件/链接）。
- `io_plan`：`image_download` 对规范后的 `dir` 申请 `SubtreeWrite` 锁（与 `docx_extract_table out_dir`、`ooxml_unpack out_dir` 一致），保证同项目并发会话不会写花同一目录。
- `changed_paths`：读取 `result.downloaded[].path` 逐个上报。
- 外部 URL **不**做项目相对路径校验（它们是网络资源，不是沙箱路径）——这是与纯文件工具的关键区别，写进 `agent-loop` delta。
- **隐私已知项**：`urls` 原样进入 `tool_calls.args_json`，持久化到 SQLite 并回放给 provider（`openai_compat.rs`）。若 URL 内嵌凭证（`https://user:pass@host` 或 query token），可能长期留存。这与 `web_search`/`web_extract` 同口径，属全工具链既定行为，不在本 change 单独处理；工具 description 已引导使用普通公网图片地址。

### D8：异步执行（复用现有范式）

- `image_download` 的 `ToolSpec.handler` 返回 `NotImplemented` 占位；
- `ToolRegistry::execute` 增加 `"image_download" => crate::tools::image_download::handler(ctx, args).await`；
- handler 不需要 `AppHandle` / `model_id` / `secrets`，仅用 `ctx.sandbox`。

### D9：测试策略

- **纯函数单测**（不触网）：URL 校验与 SSRF 判定（各类内网/环回/域名）、文件名 sanitize 与去重、格式→扩展名映射、字节真实类型校验（用 `image` crate 生成的最小 PNG 字节 vs 一段 HTML 文本）。
- **入参校验单测**：空 `urls`、超 20、非 http(s) URL。
- **注册一致性**：`io_plan` 的 `all_default_tools_have_io_plan` / `io_plan_accepts_schema_required_args_only` 覆盖新工具（更新夹具）；`changed_paths` 新增解析测试。
- **前端**：`toolLabels.test.ts` 的 `EXPECTED_TOOLS` 同步。
- 真实网络下载走手动验证（不在 CI），与 `web_search` live test 同思路。

## 工具 I/O 与归属对照

| 工具 | 网络 | 沙箱输出 | changed_paths | 受 include_web 门禁 |
|---|---|---|---|---|
| `web_search` / `web_extract` | 是 | 否 | 空 | 是（需 Tavily Key） |
| `image_download` | 是 | 是（`dir`） | `downloaded[].path` | 否（无条件可用） |
| `skill_run` / `typst_to_pdf` / `html_to_pdf` | 否 | 是 | 产物路径 | 否 |

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| SSRF（探测内网/云元数据） | http(s) only + 私网/环回/链路本地/ULA 拒绝；**重定向目标也重新校验**（Policy::custom）；记录"无 DNS 重绑定防护"为未来项 |
| 大文件撑爆内存/磁盘 | 单图 15 MiB 流式上限 + 并发 5；写后即释放字节 |
| 非图片响应（错误页/重定向到 HTML） | 按字节真实类型校验，解析失败不落地，计入 failed |
| 文件名冲突/非法字符 | sanitize + 去重（`-1`/`-2`）；扩展名以真实格式为准 |
| 同项目并发写同目录 | `SubtreeWrite` 锁 |
| 模型仍想在 skill_run 里联网取图 | system prompt 明确"先 image_download 再引用本地路径"；运行时保持无 fetch |

## Migration Plan

- 纯增量：无 DB 迁移；旧会话行为不变；不读写历史目录。

## Open Questions

- （已决）独立工具、无条件可用、默认 `images/`。
- （已决）按真实字节校验类型、扩展名以检测格式为准。
- （已决）重定向目标经 `validate_url` 复校验（`Policy::custom`），堵住公网 URL 经 302 打内网的缺口。
- （未来）DNS 解析 + 重绑定防护；可选图片压缩/缩放；可选自定义文件名。
