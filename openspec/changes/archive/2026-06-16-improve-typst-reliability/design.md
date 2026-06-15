## Context

`typst_to_pdf` 失败时，`compile.rs` 用 `format!("{d:?}")` 输出 `SourceDiagnostic`，其 `span` 字段是不透明的 `Span`，Debug 仅打印内部 id；warnings 仅 `eprintln!` 到 stderr。错误经 `ToolError::Execution(String)` → `{ "error": "<裸 Debug>" }` 回传 Agent，导致无法定位、倾向整篇重写。

关键约束：
- 引擎为 `typst-as-lib 0.14.4` + `typst 0.13.1`，离线编译；不新增 crate。
- `TypstWorld` 在 `typst-as-lib` 中为私有，compile 后不返回 World 句柄。
- 已确认可用 API：`SourceDiagnostic { severity, span, message, hints, trace }` 字段公开；`Span::id() -> Option<FileId>`；`typst::syntax::Source` 提供 `range(span)`、`byte_to_line`、`byte_to_column`。
- 体量规则：`compile.rs` 软上限 300 行（当前约 220 行），诊断格式化逻辑较多时应抽到独立 `diagnostics.rs`。

## Goals / Non-Goals

**Goals:**
- 编译/导出失败返回结构化诊断：`error_type`、`file`、`line`、`column`、`snippet`（出错行及指示符）、`message`、`hints`、`fix_guidance`。
- warnings 在成功与失败两种情况下都回传 Agent。
- 工具描述显式引导「失败用 `fs_patch` 局部修改，禁止整篇重写」。
- 手册示例可被自动编译校验；`common/*.typ` exports 与手册表自动比对。
- 8 模板 + 4 common 模块通过零警告编译并完成美学/字体/设计/语法审查修订。

**Non-Goals:**
- 不建调用效果评估 harness（问题 5）。
- 不建 PNG 渲染 / 视觉回归基建；模板美学靠人工审查 + 编译零警告把关。
- 不改变 `typst_to_pdf` 成功路径的返回结构（仅新增失败/警告信息）。

## Decisions

### 决策 1：用自建 `FileId → Source` 映射还原 span，而非访问私有 World

`TypstWorld` 私有，无法编译后复用。改为在格式化诊断时，用与编译相同的输入重建 `Source`：
- 入口文件：读 entry 文本，`FileId` 由 `typst_vpath` 对应的虚拟路径构造（与编译时一致）。
- 内置模块：`bundled::static_sources()` 已有 `(vpath, text)`，逐一 `Source::new(file_id, text)`。

对每条诊断：`span.id()` → 查映射得 `Source` → `source.range(span)` 得字节区间 → `byte_to_line` / `byte_to_column` 得行列 → 截取该行并加 `^^^` 指示符。

理由：span 编号在「相同 FileId + 相同文本」再解析时确定性一致（与 typst 官方 CLI 用 `world.source(id)` 复原诊断同理）。这样无需 fork 或依赖私有结构。

**备选**：(a) fork typst-as-lib 暴露 World —— 维护成本高；(b) 仅靠 `Span::range()` —— 仅对 `from_range` 创建的 span 有效，编译期 numbered span 取不到，故不可靠。

**降级**：若 `span.id()` 为 `None`（detached）或不在映射中，则退化为仅 `message + hints`，并标 `error_type: "unlocated"`。保证永不 panic。

### 决策 2：错误分类 `error_type` 取轻量启发式

不强行解析 Typst 内部错误枚举（未公开稳定分类）。基于 `message` 文本做关键词归类：`unknown-variable`、`unexpected-argument`、`unknown-font`、`file-not-found`(import)、`syntax`、`type-error`、`other`。每类映射一句中文 `fix_guidance`（如 unknown-variable → 「检查函数/变量名拼写，多数内置函数用连字符，如 fill-blank」）。

理由：足够驱动 Agent 局部修复且实现简单；分类表集中在一处便于迭代。

**备选**：穷举 typst 错误类型 —— 上游不保证稳定，过度工程。

### 决策 3：结构化错误通过 `ToolError::Structured(Value)` 承载

`registry.rs` 已有 `ToolError::Structured(Value)`，`to_json_value` 直接透传。失败时返回：

```json
{
  "error": "typst 编译失败",
  "diagnostics": [
    { "error_type": "unknown-variable", "file": "main.typ", "line": 21, "column": 34,
      "message": "unknown variable: fillblank", "snippet": "...^^^...",
      "hints": ["..."], "fix_guidance": "..." }
  ],
  "warnings": [ { "message": "...", "file": "...", "line": .. } ]
}
```

成功时在原 `{ path, pages }` 基础上，若有 warnings 增加 `"warnings": [...]`。

### 决策 4：诊断逻辑独立成 `diagnostics.rs`

新增 `src-tauri/src/tools/typst_export/diagnostics.rs`，承载 `Source` 映射构建、span 还原、分类、`fix_guidance` 表、结构化序列化；`compile.rs` 仅调用。`CompileOutput` 增加 `warnings: Vec<DiagnosticInfo>` 字段；编译失败路径返回携带结构化诊断的错误类型而非 `String`。

理由：满足单一职责与 300 行软上限，便于针对分类/还原单测。

### 决策 5：手册与模板的「真相一致性」用测试守护

- **手册代码块编译测试**：解析 `typst-guide.md` 中 ` ```typst ` 块，对可独立编译的片段（含必要 import）逐个 `compile`，断言无 error。对依赖上下文的片段以白名单标记跳过。
- **exports 一致性测试**：从 `common/*.typ` 正则提取顶层 `#let <name>`，与手册 §0.2 表中声明的导出比对，缺漏/多余即失败。
- **模板零警告测试**：扩展现有 `bundled_exam_zh_compiles_without_font_warnings` 至全部 8 模板，断言 `warnings.is_empty()`。

### 决策 6：建立 Typst 设计系统 `common/tokens.typ`（取代人工审查）

美学/字体/设计不靠逐文件主观评审，而是抽出一组**确定数值的设计 token** 作为唯一真相，所有模块/模板从中取值。token 改一次，全套文档风格随之统一变化；审查退化为「是否引用 token + 是否零警告」两条客观检查。

**字号阶（pt，模块比例 ≈1.2，正文基准 11pt）**

| token | 值 | 用途 |
|---|---|---|
| `fs-footnote` | 9pt | 页码、脚注、试卷结束语 |
| `fs-small` | 9.5pt | 表注、次要说明 |
| `fs-body` | 11pt | 正文 |
| `fs-lead` | 12pt | 摘要、引导段、副标题 |
| `fs-h3` | 12pt | 三级标题 |
| `fs-h2` | 14pt | 二级标题 |
| `fs-h1` | 16pt | 一级标题、试卷课程名 |
| `fs-title` | 20pt | 文档主标题 |

**间距阶（em）**：`sp-2xs 0.25` / `sp-xs 0.4` / `sp-sm 0.6` / `sp-md 1.0` / `sp-lg 1.6` / `sp-xl 2.4`

**行距与段落**：`leading-cjk 0.9em`（比现状 0.65em 更舒展，改善中文密排可读性）、`leading-latin 0.65em`、`par-spacing 1.1em`、`indent-cjk 2em`

**配色（地基固定 + 强调色可主题化）**：地基恒定——`color-ink #1a1a1a`（正文，**accent 永不覆盖正文色**）/ `color-muted #5f6368`（次要文字）/ `color-rule #d0d0d0`（分隔线/表格线）。强调相关（`accent` + `fill`）由主题决定，仅作用于标题强调、链接、表头、区块底，不承载正文与关键信息。

**线宽**：`stroke-hair 0.5pt` / `stroke-rule 0.75pt` / `stroke-heavy 1pt`

**页边距（按场景）**：`margin-report (x:2.5,y:2.5)cm` / `margin-paper (x:2.4,y:2.6)cm` / `margin-exam 1.8cm` / `margin-lecture (x:2.2,y:2)cm`

**字体角色（语义名，平台栈来自现有 `fonts-stack.typ`）**：`font-body`（CJK 衬线 + Times via `covers:"latin-in-cjk"`）/ `font-heading`（CJK 黑体 + Arial）/ `font-emphasis` / `font-math`（New Computer Modern Math）/ `font-mono`。`fonts.typ` 保留平台栈定义，新增按角色的语义别名供模板引用，避免模板直接写字体名字符串。

**统一 show 规则**：`apply-zh-body`/`apply-en-body` 重写为消费 token——
- `heading`：按级取 `fs-h1/h2/h3` + `font-heading` + `color-accent`，上/下间距用 `sp-lg`/`sp-sm`；
- `table`：`stroke: stroke-hair + color-rule`、表头 `fill: color-fill` + 粗体、`inset: sp-sm`；
- `link`：`color-accent`；
- `math.equation`：`font-math`。

**理由**：把「美学」转成可 diff、可复用、可一处调参的工程量；与 maintainability 规则的「重复逻辑提取共用模块」一致。

**备选**：(a) 逐模板手改魔数——重复且易漂移，已排除；(b) 引入第三方 Typst 主题包——增加外网/依赖，违反离线与不新增 crate 约束。

### 决策 7：主题化——锁定可读性、放开观感，避免千篇一律

token 不是僵死常量，而是分为**锁定轴**（保证质量下限）与**自由轴**（留给 Agent 表达）：

| 类别 | 轴 | 说明 |
|---|---|---|
| 锁定 | 字号阶、正文字号、行距、对比度、A4 版心、正文墨色 | 直接关乎可读性，不开放 |
| 锁定 | exam 强制 charcoal（墨色灰阶） | 黑白打印友好 |
| 自由 | `palette` 预设 / 自定义 `accent`、`fill` | 仅染标题/链接/表头/分隔/区块底 |
| 自由 | `density`：relaxed / normal / compact | 仅在 0.85–1.2 区间整体缩放**留白阶**，不改字号 |
| 自由 | `heading-style`：plain / accent-rule / accent-number | 标题装饰风格 |
| 自由 | `cover`：none / banner | 是否加封面横幅 |

**调色板预设**（开箱即有变化，5 套）：`academic-blue #1f4e79` / `slate #334155` / `burgundy #7a1f3d` / `forest #1f5132` / `charcoal #1a1a1a`（exam 默认）。

**API**：`tokens.typ` 导出 `palettes`（dict）、`default-theme`（dict）、`make-theme(palette: "academic-blue", accent: none, fill: none, density: "normal", heading-style: "accent-rule", cover: "none")`——`accent/fill` 显式传入时覆盖 palette，否则取 palette 默认。`apply-zh-body`/`apply-en-body` 增加 `theme: default-theme` 入参并据此生成 show 规则。

**用法**（模板/用户 `.typ` 一行定制）：

```typst
#show: apply-zh-body.with(theme: make-theme(palette: "burgundy", density: "relaxed"))
#show: apply-zh-body.with(theme: make-theme(accent: rgb("#0b6e6e"), heading-style: "accent-number"))
```

**护栏（防丑）**：accent 仅用于非正文元素；body 字号不开放（避免不可读）；`density` 限定区间；exam 忽略彩色 accent 回退 charcoal。8 套内置场景模板各预设**不同 palette**（如 report→academic-blue、paper→slate、lecture→forest、exam→charcoal），开箱即呈现多样性。

**理由**：结构与可读性统一、观感按文档变化；Agent 既能「选预设」低成本变化，也能「传 accent」深度定制，且越界不了可读性底线。

**备选**：(a) 完全锁死颜色——用户担心的千篇一律，已否决；(b) 全开放任意样式——易产出低质版面，故仅开放有界自由轴。

### Open Questions 的最终选择

- **手册可编译块标注**：采用**显式标注**。在代码块前置一行标记注释 `<!-- doc-agent:compile -->`；测试只编译被标记且自包含的块，未标记块视为片段跳过。理由：零误报、对作者直观。
- **是否抽 `tokens.typ`**：**采用**（即决策 6），作为本变更核心而非可选项。

## Risks / Trade-offs

- **span 重建与编译期编号不一致** → 用相同 FileId + 相同文本重建；映射未命中即安全降级为 message-only，绝不 panic；加单测用已知错误样例断言行列正确。
- **手册代码块片段不完整无法独立编译** → 采用白名单/标注机制，仅校验可独立编译者，避免误报阻塞 CI。
- **错误分类启发式误判** → `error_type` 仅作辅助，`message`/`hints`/`snippet` 始终原样提供；分类表集中可快速修正。
- **模板美学主观** → 以「零警告 + 一致字体栈 + 统一间距/层级 token」为客观底线，主观部分在 PR 说明留痕，不追求一次完美。
- **诊断 JSON 变大占用 token** → snippet 仅取出错行 ± 指示符，warnings 去重并限制条数（如 ≤ 10）。

## Migration Plan

无数据迁移。成功路径返回向后兼容（新增可选 `warnings` 字段）。失败路径由纯字符串变结构化 JSON——Agent 与前端 `toolLabels` 仅展示文本字段，需确认前端对 `Structured` 错误的兜底展示（读取 `error` 字段即可）。回滚：还原 `compile.rs`/`mod.rs` 即恢复旧行为。

## Open Questions

无遗留阻塞项。两项原 Open Question 已在「Open Questions 的最终选择」中定稿（显式标注 + 采用 `tokens.typ`）。实现期细节（token 数值微调、强调色是否在试卷场景禁用以保持纯黑白打印友好）可在 PR 中按场景决定。
