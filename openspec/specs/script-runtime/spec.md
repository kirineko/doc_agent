# script-runtime Specification

## Purpose
TBD - created by archiving change add-document-skills-runtime. Update Purpose after archive.
## Requirements
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

### Requirement: 运行时沙箱

运行时 MUST 禁用网络访问；文件读写 MUST 仅通过宿主注入的自定义 op 进行，且每次访问经现有 `Sandbox` 路径校验；单次执行 MUST 有超时上限（默认 30 秒），超时即终止并返回错误。写入类 op 在落盘前 MUST 通过文件锁系统申请目标路径 Write lock，冲突时 MUST 返回错误且不写入磁盘。

#### Scenario: 越界写被拒

- **WHEN** 脚本尝试写入项目根目录之外的路径
- **THEN** op 返回沙箱错误，文件未被创建

#### Scenario: 写锁冲突被拒

- **WHEN** 脚本尝试写入已被其他 session 占用的 `out.pptx`
- **THEN** op 返回 file busy 错误，`out.pptx` 不被覆盖

#### Scenario: 网络不可用

- **WHEN** 脚本尝试发起 fetch 请求
- **THEN** 运行时报错（无网络扩展），执行不挂起

### Requirement: skill_run 临时脚本恢复区

系统 SHALL 使用项目沙箱 `.cache/skill-run/<session_key>/` 作为 `skill_run` 的临时脚本恢复区（`session_key = hash(session_id)`，同一会话内各 turn 共用同一路径；段名为 8 位 hex）。该目录 MUST only contain temporary recovery files for the owning session。清理时机分两级：成功且无交付物时可立即清理；写出 Office 交付物或存在排版告警时在 turn 内保留供修复，turn 结束时只要不存在未修复的执行失败（`error.json`）就 MUST 删除整个 session scratch 目录，不依赖 Agent 显式调用。

#### Scenario: inline 脚本执行前保存

- **WHEN** Agent 调用 `skill_run` 并传入 `code`
- **THEN** 系统 SHALL 在执行前将脚本内容写入 `.cache/skill-run/<session_key>/script.js`

#### Scenario: 两个会话脚本路径不同

- **WHEN** session A 与 session B 同时调用 `skill_run` inline code
- **THEN** A 与 B 的 `script_path` MUST 不同
- **AND** 任一 session 清理时 MUST NOT 删除另一 session 的 scratch 目录

#### Scenario: 同 session 跨 turn 共用 script_path

- **WHEN** session A 在 turn 1 的 `skill_run.code` 执行失败
- **AND** 用户在 turn 2 继续修复
- **THEN** turn 2 的 `script_path` MUST 与 turn 1 相同
- **AND** Agent 可用 `fs_patch` + `skill_run {"path":"<script_path>"}` 重跑

#### Scenario: path 重跑使用返回 script_path

- **WHEN** `skill_run.code` 执行失败
- **THEN** 工具错误结果 MUST include session-scoped `script_path`
- **AND** Agent 可用 `skill_run {"path":"<script_path>"}` 重跑该脚本

#### Scenario: 失败现场跨 turn 保留

- **WHEN** `skill_run.code` 执行失败
- **THEN** 系统 SHALL 保留该 session 的 `script.js` 并写入同目录 `error.json`
- **AND** turn 结束时该 session scratch 目录 MUST NOT 被清理

### Requirement: fs_patch 局部文本修改

系统 SHALL 提供 `fs_patch` 工具，对项目内 UTF-8 文本文件执行精确子串替换，作为修复 `.cache/skill-run/script.js` 等大文件的首选方式（替代 `fs_write` 全量重写）。

#### Scenario: 唯一匹配替换

- **WHEN** Agent 调用 `fs_patch`，每条 edit 的 `old` 在文件中恰好出现一次
- **THEN** 系统 SHALL 应用全部替换并返回 `applied` 计数

#### Scenario: 原子性 — 任一 edit 未命中则全部不应用

- **WHEN** 任一 edit 的 `old` 未找到，或多处匹配且未设 `replace_all`
- **THEN** 系统 MUST NOT 写入任何修改
- **AND** 返回结构化错误，列出每条未命中 edit 的原因（`not found` / `multiple matches`）

#### Scenario: 拒绝无效 edit

- **WHEN** edit 的 `old` 为空字符串，或 `old` 与 `new` 相同
- **THEN** 工具 MUST 返回 invalid arguments 错误

### Requirement: skill_run 精准故障诊断
系统 SHALL provide precise, actionable diagnostics for failed `skill_run` calls and malformed tool-call arguments so the Agent can repair the current script instead of regenerating the full script.

#### Scenario: 工具参数 JSON 无效
- **WHEN** Agent 产生的工具调用参数不是合法 JSON
- **THEN** 系统 MUST NOT silently replace arguments with an empty object
- **AND** 系统 SHALL return an error message containing the JSON parser detail, line, column, and a short raw snippet around the failure

#### Scenario: JavaScript 解析失败
- **WHEN** `skill_run` 脚本无法被 JavaScript runtime 解析
- **THEN** 工具错误结果 SHALL include the parser message
- **AND** 当 runtime 提供位置时，错误结果 SHALL include line, column, source context, and `script_path` when available

#### Scenario: 引号相关诊断
- **WHEN** `skill_run` 失败位置附近包含 quote-like characters
- **THEN** 工具错误结果 SHALL include quote diagnostics that distinguish ASCII `"` (`U+0022`) from smart quotes such as `“` (`U+201C`) and `”` (`U+201D`)
- **AND** 诊断 MUST NOT silently rewrite script text

#### Scenario: 工具调用流式输出被截断
- **WHEN** provider streaming ends with output length truncation while a tool call is being produced
- **THEN** 系统 SHALL report a truncation error instead of treating the partial arguments as a normal `skill_run` script or generic missing-code error

### Requirement: 运行时目录与存在性 op

系统 SHALL 在 `skill_run` 嵌入式运行时提供沙箱内的目录 listing 与路径存在性检查，经 Native op 实现且每次访问 MUST 经过 `Sandbox` 路径校验。Agent SHOULD 使用工具返回的 OOXML `out_dir` 调用 `doc_list` / `fs.readdirSync`，MUST NOT 假设固定 `unpacked/` 存在。

#### Scenario: doc_list 列出返回的 OOXML 目录

- **WHEN** `ooxml_unpack` 返回 `.cache/ooxml/a782729d/4b2c025c`
- **AND** 脚本调用 `doc_list(".cache/ooxml/a782729d/4b2c025c/ppt/slides")`
- **THEN** 返回 slide 文件列表

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

