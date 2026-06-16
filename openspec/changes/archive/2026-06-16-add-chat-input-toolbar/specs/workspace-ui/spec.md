## ADDED Requirements

### Requirement: Chat 输入工具栏

Chat 输入区 SHALL 在 textarea 下方（或与发送按钮同一 composite 输入框底栏）展示三个工具按钮：**+**（上传文件到项目根）、**图片**（选择图片作为消息附件）、**/**（打开斜杠命令图形菜单）。各按钮 MUST 具备 tooltip 与无障碍 `aria-label`。澄清进行中（`activeClarify`）、busy、initializing 时，三按钮与 textarea MUST 一并 disabled。

#### Scenario: 工具栏可见

- **WHEN** 用户已选项目且输入区未 disabled
- **THEN** 输入框底栏展示 +、图片、/ 三个按钮

#### Scenario: 澄清期间禁用

- **WHEN** session 存在 pending clarify
- **THEN** 三按钮 disabled，与 textarea 一致

### Requirement: 斜杠命令图形菜单（二级分类）

除键盘 `/` 触发的 fuzzy 弹层外，系统 SHALL 提供 **/** 按钮打开的**二级分类**菜单：第一级为分类（通用、Word、PPT、Excel、PDF、Web，顺序与 `CATEGORY_ORDER` 一致），第二级为该分类下全部斜杠命令（展示 `label` 与一行 `description`）。选中命令后 MUST 调用与键盘斜杠相同的 prompt 插入逻辑（`insertSlashPrompt`）：填入 registry `prompt`、选中首个 `{{占位符}}`、**MUST NOT** 自动发送。

#### Scenario: 二级菜单选 Word 命令

- **WHEN** 用户点击 / 按钮 → 选择 Word 分类 → 选择「精准修改 Word」
- **THEN** 输入框填入对应 prompt 且首个占位符被选中，消息未发送

#### Scenario: 六类全展示

- **WHEN** 用户打开 / 图形菜单
- **THEN** 第一级可见 general、word、ppt、excel、pdf、web 共六类

#### Scenario: 与 @ 弹层互斥

- **WHEN** `@` 文件弹层正在展示
- **THEN** 不展示斜杠图形菜单（若已打开则关闭）

#### Scenario: Esc 关闭图形菜单

- **WHEN** 斜杠图形菜单打开且用户按 Esc 或点击外部
- **THEN** 菜单关闭，输入内容不变

### Requirement: 输入区 placeholder 补充上传提示

非 disabled 状态下，textarea placeholder SHOULD 在现有 `@`、`/`、粘贴图片提示基础上，补充 **+** 上传文件至项目根的简短说明（与 clarify/busy/initializing 专用 placeholder 互斥）。

#### Scenario: 常规定位 placeholder

- **WHEN** 用户已选项目、输入区可用
- **THEN** placeholder 提及 `@` 引用、`/` 或图形菜单任务模板、粘贴或按钮添加图片、**+** 上传文件
