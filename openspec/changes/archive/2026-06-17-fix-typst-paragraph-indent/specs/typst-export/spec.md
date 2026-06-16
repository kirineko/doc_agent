## ADDED Requirements

### Requirement: 中文段落缩进默认关闭且可主题开启

`apply-zh-body` MUST NOT 默认设置 `first-line-indent`。中文正文段落层级 MUST  primarily 通过 `par-spacing`（及既有行距 token）分隔，以保证 Agent 生成的 markup 结构下缩进行为一致。`make-theme(...)` MUST 支持自由轴参数 `cjk-paragraph-indent`（默认 `false`）；当其为 `true` 时，`apply-zh-body` MUST 对段落设置 `first-line-indent: indent-cjk`（取自 `tokens.typ`）。`indent-cjk` token MUST 保留供主题与显式排版使用。英文 `apply-en-body` 行为不变（不设首行缩进）。

#### Scenario: 默认中文主题无首行缩进

- **WHEN** 用户 `.typ` 使用 `#show: apply-zh-body` 或 `#show: apply-zh-body.with(theme: make-theme())` 并编译
- **THEN** 正文段落无全局 `first-line-indent`，标题（`= …`）与正文左缘对齐一致

#### Scenario: 主题开启传统首行缩进

- **WHEN** 用户 `.typ` 使用 `#show: apply-zh-body.with(theme: make-theme(cjk-paragraph-indent: true))` 并编译
- **THEN** 正文段落应用 `first-line-indent: indent-cjk`（2em）

#### Scenario: 八套场景模板默认无首行缩进

- **WHEN** 逐一编译 report/exam/paper/lecture 的 zh/en 共 8 个内置场景模板
- **THEN** 均编译成功、零 warning，且不因全局首行缩进导致列表项或块内段落出现不一致左缩进

### Requirement: 语法手册段落与标题 Agent 规范

内置 `syntax/typst-guide.md` MUST 载明 Agent 编写中文 Typst 时的段落与标题约束：章节标题 MUST 使用 `=` / `==` 等 heading 语法，MUST NOT 以单独一行的粗体或纯文本充当章节标题；MUST NOT 对普通正文滥用 `#pad(left: indent-cjk)` 或重复 `#set par(first-line-indent: …)`（除非文档显式启用 `cjk-paragraph-indent: true`）。手册 MUST 说明 `apply-zh-body` 已内置段落规则，§3 示例 MUST NOT 与默认无首行缩进策略矛盾。手册「常见错误」表 MUST 包含伪标题与滥用 `#pad` 条目。手册 MUST NOT 要求或示例限制 `#outline` 的 `depth`（目录深度保持 Agent/用户自由配置）。

#### Scenario: 手册含标题与段落规范

- **WHEN** 读取 `syntax/typst-guide` 中段落/标题相关章节
- **THEN** 可见上述 MUST 规则及 `cjk-paragraph-indent` 说明

#### Scenario: 手册常见错误含伪标题

- **WHEN** 读取手册「常见错误」表
- **THEN** 包含「用粗体/纯文本代替 `=` 标题」与「滥用 `#pad(left: …)` 模拟缩进」的纠正说明

#### Scenario: outline 深度不受限

- **WHEN** 场景模板或手册示例使用 `#outline(title: [目录], indent: auto)` 且未指定 `depth`
- **THEN** 仍为合法推荐写法，本能力不强制 `depth` 上限

## MODIFIED Requirements

### Requirement: 可主题化且保可读性下限

设计系统 MUST 区分锁定轴与自由轴：字号阶、正文字号、行距、版心与正文墨色为锁定轴，MUST NOT 由主题覆盖；强调色 `accent`、区块底色 `fill`、密度 `density`、标题风格 `heading-style`、封面 `cover`、中文段落首行缩进开关 `cjk-paragraph-indent` 为自由轴，MUST 可由 Agent 通过 `make-theme(...)` 选择预设调色板或传入自定义值定制。`tokens.typ` MUST 内置至少 5 套调色板预设。`accent` MUST 仅作用于标题强调、链接、表头、分隔线与区块底，MUST NOT 改变正文文字颜色。`density` MUST 仅缩放留白阶且限定在有界区间内，MUST NOT 改变正文字号。`cjk-paragraph-indent` 默认 MUST 为 `false`；为 `true` 时 MUST 仅影响中文 `apply-zh-body` 的 `first-line-indent`，MUST NOT 改变字号或行距。8 个场景模板 MUST 预设彼此不同的默认调色板，以呈现开箱多样性；exam 场景 MUST 锁定为墨色灰阶主题，忽略彩色 accent，以保证黑白打印可读。

#### Scenario: 选择预设调色板

- **WHEN** 用户 `.typ` 以 `apply-zh-body.with(theme: make-theme(palette: "burgundy"))` 套用主题
- **THEN** 标题/链接/表头按 burgundy 强调色渲染，正文仍为墨色，字号与行距不变，且默认无首行缩进

#### Scenario: 自定义强调色

- **WHEN** 传入 `make-theme(accent: rgb("#0b6e6e"))`
- **THEN** 强调元素使用该色，正文可读性与版式结构不受影响

#### Scenario: 默认呈现多样性

- **WHEN** 分别编译 report、paper、lecture、exam 的默认模板
- **THEN** 它们呈现彼此不同的强调色（exam 为墨色灰阶），而非统一单色

#### Scenario: 试卷锁定墨色

- **WHEN** 对 exam 模板传入彩色 `accent`
- **THEN** 渲染仍为墨色灰阶，关键信息不依赖彩色填充

#### Scenario: 可选开启中文首行缩进

- **WHEN** 传入 `make-theme(cjk-paragraph-indent: true)` 并用于 `apply-zh-body`
- **THEN** 正文段落获得 `indent-cjk` 首行缩进，其余锁定轴（字号、行距、版心、正文墨色）不变

### Requirement: 设计系统 token 唯一真相

系统 MUST 提供内置设计 token 模块 `common/tokens.typ`（虚拟路径 `/doc-agent/typst/common/tokens.typ`），定义字号阶、间距阶、行距、线宽、页边距、中文首行缩进量 `indent-cjk`、字体角色别名，以及调色板预设与主题构造（含 `cjk-paragraph-indent`）。全部 common 模块与 8 个场景模板的字号、间距、行距、线宽 MUST 取自该模块，MUST NOT 在模板中硬编码上述样式魔数或直接书写字体名字符串（字体名集中于 `fonts.typ`/`fonts-stack.typ`）。场景模板 MUST NOT 对参考文献等区块使用与全局段落策略冲突的硬编码 `#pad(left: 2em)` 或 `#pad(left: indent-cjk)` 模拟正文缩进。

#### Scenario: 模板从 token 取样式

- **WHEN** 审视任一场景模板与 common 模块源码
- **THEN** 其字号/间距/行距/线宽引用 `tokens.typ` 的 token，而非散落的字面量

#### Scenario: 调整 token 全局生效

- **WHEN** 修改 `tokens.typ` 中某字号或间距 token 的值
- **THEN** 所有引用该 token 的模板渲染随之一致变化，无需逐文件改动

#### Scenario: paper 参考文献无硬编码左 pad

- **WHEN** 审视 `paper/paper-zh.typ` 与 `paper/paper-en.typ` 参考文献区块
- **THEN** 不使用 `#pad(left: indent-cjk)` 或 `#pad(left: 2em)` 包裹条目列表
