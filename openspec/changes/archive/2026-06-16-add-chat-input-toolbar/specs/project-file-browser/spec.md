## ADDED Requirements

### Requirement: 在系统文件管理器中打开项目根

项目文件浏览区标题栏 SHALL 提供按钮，调用系统默认文件管理器打开当前 active 项目的**根目录**（非当前浏览子目录）。无 active 项目时按钮 MUST disabled。按钮 MUST 具备平台适当的无障碍标签（如 macOS「在 Finder 中打开」、Windows「在文件资源管理器中打开」）。

#### Scenario: 打开项目根

- **WHEN** 用户已选项目并点击该按钮
- **THEN** 系统文件管理器打开该项目根路径

#### Scenario: 无项目时不可用

- **WHEN** 未选择项目
- **THEN** 按钮 disabled 或不可见

#### Scenario: 与双击文件行为区分

- **WHEN** 用户从浏览列表双击 `报告.docx`
- **THEN** 仍用默认**应用**打开该文件，而非打开根目录
