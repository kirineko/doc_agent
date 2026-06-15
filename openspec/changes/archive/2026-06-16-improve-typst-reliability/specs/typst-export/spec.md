## ADDED Requirements

### Requirement: 结构化编译诊断

`typst_to_pdf` 编译或 PDF 导出失败时 MUST 返回结构化诊断，而非裸 Debug 字符串。每条诊断 MUST 包含 `message`，并在能定位时包含 `file`、`line`、`column` 与出错源码片段 `snippet`（含位置指示符）；MUST 透传 Typst 提供的 `hints`；MUST 附 `error_type` 分类与对应中文 `fix_guidance`。位置 MUST 由 Typst `Span` 经源文件还原为人类可读的行列。

当 `Span` 无法定位（detached 或源不可得）时，诊断 MUST 安全降级为仅 `message` + `hints`（`error_type` 标记为未定位），且 MUST NOT panic。

#### Scenario: 未定义变量报错含行列与片段

- **WHEN** 用户 `.typ` 第 21 行使用未定义函数（如 `#fillblank()`）导致编译失败
- **THEN** 返回的诊断含 `file`、`line`(=21)、`column`、出错行 `snippet` 与位置指示符、原始 `message` 与 `fix_guidance`

#### Scenario: 无法定位的诊断安全降级

- **WHEN** 某诊断的 `span` 为 detached 或其源文件不在可还原集合内
- **THEN** 返回该诊断的 `message` 与 `hints`，`error_type` 标记为未定位，且不发生 panic

#### Scenario: PDF 导出阶段失败

- **WHEN** 文档可解析但 PDF 导出阶段报错
- **THEN** 返回结构化错误而非 `{errors:?}` 裸 Debug 文本

### Requirement: 编译警告回传

`typst_to_pdf` MUST 将编译 warnings（如字体回退、弃用语法）随结果回传 Agent；成功时附于成功结果，失败时附于结构化错误。warnings MUST NOT 仅写入 stderr。warnings 条目 MUST 含 `message`，可定位时含 `file`/`line`。

#### Scenario: 成功但有警告

- **WHEN** 编译成功但产生字体回退等 warning
- **THEN** 成功结果在 `path`/`pages` 之外包含 `warnings` 列表

#### Scenario: 失败且有警告

- **WHEN** 编译失败且过程中产生 warning
- **THEN** 结构化错误同时包含 `diagnostics` 与 `warnings`

### Requirement: 失败引导局部修改

`typst_to_pdf` 工具描述 MUST 指示：编译失败时应依据结构化诊断用 `fs_patch` 做最小局部修改，禁止整篇重写已有 `.typ`。结构化诊断的 `fix_guidance` MUST 面向「改哪里」给出可操作建议。

#### Scenario: 工具描述含局部修改指引

- **WHEN** Agent 读取 `typst_to_pdf` 工具定义
- **THEN** 描述明确「失败优先 `fs_patch` 局部修复、勿整篇重写」

### Requirement: 内置手册示例与 API 一致性可验证

内置 `syntax/typst-guide.md` 中标注为可独立编译的 Typst 示例 MUST 能通过嵌入引擎编译且无 error。手册「内置模块」导出清单 MUST 与 `common/*.typ` 的实际顶层导出一致；二者偏离 MUST 由自动化测试检出。

#### Scenario: 手册可编译示例无错

- **WHEN** 运行测试编译手册中标注可独立编译的 Typst 代码块
- **THEN** 全部编译成功，无 error

#### Scenario: exports 清单漂移被检出

- **WHEN** `common/*.typ` 新增/删除/重命名一个公开 `#let` 而未同步手册导出表
- **THEN** 一致性测试失败

### Requirement: 模板语法与字体零警告

全部 8 个内置场景模板（report/exam/paper/lecture × zh/en）MUST 能通过嵌入引擎编译且 `warnings` 为空（含无 `unknown font family` 警告），并 MUST NOT 使用已弃用的 Typst 0.13 API。

#### Scenario: 全部场景模板零警告编译

- **WHEN** 在已注入捆绑字体的环境逐一编译 8 个场景模板
- **THEN** 每个模板编译成功且无任何 warning

### Requirement: 设计系统 token 唯一真相

系统 MUST 提供内置设计 token 模块 `common/tokens.typ`（虚拟路径 `/doc-agent/typst/common/tokens.typ`），定义字号阶、间距阶、行距、线宽、页边距、字体角色别名，以及调色板预设与主题构造。全部 common 模块与 8 个场景模板的字号、间距、行距、线宽 MUST 取自该模块，MUST NOT 在模板中硬编码上述样式魔数或直接书写字体名字符串（字体名集中于 `fonts.typ`/`fonts-stack.typ`）。

#### Scenario: 模板从 token 取样式

- **WHEN** 审视任一场景模板与 common 模块源码
- **THEN** 其字号/间距/行距/线宽引用 `tokens.typ` 的 token，而非散落的字面量

#### Scenario: 调整 token 全局生效

- **WHEN** 修改 `tokens.typ` 中某字号或间距 token 的值
- **THEN** 所有引用该 token 的模板渲染随之一致变化，无需逐文件改动

### Requirement: 可主题化且保可读性下限

设计系统 MUST 区分锁定轴与自由轴：字号阶、正文字号、行距、版心与正文墨色为锁定轴，MUST NOT 由主题覆盖；强调色 `accent`、区块底色 `fill`、密度 `density`、标题风格 `heading-style`、封面 `cover` 为自由轴，MUST 可由 Agent 通过 `make-theme(...)` 选择预设调色板或传入自定义值定制。`tokens.typ` MUST 内置至少 5 套调色板预设。`accent` MUST 仅作用于标题强调、链接、表头、分隔线与区块底，MUST NOT 改变正文文字颜色。`density` MUST 仅缩放留白阶且限定在有界区间内，MUST NOT 改变正文字号。8 个场景模板 MUST 预设彼此不同的默认调色板，以呈现开箱多样性；exam 场景 MUST 锁定为墨色灰阶主题，忽略彩色 accent，以保证黑白打印可读。

#### Scenario: 选择预设调色板

- **WHEN** 用户 `.typ` 以 `apply-zh-body.with(theme: make-theme(palette: "burgundy"))` 套用主题
- **THEN** 标题/链接/表头按 burgundy 强调色渲染，正文仍为墨色，字号与行距不变

#### Scenario: 自定义强调色

- **WHEN** 传入 `make-theme(accent: rgb("#0b6e6e"))`
- **THEN** 强调元素使用该色，正文可读性与版式结构不受影响

#### Scenario: 默认呈现多样性

- **WHEN** 分别编译 report、paper、lecture、exam 的默认模板
- **THEN** 它们呈现彼此不同的强调色（exam 为墨色灰阶），而非统一单色

#### Scenario: 试卷锁定墨色

- **WHEN** 对 exam 模板传入彩色 `accent`
- **THEN** 渲染仍为墨色灰阶，关键信息不依赖彩色填充
