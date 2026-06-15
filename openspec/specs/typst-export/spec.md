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

#### Scenario: Windows 系统字体

- **WHEN** 在已安装微软雅黑与宋体的 Windows 上编译中文模板
- **THEN** PDF 使用相应系统字体渲染中文

#### Scenario: 无系统中文字体

- **WHEN** 在无 SimSun/微软雅黑的环境编译
- **THEN** 编译仍成功，回退至捆绑的 Noto SC 字体，且无 `unknown font family` 警告

### Requirement: 平台范围

`typst_to_pdf` SHALL 在 macOS 与 Windows 发布目标上可用（与现有发布矩阵一致）。Linux 不在本能力范围内。

#### Scenario: 离线编译

- **WHEN** 无网络环境下调用 `typst_to_pdf`
- **THEN** 仍可完成编译，不访问 Typst 包仓库
