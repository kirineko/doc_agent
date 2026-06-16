## Why

中文 Typst 模板在 `apply-zh-body` 中全局设置了 `first-line-indent: 2em`，Agent 生成的 `.typ` 结构不稳定（伪标题、块内段落、手动 `#pad` 等），导致 PDF 中标题与正文首行缩进时而出现、时而缺失，用户需二次修改。Agent 也无法可靠区分「标题语法」与「正文语法」。应在模板层默认取消全局首行缩进，改由段间距保证层级，并同步修订语法手册与少量模板不一致处。

## What Changes

- **默认取消中文全局首行缩进**：`apply-zh-body` 不再设置 `first-line-indent`；保留 `indent-cjk` token 供显式场景使用。
- **主题可选开关**：`make-theme(...)` 新增 `cjk-paragraph-indent: false`（默认 `false`）；为 `true` 时在 `apply-zh-body` 恢复 `first-line-indent: indent-cjk`，供用户主动开启传统文书风格。
- **语法手册规范**：在 `typst-guide.md` 增补「标题必须用 `=`、禁止伪标题与滥用 `#pad`」；移除/改写与全局缩进矛盾的示例；更新 §0.2 导出表（若 `make-theme` 签名变化）。
- **模板一致性修补**：`paper-zh.typ` 去掉对参考文献的 `#pad(left: indent-cjk)`（或改为 hanging indent 专用 helper）；`paper-en.typ` 去掉硬编码 `#pad(left: 2em)`；paper 定理块可选复用 `lecture.typ` 组件（非必须，见 design）。
- **测试**：扩展现有模板零警告测试；新增手册片段或 fixture 断言「默认中文主题下 heading 与正文均无首行缩进」。

**不纳入**：
- 限制 `#outline` 目录深度（保持 `indent: auto` 与现有深度行为）。
- 新增 figure/quote 等 show 规则（可后续独立变更）。
- 编译引擎、诊断、字体栈改动。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `typst-export`：修订中文正文段落缩进策略（默认无首行缩进、主题可选开启）；修订内置手册对段落/标题的 Agent 约束；模板零警告与 token 一致性要求延续。

## Impact

- 资源：`src-tauri/assets/typst-templates/common/{fonts,tokens}.typ`、`syntax/typst-guide.md`；场景模板 `paper/paper-{zh,en}.typ`（小幅）。
- 测试：`src-tauri/src/tools/typst_export/guide_tests.rs`、`compile.rs` 内联测试。
- 行为：**BREAKING（视觉）**——已有依赖默认首行缩进的中文 PDF 在升级后将变为无首行缩进；可通过 `make-theme(cjk-paragraph-indent: true)` 恢复旧观感。
- 依赖：无新 crate；仍 Typst 0.13 / typst-as-lib 离线编译。
