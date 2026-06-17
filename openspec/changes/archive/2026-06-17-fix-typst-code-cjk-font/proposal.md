## Why

代码块（`raw`）的字体目前完全没有被模板钉死：`common/fonts.typ` 虽定义了等宽字体栈 `font-mono`，但没有任何 `show raw` 规则引用它（死代码）。结果是代码块走 Typst 默认行为 + 逐字形回退——英文恰好命中等宽字体尚可，**代码块中的中文落入不受控的 glyph fallback，Windows 上常被解析为隶书等书法体，既不稳定也与代码观感严重冲突**。

## What Changes

- 在 `apply-zh-body` / `apply-en-body` 中新增 `show raw` 规则，为代码块显式指定「等宽英文 + 受控中文衬线」的字体栈，使代码块中文确定性地使用思源宋体 / 宋体（`font-serif-zh`：Windows `SimSun`，macOS `Songti SC`/`STSong`，跨平台兜底 `Noto Serif SC`）。
- 顺手清理死代码 `font-mono`：将其正式接入 `show raw`，使等宽英文字体栈（Consolas / Menlo / Courier New）真正生效，不再依赖 Typst 隐式默认。
- 确保代码块字体在三个平台字体栈（macOS / Windows / fallback）下行为一致、可复现，且零 `unknown font family` 警告。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `typst-export`: 「字体策略」要求新增对代码块（`raw`）字体的约束——代码块 MUST 显式钉死等宽英文 + 受控中文衬线字体栈，中文 MUST NOT 依赖 Typst 不受控的 glyph fallback。

## Impact

- 代码：`src-tauri/assets/typst-templates/common/fonts.typ`（`apply-zh-body` / `apply-en-body` 新增 `show raw`；`font-mono` 接线）。
- 测试：`typst_export` 编译测试新增代码块（含中文）零警告用例。
- 无新增依赖；不改变工具接口、不改变捆绑字体清单（沿用已打包的 Noto Serif SC）。
