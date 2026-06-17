## Context

`typst_to_pdf` 用嵌入的 `typst-as-lib` 引擎离线编译。正文/标题/表格/公式字体由 `common/fonts.typ` 的 `apply-zh-body` / `apply-en-body` 经 `set text` / `show` 规则控制，但**代码块（`raw`）字体从未被任何 `show raw` 规则覆盖**。`font-mono`（`Consolas / Menlo / Courier New / Libertinus Mono`）虽已定义却无引用，属死代码。

结果：代码块走 Typst 默认 + 逐字形回退。英文字符命中等宽字体观感正常；中文字符因等宽字体无 CJK 字模，触发不受控的 glyph fallback，在 Windows 上常落到隶书（`LiSu` / SIMLI.TTF）等书法体，既不稳定也与代码风格冲突。

## Goals / Non-Goals

**Goals:**

- 代码块中文确定性地使用衬线宋体（思源宋体 / 宋体），平台一致、可复现。
- 等宽英文字体栈 `font-mono` 真正生效，消除对 Typst 隐式默认的依赖。
- 全平台（macOS / Windows / fallback）零 `unknown font family` 警告。

**Non-Goals:**

- 不改变捆绑字体清单（沿用已打包的 Noto Serif SC / Noto Sans SC，不新增字体文件）。
- 不引入独立的中文等宽字体（如等距更纱黑体）；MVP 用现有衬线宋体即可。
- 不改变正文/标题/表格/公式既有字体策略。

## Decisions

**决策 1：代码块字体栈 = `(..font-mono, ..font-serif-zh)`**

在 `apply-zh-body` 与 `apply-en-body` 内新增：

```typst
show raw: set text(font: (..font-mono, ..font-serif-zh))
```

逐字形解析顺序变为确定的：英文/符号先命中 `font-mono`（Consolas → Menlo → Courier New）；中文因前面等宽体无字模而跳过，命中 `font-serif-zh` 中的平台宋体（Windows `SimSun`、macOS `Songti SC`/`STSong`），系统缺失时兜底到捆绑的 `Noto Serif SC`。

- **为何复用 `font-serif-zh` 而非新建变量**：该栈已在 `fonts-stack-{macos,windows,fallback}.typ` 按平台调好，且天然包含 `Noto Serif SC` 跨平台兜底，复用即获得「思源宋体 / 宋体」目标且免维护重复清单。其首项 `(name: "Times New Roman", covers: "latin-in-cjk")` 在代码块语境无害——`font-mono` 在前已覆盖全部拉丁字符。
- **替代方案**：新增 `font-mono-cjk` 专用栈。被否决：与现有平台栈重复，维护成本高，MVP 不需要。

**决策 2：中文代码用衬线宋体而非黑体**

按用户偏好，代码块中文采用宋体（`font-serif-zh`）而非黑体（`font-sans-zh`），与正文衬线风格统一。

**决策 3：清理死代码 `font-mono`**

`font-mono` 由「定义但无引用」转为被 `show raw` 实际引用，等宽英文字体栈正式生效。

**决策 4：按平台拆分 `font-mono` 栈**

`show raw` 会解析字体栈中全部族名；跨平台混排 Consolas / Libertinus Mono 会在 macOS / CI 产生 `unknown font family` 警告。故 `font-mono` 迁入 `fonts-stack-{macos,windows,fallback}.typ`：macOS `Menlo`、Windows `Consolas`、fallback `DejaVu Sans Mono`。

## Risks / Trade-offs

- [代码块中文用宋体在视觉上比等宽/黑体略不「代码化」] → 这是用户明确选择；保留后续切换黑体或引入中文等宽字体的空间（仅需改一处字体栈）。
- [`font-serif-zh` 首项 `covers: latin-in-cjk` 进入代码字体列表] → 因 `font-mono` 在前已命中拉丁字符，该项不会触发，行为无副作用；测试覆盖验证。
- [不同平台代码块中文字形不一致（SimSun vs Songti SC）] → 与正文策略一致、可接受；裸系统统一回退 Noto Serif SC 保证下限。
