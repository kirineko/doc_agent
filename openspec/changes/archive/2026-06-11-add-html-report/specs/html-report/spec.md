# html-report Specification (Delta)

## ADDED Requirements

### Requirement: 内置 html-report skill

系统 SHALL 内置 `html-report` skill（`assets/skills/html-report/SKILL.md`，编译期内置），指导 Agent 在用户项目沙箱内生成**静态**网页报告项目：至少包含入口 `index.html`，推荐包含 `styles.css`；可选 `script.js` 与 `assets/` 下资源文件。Skill MUST 禁止使用 React、Vue、npm、Vite 等复杂框架或构建链。

#### Scenario: skill 可枚举与读取

- **WHEN** 应用启动或 Agent 调用 `skill_read {"skill": "html-report"}`
- **THEN** 返回 html-report 的 SKILL.md 全文，且包含目录结构约定与工具分工说明

#### Scenario: 推荐目录结构

- **WHEN** Agent 按 skill 创建新报告
- **THEN** skill 指引将产物置于项目内 `reports/<报告名>/` 或用户明确指定的其他项目相对路径，且入口文件为 `index.html`

### Requirement: 报告产物持久落盘于项目目录

HTML 报告的所有交付文件（HTML、CSS、JS、assets）MUST 通过 `fs_write` / `fs_patch`（或等效沙箱写工具）写入用户选定的**项目根目录**内，并持久保留于磁盘。报告产物 MUST NOT 写入 `.skill-run/` 或任何仅在 turn 内存在、会被自动清理的临时目录。

#### Scenario: fs_write 写入报告

- **WHEN** Agent 调用 `fs_write` 写入 `reports/q1/index.html`
- **THEN** 文件存在于项目沙箱内，重启应用后仍可访问，且出现在项目文件浏览区

#### Scenario: 禁止临时目录

- **WHEN** Agent 按 html-report skill 生成报告
- **THEN** 产出路径段 MUST NOT 为 `.skill-run` 或其子路径

### Requirement: 静态资源与打印样式规范

html-report skill SHALL 规定：样式与脚本使用**相对路径**引用；中文报告须指定系统字体（如 PingFang SC、Microsoft YaHei）；须包含打印样式（`@page`、`@media print`、表格分页建议）。Skill SHOULD 建议避免依赖外网 CDN，以保证离线打开与 PDF 导出时样式完整。

#### Scenario: 打印 CSS 指引可见

- **WHEN** Agent 读取 html-report skill
- **THEN** 文档包含可复制的 `@page` / `@media print` 示例片段

### Requirement: 生成与 PDF 导出解耦

html-report skill SHALL 将「生成 HTML 报告」与「导出 PDF」描述为**独立**能力：生成流程 MUST NOT 要求调用 `html_to_pdf`；`html_to_pdf` 的说明仅作为可选后续步骤出现。

#### Scenario: 仅生成 HTML

- **WHEN** 用户要求「做一份 HTML 销售报告」且未要求 PDF
- **THEN** Agent 可仅使用 `fs_write` 等完成报告，无需调用 `html_to_pdf`

#### Scenario: 预览不依赖导出

- **WHEN** 报告已生成于项目目录
- **THEN** 用户可通过文件浏览区双击 `index.html`，由系统默认浏览器打开预览，无需先导出 PDF

### Requirement: 与数据分析管道可选衔接

html-report skill MAY 说明与 `data_query` 的衔接方式（如将查询结果嵌入 HTML 表格），但该衔接 MUST 为可选，不得构成生成 HTML 的前置条件。

#### Scenario: 数据驱动报告

- **WHEN** Agent 先执行 `data_query` 再按 skill 生成 HTML
- **THEN** 查询结果写入项目内 CSV 或内联于 HTML，且报告文件仍持久落盘于项目目录
