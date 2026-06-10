# 提案：Document Skills 运行时（add-document-skills-runtime）

## Why

MVP 已跑通「Agent + 文件系统 + Word / Excel 基础读写」，但在 Word / Excel / PPT / PDF 的深度处理上能力不足：无 PPT 生成、无修订与批注、无公式与样式化表格、无 PDF 表格提取。`design.md`（bootstrap）第 7 节预留的 Document Skill 机制（`skill_run` 工具 + 脚本执行器）至今是 `NotImplemented` 占位。

经探索调研（参见会话结论），确定以 `doc_skills/`（docx / pdf / pptx / xlsx 四个 skill）为蓝本，在**不改变现有架构**（ToolRegistry / Sandbox / Agent Loop / ipc 均不动）的前提下，将其落地为内部能力：知识层原文收录 + 执行层映射为 Rust 原生工具与嵌入式 JS 运行时，**全程不依赖 Python / Node / LibreOffice / pandoc 等外部环境**。

## What Changes

- **内置 Skill 知识库**：收录 `doc_skills/` 四个 skill 的文档（SKILL.md、editing.md、forms.md、pptxgenjs.md 等），仅将命令行段落改写为本系统工具调用说明，其余内容遵循原文；skill 索引（name + description）注入 system prompt，新增 `skill_read` 工具按需渐进披露全文。
- **嵌入式 JS 脚本运行时**：`rustyscript`（deno_core / V8）实现 `skill_run` 工具；内置 `docx`（docx-js）、`pptxgenjs`、`exceljs`、`pdf-lib` 四个库的预打包 bundle；文件读写经自定义 op 走现有 `Sandbox`，禁网络。**PPT 生成能力由此解锁**。
- **OOXML 工具链（Rust 原生）**：`ooxml_unpack` / `ooxml_pack` 工具移植原 skill 的 unpack.py / pack.py / validate.py 语义（解包美化、合并 run、回包压缩、自动修复、XSD 校验 + roundtrip 校验）；`docx_comment`（批注）与 `docx_accept_changes`（接受修订，纯 XML 变换替代 LibreOffice）。
- **数据分析管道**：`pdf_extract_table`（pdfsink-rs，pdfplumber 同款策略）、`docx_extract_table`（quick-xml 解析 `w:tbl`）、`data_query`（polars + polars-sql，SQL 接口做清洗 / 聚合 / 透视）；中间产物为沙箱内 CSV/JSON 文件。
- **无 LibreOffice 验证体系**：xlsx 公式重算校验用 IronCalc（`xlsx_recalc` 工具，扫描 #REF! 等公式错误）；OOXML 用 XSD + roundtrip；docx 预览降级为 mammoth.js（docx→HTML）；.doc 旧格式编辑与 pptx 缩略图明确降级（提示用户）。

## Capabilities

### New Capabilities

- `document-skills`：内置 skill 知识库的收录、索引注入与 `skill_read` 渐进披露。
- `script-runtime`：嵌入式 JS 运行时（`skill_run`）、内置库 bundle、沙箱化文件访问与执行限制。
- `ooxml-toolchain`：OOXML 解包 / 回包 / 校验 / 自动修复工具链，及批注、接受修订等 XML 变换工具。
- `data-analysis`：Word / PDF 表格提取、polars SQL 数据整理、xlsx 公式重算校验，端到端「提取 → 整理 → 美观输出」管道。

### Modified Capabilities

- `office-tools`：移除「PPT 生成排除在 MVP 之外」的限制性 requirement——PPT 生成 / 编辑改经 `script-runtime`（pptxgenjs）与 `ooxml-toolchain` 提供。

## Impact

- **代码**：`src-tauri/src/tools/`（新增 `ooxml/`、`data/`、`runtime/` 等模块，改写 `skill.rs`）、`src-tauri/src/core/`（skill 仓库加载）、`registry.rs`（注册新工具）、agent system prompt 组装处（注入 skill 索引）。前端零改动（工具卡片自动展示新工具）。
- **新增 Rust 依赖**：`rustyscript`（deno_core/V8）、`polars`（裁剪 feature：lazy/sql/csv）、`libxml`（XSD 校验）、`pdfsink-rs`（PDF 表格）、`ironcalc`（公式重算）、`quick-xml`、`zip`。各依赖选型理由见 design.md。
- **构建产物**：体积增量约 +70MB（V8 ~35MB、polars ~20MB、其余 ~15MB），是「零用户环境依赖」的明确取舍；JS 库 bundle 需新增构建步骤（esbuild 预打包，产物进 `src-tauri/assets/`）。
- **许可证**：`doc_skills/` 原文为 Anthropic 专有许可，本变更仅限内部使用场景收录；对外分发前需另行评估（风险已在探索阶段向决策人说明，决策为「尽量遵循原文」）。
- **排除（非本变更范围）**：扫描件 PDF 的 OCR、.doc/.ppt 旧格式的编辑（仅读取）、pptx 视觉缩略图、实时预览渲染、skill 的项目级 / 用户级分层仓库（仅内置层）。
