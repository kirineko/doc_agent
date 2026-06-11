## ADDED Requirements

### Requirement: Agent 旧版 Office 转现代格式
系统 SHALL 提供 `office_convert` 工具，将项目沙箱内的 `.doc`、`.xls`、`.ppt` 转为 `.docx`、`.xlsx`、`.pptx`。转换 MUST 使用纯 Rust 实现（`office_oxide`），不得依赖外部 Office 安装或 shell 命令。

#### Scenario: 默认输出带 -converted 后缀
- **WHEN** Agent 对 `报告.xls` 调用 `office_convert` 且未指定 `out_path`
- **THEN** 项目内生成 `报告-converted.xlsx`，工具返回该相对路径与目标格式

#### Scenario: 指定 out_path 也必须含 -converted
- **WHEN** Agent 指定 `out_path` 为 `报告.xlsx`（无 `-converted` 后缀）
- **THEN** 工具返回参数错误，不写入文件

#### Scenario: 目标已存在不覆盖
- **WHEN** `报告-converted.xlsx` 已存在于项目内且 Agent 再次转换同名源文件
- **THEN** 工具返回明确错误，现有文件内容不变

#### Scenario: 不支持的路径越界
- **WHEN** Agent 传入项目沙箱外的路径
- **THEN** 工具返回 sandbox 错误，不执行转换

### Requirement: 旧格式转换与 OOXML 编辑边界
系统 SHALL 允许对旧格式执行 `office_convert` 与 `office_read_to_markdown`，但 SHALL NOT 对 `.doc/.xls/.ppt` 执行 `ooxml_unpack` 或基于 OOXML 的编辑工具。

#### Scenario: 解包旧格式仍被拒绝
- **WHEN** Agent 对 `.doc` 调用 `ooxml_unpack`
- **THEN** 返回「旧格式不支持解包编辑，请先 office_convert」类错误

### Requirement: 优先读取、仅在必要时转换
Agent 与工具描述 SHALL 引导：旧格式文件在**仅阅读或 SQL 分析**场景下 MUST 优先使用 `office_read_to_markdown` 或 `data_query`（`.xls`），不得默认调用 `office_convert` 新建项目文件。`office_convert` 仅在下述情况使用：用户明确要求现代格式产物、或下游工具仅支持 OOXML（如 `ooxml_unpack`、`excel_write`、`xlsx_recalc`）。

#### Scenario: 分析 xls 不新建转换文件
- **WHEN** 用户要求汇总或查询某 `.xls` 表格数据
- **THEN** Agent 使用 `data_query` 直接以该 `.xls` 为数据源，项目目录内不出现新的 `-converted.xlsx`

#### Scenario: 阅读 doc 不新建转换文件
- **WHEN** 用户要求阅读或摘要某 `.doc` 内容
- **THEN** Agent 使用 `office_read_to_markdown`，不调用 `office_convert`

#### Scenario: 转换可能丢失格式
- **WHEN** Agent 因必要原因调用 `office_convert` 并完成转换
- **THEN** 工具成功生成 `-converted` 产物，且 Agent 知晓转换不保证版式保真（复杂格式可能丢失）

