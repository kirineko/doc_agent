# typst-export Specification

## Purpose

提供与 `html_to_pdf` 并列的 Typst 排版导出能力：嵌入 `typst-as-lib` 离线编译沙箱内 `.typ` 为 PDF，内置场景模板与通用语法手册，供 Agent 生成公式密集、版式严谨的文档（试卷、论文、讲义、报告等）。
## Requirements
### Requirement: typst_to_pdf 工具

系统 SHALL 提供 `typst_to_pdf`，将项目沙箱内 Typst 源编译为 PDF。编译 MUST 使用嵌入的 `typst-as-lib` 引擎，不得依赖外网或用户安装 Typst CLI。

输入 `path`：项目相对 `.typ` 文件，或含 `main.typ` 的目录（缺省则报错）。输出 `out_path` MUST 为沙箱内 `.pdf` 路径。内置资源虚拟路径 `/doc-agent/typst/**` MUST 可被用户 `.typ` 通过 `#import` 引用。

编译 MUST 在独立线程执行并设有超时（默认 60 秒）；超时 MUST 不写入最终 `out_path`（仅允许临时文件，且应清理）。

#### Scenario: 编译单个 typ 文件

- **WHEN** Agent 对 `docs/exam.typ` 调用 `typst_to_pdf`，`out_path` 为 `docs/exam.pdf`
- **THEN** 沙箱内生成有效 PDF，返回输出路径与页数

#### Scenario: 编译目录入口

- **WHEN** `path` 为含 `main.typ` 的目录
- **THEN** 以 `main.typ` 为入口编译并输出 PDF

#### Scenario: 目录无 main.typ

- **WHEN** `path` 为不含 `main.typ` 的目录
- **THEN** 返回明确错误，不写入 PDF

#### Scenario: 引用内置模块

- **WHEN** 用户 `.typ` 含 `#import "/doc-agent/typst/common/fonts.typ": *`
- **THEN** 编译成功且使用配置的字体栈

#### Scenario: 超时

- **WHEN** 编译超过 60 秒
- **THEN** 返回超时错误，且不更新最终 `out_path`

#### Scenario: 覆盖已有 PDF

- **WHEN** 对同一 `out_path` 再次调用 `typst_to_pdf` 且编译成功
- **THEN** 输出 PDF 被新内容替换

### Requirement: typst_list_templates 与 typst_read_template

系统 SHALL 提供 `typst_list_templates` 与 `typst_read_template`。

`typst_list_templates` MUST 返回内置资源列表，含 `id`、`category`、`lang`、`title`、`description`、`import_path`。

`typst_read_template` MUST 按 `template` id 返回完整源码，供 `fs_write` 复制到项目后编辑。

#### Scenario: 列出模板与手册

- **WHEN** 调用 `typst_list_templates`
- **THEN** 返回语法手册 `syntax/typst-guide` 及 report/exam/paper/lecture 各中英场景模板（至少 9 条）

#### Scenario: 读取场景模板

- **WHEN** 调用 `typst_read_template`，`template` 为 `exam/exam-zh`
- **THEN** 返回该模板 Typst 源码字符串

#### Scenario: 读取语法手册

- **WHEN** 调用 `typst_read_template`，`template` 为 `syntax/typst-guide`
- **THEN** 返回通用 Typst 语法手册（Markdown）

#### Scenario: 未知模板 id

- **WHEN** `template` 不存在
- **THEN** 返回参数错误

### Requirement: 内置 Typst 模板与公共模块

系统 SHALL 内置：

- 公共模块：`common/fonts.typ`、`common/page.typ`、`common/exam.typ`（含 `calc-item` 等试卷辅助）
- 场景模板：report、exam、paper、lecture 各 zh/en 共 8 套
- 语法手册：`syntax/typst-guide.md`（通用 Typst 0.13 参考，非仅数学）

#### Scenario: 试卷计算题编号

- **WHEN** 场景模板或用户 `.typ` 使用 `calc-item` 连续出题
- **THEN** 计算题题号自动递增

### Requirement: Typst 语法手册强制阅读

同一会话内，Agent 在调用 `typst_to_pdf`、`typst_list_templates`、`typst_read_template`（场景模板），或通过 `fs_write`/`fs_patch` 编写/修改 `.typ` 前，MUST 先 `typst_read_template` 读取 `syntax/typst-guide`。系统提示与工具描述 MUST 体现该约束。

#### Scenario: 首次 Typst 导出

- **WHEN** 用户要求生成 Typst PDF 且会话内尚未读取语法手册
- **THEN** Agent 先调用 `typst_read_template` 读取 `syntax/typst-guide`，再编写 `.typ` 并调用 `typst_to_pdf`

### Requirement: 字体策略

模板字体栈 MUST 优先使用当前平台常见系统字体（Windows：微软雅黑、宋体、黑体；macOS：Songti SC、Heiti SC、PingFang SC 等），拉丁文 MUST 使用 `covers: "latin-in-cjk"` 与 Times New Roman 分离；并 MUST 捆绑 Noto Sans SC / Noto Serif SC（Subset Regular + Bold）作为跨平台回退，经 `TypstKitFontOptions::include_dirs` 注入，以便在无平台中文字体的环境仍可无警告编译。

代码块（Typst `raw`）字体 MUST 由 `apply-zh-body` 与 `apply-en-body` 通过 `show raw` 规则显式钉死，MUST NOT 依赖 Typst 隐式默认或不受控的逐字形回退。该代码块字体栈 MUST 为「等宽英文 + 受控中文衬线」组合：英文/符号 MUST 优先使用等宽字体栈 `font-mono`（Consolas / Menlo / Courier New / Libertinus Mono），中文 MUST 确定性地使用衬线宋体栈 `font-serif-zh`（Windows `SimSun`、macOS `Songti SC`/`STSong`，并以捆绑 `Noto Serif SC` 跨平台兜底）。`font-mono` MUST 被 `show raw` 实际引用而非保留为未使用的死代码。

#### Scenario: Windows 系统字体

- **WHEN** 在已安装微软雅黑与宋体的 Windows 上编译中文模板
- **THEN** PDF 使用相应系统字体渲染中文

#### Scenario: 无系统中文字体

- **WHEN** 在无 SimSun/微软雅黑的环境编译
- **THEN** 编译仍成功，回退至捆绑的 Noto SC 字体，且无 `unknown font family` 警告

#### Scenario: 代码块中文使用衬线宋体而非回退书法体

- **WHEN** 用户 `.typ` 套用 `apply-zh-body` 并含中英文混排的代码块（` ``` ` 围栏）
- **THEN** 代码块英文使用 `font-mono`（如 Consolas），中文确定性使用 `font-serif-zh`（Windows `SimSun` / macOS `Songti SC`，缺失时 `Noto Serif SC`），MUST NOT 出现隶书等不受控的回退字体

#### Scenario: 代码块字体零警告编译

- **WHEN** 在已注入捆绑字体的环境编译含中文代码块的模板
- **THEN** 编译成功且 `warnings` 为空（无 `unknown font family`）

### Requirement: 平台范围

`typst_to_pdf` SHALL 在 macOS 与 Windows 发布目标上可用（与现有发布矩阵一致）。Linux 不在本能力范围内。

#### Scenario: 离线编译

- **WHEN** 无网络环境下调用 `typst_to_pdf`
- **THEN** 仍可完成编译，不访问 Typst 包仓库

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

### Requirement: 中文段落缩进默认关闭且可主题开启

`apply-zh-body` MUST NOT 默认设置 `first-line-indent`。中文正文段落层级 MUST primarily 通过 `par-spacing`（及既有行距 token）分隔，以保证 Agent 生成的 markup 结构下缩进行为一致。`make-theme(...)` MUST 支持自由轴参数 `cjk-paragraph-indent`（默认 `false`）；当其为 `true` 时，`apply-zh-body` MUST 对段落设置 `first-line-indent: indent-cjk`（取自 `tokens.typ`）。`indent-cjk` token MUST 保留供主题与显式排版使用。英文 `apply-en-body` 行为不变（不设首行缩进）。

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

