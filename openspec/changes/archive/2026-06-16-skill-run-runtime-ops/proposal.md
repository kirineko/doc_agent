## Why

`skill_run` 是 docx/pptx/xlsx/pdf 生成与 OOXML 批量编辑的核心执行面，但 **Agent 成功率被「文档说一套、运行时做一套」持续拖累**：

1. **OpenSpec 漂移**：`script-runtime/spec.md` 仍写「deno_core / V8」「JavaScript/TypeScript」「可直接 `import`」，实现已是 **boa_engine 0.21** + 全局变量 / `require()` 白名单（见 `tools/runtime/mod.rs`）。Spec 中的 Scenario 会诱导模型写出无法执行的 `import PptxGenJS from "pptxgenjs"`。
2. **缺少单一运行时参考**：system prompt 要求「不得凭记忆直接编写 skill_run 代码」，但 `index_markdown()` 只列 format skills（docx/pdf/…），**没有** `skill_read runtime`。各 SKILL.md 教格式与示例，polyfill / 限制分散在代码注释里。
3. **Native API 缺口**：仅有 `doc_read` / `doc_write` / `doc_log`（`ops.rs`）。OOXML 模板流程虽以固定路径为主（`unpacked/word/document.xml`），但列 slide 文件、检查路径是否存在等场景 Agent 仍会试探 `fs.readdirSync` / `existsSync`——当前不存在，只能硬编码或失败重试。
4. **Bundle 误加载（有实测成本）**：`bundles_for_code` 中 `lower.contains("pptx")` 会在脚本仅含 `"output.pptx"` 等路径字符串时加载 **~374KB** pptxgenjs bundle（OOXML 纯 `fs` 编辑脚本常见）。四 bundle 合计 ~2.1MB，误加载直接吃掉 30s 超时预算的开头秒数。
5. **已有能力未对外说明**：`normalize.rs` 已处理 `require()` 别名、`main()` 剥离、无 main 时自动包裹——Agent 不知道，仍会重复犯错。

Stop turn 已落地；下一优先级应是 **让 Agent 写对脚本、少踩运行时坑**，而非继续加 format 功能。

## 调研结论（Problem Analysis）

| 失败模式 | 根因 | 代码/文档证据 | 本 change 对策 |
|----------|------|---------------|----------------|
| `import … from "pptxgenjs"` 语法/模块错误 | Spec Scenario + bundle 为 IIFE 全局，非 ESM | `script-runtime/spec.md` L20–21；bundle 无 export | 修正 spec；`normalize` 将常见 import 改写成全局；runtime 文档明确禁止裸 import |
| 脚本用了 `fs.readdirSync` / `existsSync` | fs shim 仅有 read/write | `mod.rs` HELPERS L117–138 | 新增 doc_list/doc_exists + fs 映射 |
| OOXML 编辑 turn 加载 pptxgenjs | 路径字符串含 `.pptx` 触发启发式 | `bundles_for_code` L362 `lower.contains("pptx")` | **仅收紧 pptxgenjs** 触发条件 |
| Agent 不知道用全局还是 require | 文档分散；normalize 行为未公开 | `normalize.rs`；skill 示例混用 | runtime SKILL.md 能力矩阵 + system prompt / index 入口 |
| 误读运行时引擎 | Spec 写 V8 | 同上 | spec Purpose + 引擎描述全面修正 |
| docx bundle 误加载 | 理论上有 `.docx` 字符串风险 | fs-only 编辑示例（`docx/editing.md`）**通常不含** `docx` 库关键字；现有测试 `skill_run_fs_edits_unpacked_docx_xml` 不触发 docx bundle | **本 change 不收紧 docx 规则**（避免漏加载 docx-js）；仅文档说明 |

**刻意不做**：`doc_list` 与 `@` 索引共用「隐藏 unpacked 子树」规则——模板编辑需要 `list("unpacked/ppt/slides")` 列 slide 文件；`@` 索引隐藏 unpacked 是为避免误引用，语义不同（见 `project_files.rs` `should_skip_entry` vs `list_project_dir`）。

## What Changes

### P0 — Spec 真相 + Agent 可发现性（最高 ROI）

- **修正** `script-runtime` spec：引擎 → boa_engine；语言 → JavaScript only；库加载 → 全局 / `require()`；更新 Scenario；补全 Purpose（现为 TBD）
- **新增** 内置 `skill_read {"skill":"runtime"}` + `SKILL.md` 能力矩阵（API 表、polyfill 清单、normalize 行为、限制、示例）
- **更新** `index_markdown()`、`skill_read` / `skill_run` 工具描述、system prompt 一句：编写/修复 skill_run 前先 `skill_read runtime`
- **新增** `normalize.rs` 对常见 `import … from 'lib'` 的兼容改写（与现有 `require` 改写同层，**不是**实现 ES module）

### P1 — Native op + fs 扩展

- **新增** `__doc_exists` / `doc_exists`、`__doc_list` / `doc_list`（Sandbox 校验；list 复用 `list_project_dir` 单层语义）
- **新增** `fs.existsSync`、`fs.readdirSync`（映射到上述 op；`readdirSync` 仅返回名称，类型信息用 `doc_list`）

### P1 — Bundle 启发式

- **收紧** pptxgenjs：移除裸子串 `pptx`；改为 `pptxgenjs`、`PptxGenJS`、`new PptxGenJS` 等库用法信号
- **维持** docx / exceljs / pdf-lib 现有规则（exceljs 已较精确；docx 收紧风险大于收益）

### P2 — 诊断与 SKILL 交叉引用

- **更新** `with_runtime_hint` / `build_script_error` hint：指向 `skill_read runtime`；模块未找到时列出白名单
- **小幅更新** docx/pptx/xlsx/pdf SKILL.md：skill_run 段落增加 runtime 文档链接（不重写 format 指引）

## Capabilities

### New Capabilities

（无 — 能力归入既有 `script-runtime`）

### Modified Capabilities

- `script-runtime`：Native 目录/存在 op；import 兼容 normalize；spec 引擎/模块语义修正；runtime 能力矩阵文档；pptxgenjs bundle 启发式；Agent 可发现性要求

## Impact

- **后端**：`tools/runtime/ops.rs`、`mod.rs`、`normalize.rs`；`core/skills.rs`；`core/project_files.rs`（list 复用）；`agent/loop_support.rs`（system prompt）；`tools/skill.rs`
- **文档**：`assets/skills/runtime/SKILL.md`（新）；format SKILL.md 最小 diff
- **Spec**：`openspec/specs/script-runtime/spec.md`（归档合并）
- **测试**：runtime op 单测；pptx 路径不误加载 bundle；import normalize；`index_markdown` 含 runtime
- **依赖**：无新 crate

## 成功标准

- Agent 按 **旧 spec** 生成 `import PptxGenJS …` 时，normalize 后可执行或给出明确「用全局/require」提示
- 仅含 `"output.pptx"` 路径、无 PptxGenJS 用法的 fs 脚本 **不** 加载 pptxgenjs bundle
- `doc_list("unpacked/ppt/slides")` / `doc_exists("template.docx")` 在沙箱内可用且有测试
- `script-runtime` spec 全文无 V8/TypeScript/裸 import 表述
- system prompt + skill 索引可发现 `skill_read runtime`

## 非目标（本 change 不做）

- TypeScript 转译、完整 ES module 支持、dynamic import
- 真实 `setTimeout` 延迟、fetch、shell、npm 任意包
- boa heap 上限、skill_run 硬中断（属 stop-turn 后续）
- docx bundle 启发式收紧（单独评估）
- 让 `doc_list` 与 `@` 索引忽略规则完全一致
