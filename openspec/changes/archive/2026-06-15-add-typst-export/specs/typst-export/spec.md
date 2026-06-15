## ADDED Requirements

### Requirement: typst_to_pdf 工具

系统 SHALL 提供 `typst_to_pdf`，将项目沙箱内 Typst 源编译为 PDF。编译 MUST 使用嵌入的 `typst-as-lib` 引擎，不得依赖外网或用户安装 Typst CLI。

输入 `path`：项目相对 `.typ` 文件，或含 `main.typ` 的目录（缺省则报错）。输出 `out_path` MUST 为沙箱内 `.pdf` 路径。内置模板虚拟路径 `/doc-agent/typst/**` MUST 可被用户 `.typ` 通过 `#import` 引用。

#### Scenario: 编译单个 typ 文件

- **WHEN** Agent 对 `docs/exam.typ` 调用 `typst_to_pdf`，`out_path` 为 `docs/exam.pdf`
- **THEN** 沙箱内生成有效 PDF，返回输出路径与页数

#### Scenario: 编译目录入口

- **WHEN** `path` 为含 `main.typ` 的目录
- **THEN** 以 `main.typ` 为入口编译并输出 PDF

#### Scenario: 目录无 main.typ

- **WHEN** `path` 为不含 `main.typ` 的目录
- **THEN** 返回明确错误，不写入 PDF

#### Scenario: 引用内置模板

- **WHEN** 用户 `.typ` 含 `#import "/doc-agent/typst/common/fonts.typ": *`
- **THEN** 编译成功且使用配置的字体栈

#### Scenario: 超时

- **WHEN** 编译超过 60 秒
- **THEN** 返回超时错误

### Requirement: 内置 Typst 模板

系统 SHALL 内置中英 Typst 模板，覆盖报告、试卷、论文、讲义四类场景，各含中文与英文版本。每套模板 MUST 通过 `typst_list_templates` 列出，并通过 `typst_read_template` 返回完整源码。

公共模块 MUST 包含 `common/fonts.typ`（中英字体栈与数学字体）与 `common/page.typ`（A4 页面预设）。

#### Scenario: 列出模板

- **WHEN** 调用 `typst_list_templates`
- **THEN** 返回至少 8 条记录（report/exam/paper/lecture × zh/en），含 id、category、lang、title、import_path

#### Scenario: 读取模板

- **WHEN** 调用 `typst_read_template`，`template` 为 `exam/exam-zh`
- **THEN** 返回该模板 Typst 源码字符串

#### Scenario: 未知模板 id

- **WHEN** `template` 不存在
- **THEN** 返回参数错误

### Requirement: 字体策略

模板字体栈 MUST 优先使用 Windows/macOS 常见系统字体名（含 Microsoft YaHei、SimSun、SimHei、Times New Roman、Songti SC、PingFang SC 等），并 MUST 提供 Typst 内嵌字体与 Noto CJK 作为回退，以便在无微软字体的环境仍可编译。

#### Scenario: Windows 系统字体

- **WHEN** 在已安装微软雅黑与宋体的 Windows 上编译中文模板
- **THEN** PDF 使用相应系统字体渲染中文

#### Scenario: 无系统中文字体

- **WHEN** 在无 SimSun/微软雅黑的环境编译
- **THEN** 编译仍成功，回退至内嵌/Noto 字体
