## ADDED Requirements

### Requirement: 运行时目录与存在性 op
系统 SHALL 在 `skill_run` 嵌入式运行时提供沙箱内的目录 listing 与路径存在性检查，经 Native op 实现且每次访问 MUST 经过 `Sandbox` 路径校验。

#### Scenario: doc_list 列出直接子项
- **WHEN** 脚本调用 `doc_list("unpacked/ppt/slides")` 且该路径为项目内合法目录
- **THEN** 返回 JSON 数组，每项含 `name` 与 `is_dir`
- **AND** 结果 MUST 不包含以 `.` 开头的隐藏项、`node_modules`、`target`、`~$` 临时文件
- **AND** MUST NOT 因 OOXML 工作目录名而拒绝 listing（Agent 需能列 `unpacked/` 下 slide 文件）

#### Scenario: doc_list 默认项目根
- **WHEN** 脚本调用 `doc_list()` 或 `doc_list(".")`
- **THEN** 返回项目根目录的直接子项 listing

#### Scenario: doc_exists 判断存在
- **WHEN** 脚本调用 `doc_exists("template.docx")` 且沙箱内该路径存在
- **THEN** 返回 `true`

#### Scenario: doc_exists 不存在返回 false
- **WHEN** 脚本调用 `doc_exists("missing.txt")` 且路径不存在
- **THEN** 返回 `false`（不抛错）

#### Scenario: fs polyfill 映射
- **WHEN** 脚本调用 `fs.existsSync(path)` 或 `fs.readdirSync(path)`
- **THEN** 行为分别等价于 `doc_exists(path)` 与 `doc_list(path)` 的名称数组

### Requirement: 运行时能力矩阵文档
系统 SHALL 提供内置 `skill_read {"skill":"runtime"}`，描述 boa 嵌入式运行时的引擎、入口、`require`/全局变量、文件 API、polyfill、自动 normalize 行为、已知限制与故障修复流程。skill 索引（`index_markdown`）与 system prompt MUST 指向该文档；Agent 在编写或修复 `skill_run` 脚本前 SHOULD 先读取。

#### Scenario: 读取 runtime 文档
- **WHEN** Agent 调用 `skill_read {"skill":"runtime"}`
- **THEN** 返回运行时能力矩阵全文
- **AND** 文档 MUST 明确：引擎为 boa_engine（非 Node/V8）；语言为 JavaScript（非 TypeScript）；不支持通用 ES module `import`（常见 import 由 normalize 改写）

#### Scenario: skill_run 描述指向 runtime 文档
- **WHEN** Agent 查看 `skill_run` 工具 schema
- **THEN** 工具描述 MUST 要求先 `skill_read runtime`

### Requirement: import 语句兼容 normalize
系统 SHALL 在执行前将 `skill_run` 脚本中常见的 ES module 风格 `import … from '…'` 单行语句改写为等价的 `require`/全局变量写法（与现有 `require()` normalize 同层），**不得**声称支持完整 ES module 语义。

#### Scenario: import pptxgenjs 改写
- **WHEN** 脚本含 `import PptxGenJS from 'pptxgenjs'`
- **THEN** normalize 后脚本 MUST NOT 保留该 import 语句
- **AND** 脚本 MUST 可通过全局 `PptxGenJS` 或等价绑定执行

#### Scenario: import docx 解构改写
- **WHEN** 脚本含 `import { Document, Packer } from 'docx'`
- **THEN** normalize 后 MUST 等价于从全局 `docx` 解构

#### Scenario: 无法识别的 import 给出指引
- **WHEN** 脚本含无法改写的 import 且执行失败
- **THEN** 错误 hint MUST 指向 `skill_read runtime` 与 require/全局写法

## MODIFIED Requirements

### Requirement: skill_run 执行 JavaScript
系统 SHALL 提供 `skill_run` 工具，在嵌入式 JS 运行时（**boa_engine**，纯 Rust）中执行 Agent 提供的 **JavaScript** 代码，并返回脚本的结构化结果。运行时 MUST 随应用内置，不依赖用户机器上的 Node.js。`skill_run` MUST accept exactly one script source: inline `code` or project-relative `path`. 执行前 MUST 应用脚本 normalize（require/import 兼容、main 包裹、剥离末尾 `main()` 调用）。

#### Scenario: 执行简单脚本
- **WHEN** Agent 调用 `skill_run`，代码为返回 JSON 值的脚本
- **THEN** 工具返回该 JSON 值，且执行发生在应用进程内嵌运行时

#### Scenario: 通过项目路径执行脚本
- **WHEN** Agent 调用 `skill_run` 并传入 `path` 指向项目沙箱内的 JavaScript 文件
- **THEN** 系统 SHALL 读取该文件并执行其脚本内容
- **AND** 路径读取 MUST 经过现有 `Sandbox` 校验

#### Scenario: 拒绝多个脚本来源
- **WHEN** Agent 调用 `skill_run` 同时传入 `code` 和 `path`
- **THEN** 工具 MUST 返回 invalid arguments 错误
- **AND** 不得执行任一脚本来源

#### Scenario: 拒绝缺失脚本来源
- **WHEN** Agent 调用 `skill_run` 且未传入 `code` 或 `path`
- **THEN** 工具 MUST 返回 invalid arguments 错误

#### Scenario: 脚本异常透出
- **WHEN** 脚本抛出异常
- **THEN** 工具返回包含异常消息与堆栈的错误，供 Agent 修正后重试

### Requirement: 内置文档生成库
运行时 SHALL 内置 `docx`（docx-js）、`pptxgenjs`、`exceljs`、`pdf-lib` 四个库的预打包 bundle；脚本通过 **`require('模块名')` 或全局变量**（ExcelJS、PptxGenJS、docx、PDFLib）使用，无需网络下载。Bundle MUST 按代码关键字启发式按需注入；**MUST NOT** 仅因路径字符串含 `.pptx` 扩展名（如 `"output.pptx"`）而加载 pptxgenjs bundle。

#### Scenario: pptxgenjs 生成 PPT
- **WHEN** 脚本使用 `PptxGenJS` 或 `require('pptxgenjs')` 并生成演示文稿写入项目目录
- **THEN** 产物 `.pptx` 为可被 Office 打开的合法 OOXML

#### Scenario: exceljs 生成样式化表格
- **WHEN** 脚本用 exceljs 创建含字体、边框、数字格式与条件格式（colorScale）的工作簿
- **THEN** 产物 `.xlsx` 打开后样式与条件格式生效

#### Scenario: 仅含 pptx 路径字符串不加载 pptxgenjs
- **WHEN** 脚本仅通过 `fs.readFileSync` / `fs.writeFileSync` 编辑 XML，且代码中出现 `"output.pptx"` 等路径字符串但无 `PptxGenJS` / `pptxgenjs` 库用法
- **THEN** 运行时 MUST NOT 注入 pptxgenjs bundle
