# script-runtime Specification

## Purpose
TBD - created by archiving change add-document-skills-runtime. Update Purpose after archive.
## Requirements
### Requirement: skill_run 执行 JavaScript
系统 SHALL 提供 `skill_run` 工具，在嵌入式 JS 运行时（deno_core / V8）中执行 Agent 提供的 JavaScript/TypeScript 代码，并返回脚本的结构化结果。运行时 MUST 随应用内置，不依赖用户机器上的 Node.js。`skill_run` MUST accept exactly one script source: inline `code` or project-relative `path`.

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
运行时 SHALL 内置 `docx`（docx-js）、`pptxgenjs`、`exceljs`、`pdf-lib` 四个库的预打包 bundle，脚本可直接 import，无需网络下载。

#### Scenario: pptxgenjs 生成 PPT
- **WHEN** 脚本 `import PptxGenJS from "pptxgenjs"` 并生成演示文稿写入项目目录
- **THEN** 产物 `.pptx` 为可被 Office 打开的合法 OOXML

#### Scenario: exceljs 生成样式化表格
- **WHEN** 脚本用 exceljs 创建含字体、边框、数字格式与条件格式（colorScale）的工作簿
- **THEN** 产物 `.xlsx` 打开后样式与条件格式生效

### Requirement: 运行时沙箱
运行时 MUST 禁用网络访问；文件读写 MUST 仅通过宿主注入的自定义 op 进行，且每次访问经现有 `Sandbox` 路径校验；单次执行 MUST 有超时上限（默认 30 秒），超时即终止并返回错误。

#### Scenario: 越界写被拒
- **WHEN** 脚本尝试写入项目根目录之外的路径
- **THEN** op 返回沙箱错误，文件未被创建

#### Scenario: 网络不可用
- **WHEN** 脚本尝试发起 fetch 请求
- **THEN** 运行时报错（无网络扩展），执行不挂起

#### Scenario: 超时熔断
- **WHEN** 脚本执行死循环
- **THEN** 到达超时上限后执行被终止，工具返回超时错误

### Requirement: skill_run 临时脚本恢复区

系统 SHALL 使用项目沙箱 `.cache/skill-run/` 作为 `skill_run` 的临时脚本恢复区。该目录 MUST only contain temporary recovery files for the latest run。清理时机分两级：成功且无交付物时立即清理；写出 Office 交付物或存在排版告警时在 turn 内保留供修复，turn 结束时只要不存在未修复的执行失败（`.cache/skill-run/error.json`）就 MUST 自动清理，不依赖 Agent 显式调用。

#### Scenario: inline 脚本执行前保存

- **WHEN** Agent 调用 `skill_run` 并传入 `code`
- **THEN** 系统 SHALL 在执行前将脚本内容写入 `.cache/skill-run/script.js`

#### Scenario: 纯计算脚本成功后立即清理

- **WHEN** `skill_run` 执行成功且未写出 `.docx/.pptx/.xlsx/.xlsm` 交付物、无排版告警
- **THEN** 系统 SHALL 删除 `.cache/skill-run/` 临时目录（若存在）

#### Scenario: 写出交付物后 turn 内保留脚本

- **WHEN** `skill_run` 执行成功且写出 Office 交付物（或返回 `style_warnings`）
- **THEN** 系统 SHALL 保留 `.cache/skill-run/script.js` 供本 turn 内 `fs_patch` 修复与 `path` 重跑
- **AND** 成功响应 SHALL include `script_path` 与修复指引
- **AND** 系统 SHALL 清除上一次失败遗留的 `.cache/skill-run/error.json`

#### Scenario: turn 结束自动清理

- **WHEN** Agent turn 结束（正常完成或达到最大工具步数）
- **AND** `.cache/skill-run/error.json` 不存在（无未修复的脚本失败）
- **THEN** 系统 SHALL 自动删除 `.cache/skill-run/`，无论 `style_warnings` 是否被处理

#### Scenario: 失败现场跨 turn 保留

- **WHEN** `skill_run.code` 执行失败
- **THEN** 系统 SHALL 保留 `.cache/skill-run/script.js` 并写入 `.cache/skill-run/error.json`
- **AND** 工具错误结果 MUST include `script_path` with value `.cache/skill-run/script.js`
- **AND** turn 结束时该目录 MUST NOT 被清理

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

