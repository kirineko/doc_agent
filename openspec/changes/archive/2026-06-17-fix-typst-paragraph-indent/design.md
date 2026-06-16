## Context

`apply-zh-body`（`common/fonts.typ`）通过 `#set par(first-line-indent: indent-cjk)` 对全部段落施加 2em 首行缩进。Agent 生成的 `.typ` 常混用 `=` 标题、伪标题（粗体/纯文本）、`#block`/`#pad` 等 markup，Typst 对 `par` 的解析边界不稳定，导致 PDF 中标题与正文缩进偶发不一致。英文 `apply-en-body` 无此设置，问题仅影响中文路径。

约束：
- Typst 0.13 / typst-as-lib 离线编译，不新增 crate。
- 设计系统已有 `tokens.typ` + `make-theme(...)`；`indent-cjk` token 已存在。
- 用户明确要求：**不限制 `#outline` 目录深度**。
- 变更范围限于模板资源 + 手册 + 测试，不动编译引擎。

## Goals / Non-Goals

**Goals:**
- 默认中文 PDF 段落**无首行缩进**，段间距（`par-spacing`）承担层级分隔。
- 用户可通过 `make-theme(cjk-paragraph-indent: true)` 主动恢复传统首行两字缩进。
- 语法手册明确 Agent 标题/段落规范，消除与模板矛盾的示例。
- 修补 paper 模板中手动 `#pad(left: …)` 与全局缩进叠加的不一致。
- 测试守护：8 模板零 warning + 默认主题缩进行为可断言。

**Non-Goals:**
- 限制 `#outline(depth: …)` 或目录层级。
- 新增 figure/quote/code-block 等 show 规则。
- 智能 `show par.where(...)` 按上下文选择性缩进（复杂度高、Agent 边界 case 仍多）。
- PNG 视觉回归或人工美学评审。

## Decisions

### 决策 1：默认关闭中文首行缩进

`apply-zh-body` 的 `#set par(...)` **移除** `first-line-indent`，保留 `justify`、`leading`、`spacing`。

**理由**：Agent 自动生成场景下，结构无关的段间距比依赖 markup 解析的首行缩进更稳定；与 `apply-en-body` 行为对齐。

**备选**：
- (a) 保留全局缩进 + 强化手册 —— 无法消除 block/list 边界 case，已否决。
- (b) 智能 show 规则 —— 实现与维护成本高，留作未来独立 spike。

### 决策 2：主题自由轴新增 `cjk-paragraph-indent`

在 `make-theme(...)` 增加布尔参数 `cjk-paragraph-indent: false`（默认）。主题 dict 携带该字段；`apply-zh-body` 仅在 `theme.cjk-paragraph-indent == true` 时设置 `first-line-indent: indent-cjk`。

**理由**：满足少数正式文书需求，不增加 Agent 负担（默认 false）；与现有自由轴（palette、density、heading-style）一致。

**用法示例**：

```typst
#show: apply-zh-body.with(theme: make-theme(cjk-paragraph-indent: true))
```

### 决策 3：保留 `indent-cjk` token，不删除

`tokens.typ` 中 `indent-cjk = 2em` 继续存在，供：
- `cjk-paragraph-indent: true` 主题使用；
- 未来 hanging indent helper（若需要参考文献悬挂缩进）。

**理由**：token 唯一真相原则不变；仅改变默认是否应用。

### 决策 4：paper 模板参考文献去掉左 pad

`paper-zh.typ` 删除 `#pad(left: indent-cjk)[…]` 包裹，改为普通段落列表（与默认无缩进一致）。`paper-en.typ` 删除 `#pad(left: 2em)[…]`，同样改为普通列表。

**理由**：手动 pad 与（旧）全局缩进叠加造成双重偏移；无首行缩进后 pad 语义不清。若后续需要悬挂缩进，单独引入 `#let ref-item(...)` helper（本变更不强制）。

**备选**：paper 定理块复用 `lecture.typ` 的 `definition-zh` —— 可选优化，本变更**不纳入** MVP，避免 scope 膨胀。

### 决策 5：手册修订要点

在 `syntax/typst-guide.md`：

1. **新增 §「段落与标题规范」**（或并入 §5/§23）：
   - 章节标题 MUST 使用 `=` / `==`，禁止单独一行 `*粗体*` 或纯文本充当标题。
   - 正文 MUST NOT 对普通段落使用 `#pad(left: indent-cjk)` 或 `#set par(first-line-indent: …)`（除非显式开启 `cjk-paragraph-indent: true`）。
2. **修订 §3 示例**：删除「推荐同时 `#set par(first-line-indent: 2em)`」的误导；强调 `#show: apply-zh-body` 已含段落规则。
3. **修订 §22 主题**：文档 `make-theme` 新参数 `cjk-paragraph-indent`。
4. **§23 常见错误表**：增加伪标题、滥用 `#pad` 两行。
5. **§0.2 exports 表**：若 `make-theme` 签名变化，同步更新。

**不修改**：`#outline(title: [目录], indent: auto)` 保持现状，不添加 `depth` 限制。

### 决策 6：测试策略

1. 现有 8 模板零 warning 测试继续通过（编译路径不变）。
2. 新增 fixture `.typ`（或内联 source）编译后，通过源码级断言：`apply-zh-body` 默认 `#set par` 不含 `first-line-indent`；`cjk-paragraph-indent: true` 时含 `indent-cjk`。
3. 手册 exports 一致性测试随 `make-theme` 签名更新；可编译示例块仍须零 error。

**理由**：不引入 PDF 像素对比；结构与编译零 warning 为客观验收，与 `improve-typst-reliability` 一致。

## Risks / Trade-offs

- **[视觉 BREAKING] 升级后中文 PDF 默认无首行缩进** → 在 proposal/CHANGELOG 说明；一行 `make-theme(cjk-paragraph-indent: true)` 可恢复。
- **[Agent 仍写伪标题]** → 手册规范 + 常见错误表缓解；无缩进后伪标题至少不会出现「缩进假标题」的二次混乱。
- **[用户期望传统书籍排版]** → 主题开关覆盖；不在默认路径强加。
- **[paper 参考文献失去左缩进]** → 无首行缩进场景下左对齐列表可接受；悬挂缩进可后续独立 helper。

## Migration Plan

1. 改 `tokens.typ` / `fonts.typ` → 跑模板编译测试。
2. 改 paper 模板 + `typst-guide.md` → 跑 guide_tests / exports 测试。
3. 无数据迁移；用户项目内 `.typ` 若手写 `#set par(first-line-indent: …)` 不受影响（局部覆盖仍有效）。

回滚：还原 `fonts.typ` 一行 `first-line-indent` 即可恢复旧默认。

## Open Questions

无阻塞项。paper 定理块复用 `lecture.typ` 留待后续独立变更（若需要）。
