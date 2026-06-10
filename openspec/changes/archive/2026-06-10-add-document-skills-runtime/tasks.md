# 实施任务：Document Skills 运行时

每组任务附「代码参考」，为实现骨架而非成品；实现与 design.md 冲突时先更新 artifact。

## 1. Spike 验证（裁决 design 中的开放决策点）

- [x] 1.1 spike：boa_engine + exceljs bundle ✅ 通过（IIFE 打包 + boa 0.21 `annex-b` 特性，根因为 `String.prototype.substr` 缺失；产物 xlsx 校验有效）
- [x] 1.2 spike：pptxgenjs 在 boa 下跑通 ✅（polyfill 清单：setTimeout/setImmediate/queueMicrotask/process.nextTick/crypto.getRandomValues/TextEncoder/TextDecoder/btoa/atob，已固化进运行时 HELPERS）
- [x] 1.3 spike：pdfsink-rs 表格提取（pdf-lib 生成的 PDF 经 text 策略提取并通过 data_query 验证；真实业务 PDF 留待实际使用反馈）
- [x] 1.4 spike：libxml 本期不引入 → 已锁定 D4 退路（XML well-formed + roundtrip），见 design.md

## 2. 依赖与构建基建

- [x] 2.1 Cargo 依赖：`boa_engine`、`polars`(lazy,sql,csv)、`ironcalc`、`pdfsink-rs`、`quick-xml`、`zip`、`base64`、`tempfile`（**注**：rustyscript 因 aes/serde 冲突未采用）
- [x] 2.2 `scripts/bundle-js-libs.mjs` + `npm run bundle:js` + `src-tauri/assets/js/*.bundle.js`
- [x] 2.3 XSD schemas → `src-tauri/assets/schemas/`（39 个 xsd，供后续 libxml 接入）
- [x] 2.4 `.github/workflows/ci.yml`：`bundle:js` + Rust cache

## 3. JS 脚本运行时（specs/script-runtime）

- [x] 3.1 `tools/runtime/ops.rs`：`__doc_read` / `__doc_write` / `__doc_log`（Sandbox 校验）
- [x] 3.2 `tools/runtime/mod.rs`：独立线程 + 超时 + 按需加载 bundle
- [x] 3.3 `skill_run` 实现（sync + async `main()`，Promise 经 `run_jobs` settle；32MB 栈线程）
- [x] 3.4 测试：`skill_run` 简单脚本、async main、exceljs 写 xlsx、`skill_read`、未知 skill 列表

## 4. Skill 知识库（specs/document-skills）

- [x] 4.1 `core/skills.rs`：编译期内置 + `index_markdown()` / `read()`
- [x] 4.2 `assets/skills/` + `scripts/prepare-skills.mjs`（命令映射 + 系统约束段）
- [x] 4.3 `skill_read` 工具
- [x] 4.4 `loop_runner.rs` system prompt 注入 skill 索引
- [x] 4.5 测试：四 skill 索引、无 `python scripts/` 残留、未知 skill 报错

## 5. OOXML 工具链（specs/ooxml-toolchain）

- [x] 5.1 `ooxml_unpack`（zip 解包；旧格式报错；smart quotes 基础处理）
- [x] 5.2 `ooxml_pack`（zip 回包；自动修复 MVP no-op）
- [x] 5.3 `validate.rs`（well-formed XML + roundtrip；**非 XSD 全量**）
- [x] 5.4 `docx_comment`（comments.xml MVP）
- [x] 5.5 `docx_accept_changes`（纯 XML 变换）
- [x] 5.6 测试：unpack→pack roundtrip 已通过（根因：zip 目录条目尾随 `/` 被 `fs::write` 写入导致 os error 2，现跳过目录条目）

## 6. 数据分析管道（specs/data-analysis）

- [x] 6.1 `docx_extract_table`（quick-xml `w:tbl`）
- [x] 6.2 `pdf_extract_table`（pdfsink-rs）
- [x] 6.3 `data_query`（polars-sql + calamine 桥接 xlsx）
- [x] 6.4 `xlsx_recalc`（IronCalc；umya 产物可能触发 catch 降级 warning）
- [x] 6.5 测试：PDF 生成→提取→data_query 端到端（`smoke_pdf_data_pipeline`）

## 7. 注册、文案与收尾

- [x] 7.1 `registry.rs` 注册全部 10 个新工具
- [x] 7.2 skill 文档追加系统约束段（prepare-skills.mjs）
- [x] 7.3 `MAX_TOOL_STEPS` 8 → 16
- [x] 7.4 端到端冒烟三链路（registry 级）：PPT 创建（pptxgenjs）、修订批注（unpack→comment→pack）、PDF 数据管道（pdf-lib→extract→query）
- [x] 7.5 README 能力说明；`cargo test` 40 项通过；`clippy -D warnings` 通过（含 build.rs）；`cargo fmt --check` 通过；前端 `typecheck`（lib 升 ES2022）与 `npm test` 通过
