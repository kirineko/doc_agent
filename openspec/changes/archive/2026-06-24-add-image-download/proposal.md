# 提案：Agent 图片下载能力（add-image-download）

## Why

用户用 doc-agent 制作 Word / PPT / PDF 时常需插入网络图片，但当前系统没有把图片下载到本地的能力：

- `skill_run`（boa 运行时）**无 fetch / 无网络**，docx-js / pptxgenjs / PDFLib 只能引用**本地文件**；
- `typst_to_pdf`、`html_to_pdf` 同样只读取沙箱内文件；
- `web_extract` 只抽取网页正文（文本），不保存二进制图片。

结果是：模型拿到图片 URL 也无法落地为可被文档引用的本地文件，图文文档制作受阻。

需要一个**独立**的工具，把一批图片 URL 下载到项目目录，返回本地相对路径，供后续文档生成按本地路径引用——并与具体文档格式（Word/PPT/PDF）**解耦**：下载只负责"取图落地"，文档生成只负责"按本地路径引用"。

## What Changes

- 新增 1 个 Agent 工具 `image_download`：批量下载 1–20 个公网 http(s) 图片到项目沙箱内目录（默认 `images/`），返回每张图的本地相对路径与元数据（尺寸、字节数、格式），逐条返回成功/失败。
- **始终可用**：不依赖任何 API Key，不受 Web 搜索开关（`include_web`）影响（工具名不以 `web_` 开头）。
- 复用已有异步工具机制（stub handler + `execute` 异步特判，同 `web_search` / `html_to_pdf`）。
- 与文档生成解耦：`image_download` 不感知 docx/pptx/pdf；文档侧仍只引用本地路径。system prompt 增加一条简短指示——插入网络图片须先 `image_download` 落地、再在 `skill_run` / typst / html 中按返回的本地路径引用。
- 安全与健壮：仅 http/https；SSRF 防护（拒环回/私网/链路本地地址）；单图体量上限；并发与超时限制；按真实字节校验图片类型（防把 HTML 错误页当图片保存）；批量部分失败不影响其余成功项。
- 沙箱与治理：输出目录经 `resolve_for_write` 限定在项目根内，`ToolIoPlan` 对输出目录申请 `SubtreeWrite` 锁；下载成功的文件进入 `tool_result.changed_paths`，出现在文件浏览 / `@` 候选 / 产物面板。
- 前端：`toolLabels` 增加 `image_download → 下载图片` 中文标签并同步测试。
- 复用现有依赖 `reqwest`（已含 `stream` + `rustls-tls`）、`image`、`futures-util`，**不新增依赖**。

## Capabilities

### New Capabilities

- `image-download`：`image_download` 工具契约、批量下载、类型/体量校验、外部地址安全约束、部分失败处理、与文档生成解耦、无条件可用。

### Modified Capabilities

- `agent-loop`：澄清"沙箱内执行"对 `image_download` 这类**混合工具**（外部 URL 输入不做路径校验，但文件输出限定沙箱内且 `changed_paths` 非空）的适用方式；明确该工具无条件注册（不依赖 Tavily Key）。
- `workspace-ui`：`image_download` 中文工具链标签。

## Impact

- **Rust**：新增 `src-tauri/src/tools/image_download.rs`；改 `tools/mod.rs`、`registry.rs`（注册 + `execute` 异步分支）、`io_plan.rs`（`SubtreeWrite` 锁 + 测试夹具）、`changed_paths.rs`（上报下载文件 + 测试）、`agent/loop_support.rs`（system prompt 一行指示）。无新增 Cargo 依赖。
- **前端**：`src/lib/toolLabels.ts` + `toolLabels.test.ts` 同步新增标签与 `EXPECTED_TOOLS`。
- **风险**：从用户机器直发任意 URL 存在 SSRF 风险 → 以 http(s) + 私网/环回地址拒绝缓解；下载大文件占内存/磁盘 → 单图字节上限 + 并发上限 + 流式累计字节中断；非图片响应 → 按字节真实类型校验后才落地。
