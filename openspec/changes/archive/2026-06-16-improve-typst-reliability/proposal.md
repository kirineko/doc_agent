## Why

Agent 调用 Typst 导出经常失败且难以恢复：编译错误以 `{:?}` Debug 形式回传，span 不可读、无文件/行列/源码片段，warnings 还被直接丢弃；Agent 因而只能猜错误位置，常常推倒重写而非局部修补。同时语法手册与模板可能与真实内置 API 漂移，模板美学/字体/语法亦未系统审查。本变更聚焦把「错误可读 + 引导局部修改 + 手册/模板正确可信」一次性补齐。

## What Changes

- **结构化编译诊断**：`typst_to_pdf` 失败时返回结构化错误（`error_type`、`file`、`line`、`column`、出错源码片段 `snippet`、`message`、`hints`），由 Typst `Span` 还原为人类可读位置。
- **暴露编译警告**：成功与失败时均把 warnings（如字体回退、弃用语法）随结果回传给 Agent，不再仅 `eprintln!` 到 stderr。
- **引导局部修改**：在结构化错误中附 `fix_guidance`，并在工具描述中明确「编译失败应优先 `fs_patch` 做最小修改，禁止整篇重写」。
- **手册校验与修订**：新增测试抽取 `syntax/typst-guide.md` 中**显式标注为可独立编译**的 Typst 代码块真实编译；新增 `common/*.typ` 公开符号与手册「内置模块」表的一致性校验；据校验结果修订手册错误/补缺。
- **建立可主题化的 Typst 设计系统（`common/tokens.typ`）**：以设计 token（字号阶、间距阶、行距、线宽、页边距、字体角色）作为唯一真相统一版式；颜色与观感通过「调色板预设 + `make-theme(...)` 受控覆盖」开放给 Agent，锁定可读性轴、放开强调色/密度/标题风格等自由轴，既统一又不千篇一律。
- **重构模板消费 token**：4 个 common 模块 + 8 个场景模板改为从 `tokens.typ` 取值，消除硬编码魔数（散落的 `16pt`/`0.65em`/`0.75pt` 等），并保证全部零警告编译、无弃用 API。

不纳入（用户明确排除）：调用效果评估 harness（问题 5）、PNG 渲染/视觉回归基建。美学以「token 唯一真相 + 零警告编译」为客观验收，不引入主观人工评审环节。

## Capabilities

### New Capabilities
<!-- 无新增能力，均为对现有 typst-export 能力的需求增强 -->

### Modified Capabilities
- `typst-export`: 新增「编译诊断须结构化且含可定位信息与警告」「失败须引导局部修改」「内置手册示例与公开 API 一致性可验证」「模板须通过美学与零警告语法审查」等需求。

## Impact

- 代码：`src-tauri/src/tools/typst_export/compile.rs`（诊断格式化、span 还原、warnings 收集）、`src-tauri/src/tools/typst_export/mod.rs`（结构化错误回传、工具描述）、`src-tauri/src/tools/registry.rs`（如需 `ToolError::Structured` 承载结构化诊断）。
- 资源：`src-tauri/assets/typst-templates/**`——新增 `common/tokens.typ`；重构 `common/{fonts,page,exam,lecture}.typ` 与 8 个场景模板消费 token；修订 `syntax/typst-guide.md`。`bundled.rs` 须挂载新增的 `tokens.typ` 虚拟路径。
- 测试：`src-tauri/src/tools/typst_export/compile.rs` 内联测试、`tools/tests.rs`（手册代码块编译、exports 一致性、模板零警告）。
- 依赖：复用既有 `typst`/`typst-as-lib`/`typst-syntax` 0.13/0.14，不新增 crate。
