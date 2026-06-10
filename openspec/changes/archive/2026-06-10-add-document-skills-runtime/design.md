# 设计：Document Skills 运行时

## Context

- bootstrap MVP 已具备：ToolRegistry（schema 化工具注册 + 沙箱执行）、`Sandbox`（canonicalize + 项目根前缀校验）、Agent Loop（多轮工具调用）、`skill_run` 占位（`tools/skill.rs`，`NotImplemented`）。
- `doc_skills/` 收录了 docx / pdf / pptx / xlsx 四个 skill（Anthropic 文档技能），其结构为四层：①知识层（SKILL.md 等纯文本）②脚本层（unpack/pack/validate 等 Python 胶水）③库层（docx-js / pptxgenjs / openpyxl 等创建库）④重型二进制层（LibreOffice / pandoc / Poppler）。
- 约束：不改变现有架构（依赖方向 `ipc → agent → tools/core` 不变）；不依赖用户机器上的 Python / Node / LibreOffice；产品定位轻量桌面应用，体积增量需有明确账目。

## Goals / Non-Goals

**Goals:**

- 四个 skill 的知识层原文收录为内置 skill 仓库，agent 经渐进披露按需读取。
- 脚本层语义全部 Rust 化（OOXML 解包 / 回包 / 校验 / 批注 / 接受修订）。
- 库层经嵌入式 JS 运行时承载（docx-js / pptxgenjs / exceljs / pdf-lib），解锁 PPT 生成。
- 重型二进制层全部替代或显式降级（IronCalc 重算、XSD + roundtrip 校验、mammoth 预览）。
- 数据管道：Word / PDF 表格提取 → polars SQL 整理 → 美观表格输出。

**Non-Goals:**

- 不做扫描件 OCR、不做 .doc/.ppt 旧格式编辑（仅读取）、不做 pptx 视觉缩略图。
- 不做 skill 分层仓库（项目级 / 用户级），本期仅内置层。
- 不引入通用 shell 执行工具；JS 是唯一脚本面，且无网络、文件访问全部经 Sandbox。

## 架构总览

```
┌─ system prompt ───────────────────────────────────────────────┐
│ 现有内容 + skill 索引（name + description，来自内置仓库）        │
└──────────────┬────────────────────────────────────────────────┘
               ▼ agent 需要时
   skill_read("docx") ──▶ core/skills: 内置仓库（编译期 include）
               ▼ 按 skill 指引选择执行路径
 ┌─ 编辑现有文档（Rust 原生）─────┐  ┌─ 创建新文档（JS 运行时）──────┐
 │ ooxml_unpack → fs_read/write  │  │ skill_run(code)               │
 │ → ooxml_pack(XSD+roundtrip)   │  │  └ rustyscript(deno_core/V8)  │
 │ docx_comment / accept_changes │  │    内置 bundle: docx/pptxgenjs │
 └───────────────────────────────┘  │    /exceljs/pdf-lib           │
 ┌─ 数据管道（Rust 原生）─────────┐  │    op_doc_read/write→Sandbox  │
 │ docx_extract_table (quick-xml)│  │    无网络、超时熔断            │
 │ pdf_extract_table (pdfsink-rs)│  └───────────────────────────────┘
 │ data_query (polars-sql)       │  ┌─ 产物验证（Rust 原生）────────┐
 │ 中间格式: 沙箱内 CSV/JSON      │  │ xlsx_recalc (IronCalc)        │
 └───────────────────────────────┘  │ ooxml_pack 内置 XSD 校验      │
                                    └───────────────────────────────┘
 ToolRegistry / Sandbox / Agent Loop / ipc / 前端: 零改动
```

## 模块划分（遵循体量规则）

```
src-tauri/src/
  core/
    skills.rs            # 内置 skill 仓库: 索引 + 全文读取（≤150 行）
  tools/
    skill.rs             # 改写: skill_read + skill_run 的 ToolSpec 定义
    runtime/
      mod.rs             # JsRuntime 封装: 创建/执行/超时（≤200 行）
      ops.rs             # 自定义 op: 沙箱文件读写、日志（≤150 行）
    ooxml/
      mod.rs             # ooxml_unpack / ooxml_pack ToolSpec
      unpack.rs          # zip 解包 + pretty-print + merge runs
      pack.rs            # condense + 自动修复 + 调用 validate
      validate.rs        # XSD(libxml) + roundtrip 校验
      comment.rs         # docx_comment（移植 comment.py）
      redline.rs         # docx_accept_changes（接受修订 XML 变换）
    data/
      mod.rs             # ToolSpec 定义
      extract_docx.rs    # w:tbl 表格提取 → CSV
      extract_pdf.rs     # pdfsink-rs 表格提取 → CSV
      query.rs           # polars-sql data_query
      recalc.rs          # IronCalc xlsx_recalc
src-tauri/assets/
  skills/                # 内置 skill 文档（原文 + 命令映射改写）
  js/                    # esbuild 产物: docx.js pptxgenjs.js exceljs.js pdf-lib.js
  schemas/               # ISO-IEC 29500 XSD（自 doc_skills 复制）
scripts/
  bundle-js-libs.mjs     # esbuild 预打包脚本（npm run bundle:js）
```

## Decisions

### D1：JS 引擎选 boa_engine（实现期调整，原方案 rustyscript/V8）

- **原方案**：rustyscript（deno_core / V8）——完整 ES + TS，用户要求「功能全面」。
- **实现期调整**：rustyscript 与 `umya-spreadsheet` 在 `aes` crate 版本上冲突；降级 `default-features = false` 后仍触发 `swc_common` 与 `serde` 不兼容。**改用 boa_engine（纯 Rust，Apache-2.0）** 作为 `skill_run` 运行时。
- **当前能力**（boa 0.21 + `annex-b,js` 特性）：sync / async `function main()`（Promise 经 `run_jobs` settle）；bundle 为 esbuild IIFE 格式按需加载（代码含 `exceljs`/`docx`/`pptx`/`pptxgenjs`/`pdf-lib` 关键字时注入）；`doc_read`/`doc_write`/`doc_log` 经原生函数走 Sandbox；运行线程 32MB 栈（boa 解析大 bundle 递归深）。
- **polyfill 清单**（已固化进运行时 HELPERS）：`setTimeout`/`setImmediate`/`clearTimeout`/`clearImmediate`/`queueMicrotask`/`console`(→doc_log)/`process.nextTick`/`crypto.getRandomValues`/`TextEncoder`/`TextDecoder`/`btoa`/`atob`/`self`。
- **spike 结论**：exceljs / pptxgenjs / pdf-lib 在 boa 下均跑通（产出有效 xlsx/pptx/pdf）；exceljs 早期加载失败根因是 boa 默认不带 Annex B 的 `String.prototype.substr`，启用 `annex-b` 特性解决。
- **备选保留**：解决依赖冲突后回切 rustyscript/V8；或 Node sidecar（体积与分发成本更高）。
- **2026-06 复验**：rustyscript 0.12.3（最新）仍不可用——① 默认特性下 `deno_crypto` 钉死 `aes =0.8.3`，与 `umya-spreadsheet` 的 `aes ^0.8.4` 硬冲突；② `no-default-features` 可解析但 `deno_ast =0.49.0` 锁 `swc_common ^9`（最高 9.2.0），其 `serde::__private` 用法与 serde ≥1.0.220 编译不兼容，而 `docx-rs`（serde_json ≥1.0.142）与 `indexmap 2.14` 强制新 serde，钉旧版会级联冲突。需等 rustyscript 升级 deno_ast ≥0.50（swc_common 14+ 已修复）。**维持 boa_engine。**

### D2：分析用 polars（+ polars-sql），暴露 SQL 接口而非 DataFrame API

- **理由**：给 agent 暴露一个 `data_query(sources, sql)` 工具，模型写 SQL 完成清洗 / 聚合 / 透视 / 连接，比映射几十个 DataFrame 方法的工具集对模型友好、schema 稳定。xlsx 读取由现有 `calamine` 桥接为 DataFrame（polars Rust 端不原生读 xlsx）。
- **feature 裁剪**：仅启 `lazy`、`sql`、`csv`、`dtype-slim` 类最小集合，控制体积（预估 +15~25MB）。
- **备选**：datafusion（同为 SQL 引擎，但生态对 Excel 场景的 dtype 处理不如 polars 顺手）；自写聚合逻辑（长尾场景无法覆盖），均否决。

### D3：PDF 表格提取选 pdfsink-rs，Word 表格用 quick-xml 自研

- **理由**：pdfsink-rs 纯 Rust、明确对标 pdfplumber（原 pdf skill 所用库），支持 `lines / lines_strict / text / explicit` 四种策略，零外部依赖。Word 侧 `w:tbl/w:tr/w:tc` 为确定性结构，quick-xml 遍历即可，可靠性高于任何启发式。
- **备选**：ripdoc（Tabula 算法，作为 spike 对比项保留）、trex（可选 ONNX 路由，重）、pdfium-render（仅文本坐标无表格重建）。spike（任务 1.3）以真实业务 PDF 验收提取质量后锁定。

### D4：稳定性校验 = XSD（libxml2）+ 自动修复 + roundtrip 三重闭环

- **理由**：用户点名「确保 XML 修改和生成的稳定性」。原 skill 的 validate.py 用 lxml 对 ISO-IEC 29500 XSD 全量校验，schema 文件已在 `doc_skills/docx/scripts/office/schemas/`，直接复用。
- **实现期裁决（任务 1.4）**：本期未引入 `libxml`（spike 未跑 Windows MSVC）。**采用退路方案**：well-formed XML 解析校验 + `[Content_Types].xml` 存在性检查 + roundtrip（xlsx 用 calamine 重开）。XSD 全量校验列为后续增强。
- **自动修复**：MVP 阶段为 no-op；durableId / `xml:space` 自动修复待补。
- **roundtrip**：pack 产物 zip 结构校验 + xlsx calamine 解析。

### D5：xlsx 公式重算用 IronCalc 替代 LibreOffice recalc

- **理由**：用户要求不用 LibreOffice。IronCalc（Apache-2.0/MIT，v0.7.1，活跃维护）纯 Rust、可读 xlsx 并 `evaluate()`，重算后扫描 `#REF! / #DIV/0! / #VALUE! / #N/A / #NAME?`，与原 xlsx skill「零公式错误」验收语义一致。
- **已知限制**：数组公式暂不支持（其路线图最高优先级）→ skill 文案中注明避免生成数组公式；IronCalc 不认识的函数报 `#NAME?` 时按警告而非硬错误处理（白名单见 tasks）。
- **备选**：HyperFormula（JS，GPLv3 商用收费）许可不利，否决。

### D6：skill 文档「原文收录 + 最小命令映射改写」

- **理由**：用户要求尽量遵循原文。仅替换执行命令段（映射表见 tasks 4.2），知识性内容一字不动。文件经编译期 `include_str!` 内置，避免运行时路径问题。
- **许可证边界**：仅内部使用；对外分发前需法务评估（proposal 已记录）。

### D7：降级矩阵（无 LibreOffice / pandoc / Poppler）

| 原能力 | 处置 |
|---|---|
| OOXML 校验 | XSD + roundtrip（D4），等效或更严 |
| xlsx 公式重算 | IronCalc（D5） |
| 接受修订 accept_changes.py | Rust 纯 XML 变换（应用 `w:ins` 内容、移除 `w:del`、处理 `w:pPr/w:rPr/w:del` 段落标记） |
| docx → PDF/图片视觉校验 | 降级：mammoth.js（docx→HTML，JS 运行时内）近似预览 + 结构校验兜底 |
| pptx 缩略图 thumbnail.py | 降级：XML 结构校验 + `office_read_markdown` 逆向抽文本自检，skill 文案引导用户用系统应用人工确认 |
| .doc/.ppt → 新格式转换 | 降级：读取走现有 office_oxide；编辑场景工具返回明确错误提示「请先另存为 .docx」 |
| pandoc 文本提取 | 已有 `office_read_markdown` 等价覆盖 |

## 关键接口骨架（代码参考索引）

新增工具一览（全部经 `ToolRegistry::default_tools()` 注册，handler 签名不变 `fn(&ToolContext, Value) -> Result<Value, ToolError>`）：

| 工具名 | 模块 | 一句话职责 |
|---|---|---|
| `skill_read` | `tools/skill.rs` | 读取内置 skill 全文（name → 文档内容） |
| `skill_run` | `tools/skill.rs` + `tools/runtime/` | 在嵌入式 JS 运行时执行代码，产物经沙箱落盘 |
| `ooxml_unpack` | `tools/ooxml/unpack.rs` | docx/pptx/xlsx → 解包目录（美化 + merge runs） |
| `ooxml_pack` | `tools/ooxml/pack.rs` | 解包目录 → 文档（修复 + XSD + roundtrip） |
| `docx_comment` | `tools/ooxml/comment.rs` | 向解包目录注入批注（移植 comment.py） |
| `docx_accept_changes` | `tools/ooxml/redline.rs` | 接受全部修订生成干净文档 |
| `docx_extract_table` | `tools/data/extract_docx.rs` | 提取 Word 表格 → CSV |
| `pdf_extract_table` | `tools/data/extract_pdf.rs` | 提取 PDF 表格 → CSV |
| `data_query` | `tools/data/query.rs` | 对 CSV/xlsx 源执行 polars SQL → CSV/JSON |
| `xlsx_recalc` | `tools/data/recalc.rs` | IronCalc 重算并报告公式错误 |

`ToolContext` 唯一扩展点：`skill_run` 需要持有 JS 运行时句柄。为避免改动既有 handler 签名，运行时以 `OnceLock`/`thread_local` 方式在 `tools/runtime/mod.rs` 内自管理（每次调用新建短生命周期 Runtime，天然隔离、无状态泄漏），`ToolContext` 不变。

各模块详细代码参考随实施步骤写在 `tasks.md` 对应任务下。

## Risks / Trade-offs

- [V8 体积 ~35MB + polars ~20MB，总增量 ~70MB] → 与「体积尽量小」冲突，proposal 已记账并确认取舍；polars feature 裁剪 + release 构建 `opt-level="z"` + strip 缓解。
- [JS 库在无 DOM / 无 Node 环境下的兼容性未知] → 阶段 1 spike 先行（pptxgenjs 最高风险，其依赖 JSZip）；不通过则该库退到 OOXML 模板路线（unpack 模板 → 改 XML → pack）。
- [pdfsink-rs 对复杂版式 PDF 提取质量不确定] → spike 用真实业务 PDF 验收；不达标依次试 ripdoc / trex；最终兜底为「文本 + 坐标导出 → agent 自行重建」。
- [libxml2 在 Windows 的构建 / 分发成本] → spike 裁决；退路为针对性规则校验 + roundtrip（D4 备选）。
- [IronCalc 函数覆盖不全造成误报] → `#NAME?` 降级为 warning；skill 文案约束模型使用常用函数集。
- [skill_run 给模型任意代码执行能力] → 运行时无网络扩展、文件 op 全部经 Sandbox 校验、执行超时（默认 30s）+ 内存上限；与现有工具同等审计（tool_calls 持久化）。
- [V8 (deno_core) 编译时间显著拉长 CI] → CI 缓存 sccache / cargo cache；可接受。

## Open Questions

- pptxgenjs 在 rustyscript 下需要哪些 polyfill（spike 输出物：polyfill 清单或否决结论）。
- libxml2 路线在 Windows CI 上的可行性（spike 裁决 D4 主路 / 退路）。
- `data_query` 的多源 JOIN 是否需要在 MVP 支持（先支持单源 + 多源 UNION，JOIN 视任务 5 实测决定）。
