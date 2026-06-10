# 嵌入式 JS 脚本运行时能力

## ADDED Requirements

### Requirement: skill_run 执行 JavaScript
系统 SHALL 提供 `skill_run` 工具，在嵌入式 JS 运行时（deno_core / V8）中执行 Agent 提供的 JavaScript/TypeScript 代码，并返回脚本的结构化结果。运行时 MUST 随应用内置，不依赖用户机器上的 Node.js。

#### Scenario: 执行简单脚本
- **WHEN** Agent 调用 `skill_run`，代码为返回 JSON 值的脚本
- **THEN** 工具返回该 JSON 值，且执行发生在应用进程内嵌运行时

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
