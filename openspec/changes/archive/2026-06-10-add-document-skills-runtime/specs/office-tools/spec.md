# Office 工具能力（delta）

## REMOVED Requirements

### Requirement: PPT 生成排除在 MVP 之外
**Reason**: Document Skills 运行时落地后，PPT 生成由 `script-runtime`（内置 pptxgenjs）与 `ooxml-toolchain`（模板解包 / 回包）提供，原限制不再成立。
**Migration**: PPT 生成走 `skill_read("pptx")` 获取指引 → `skill_run`（pptxgenjs 从零创建）或 `ooxml_unpack`/`ooxml_pack`（基于模板编辑）；PPT 读取仍用现有 `office_read_markdown`。

## ADDED Requirements

### Requirement: PPT 生成与编辑
系统 SHALL 支持在项目目录内生成新 PPT（经脚本运行时 pptxgenjs）及基于既有 PPT 模板的内容编辑（经 OOXML 解包 / 回包），产物 MUST 为可被 Office 打开的合法 OOXML。

#### Scenario: 从零生成 PPT
- **WHEN** Agent 按 pptx skill 指引经 `skill_run` 生成多页演示文稿
- **THEN** 项目内生成 `.pptx`，可被 Office 正常打开且页面内容正确

#### Scenario: 基于模板编辑
- **WHEN** Agent 对既有 `.pptx` 解包、替换文本与图片引用后回包
- **THEN** 产物保留模板母版与样式，仅目标内容被修改
