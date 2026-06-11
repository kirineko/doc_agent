## ADDED Requirements

### Requirement: clarify skill 注册与枚举

系统现有的 `document-skills` Requirement「内置 Skill 仓库」SHALL 扩展：skill 仓库新增 `clarify` skill（名称 `clarify`，描述「文档创作前的需求澄清流程，帮助明确内容、结构与排版风格」）。`skill_read` 工具的 `skill` 参数枚举说明 MUST 包含 `clarify`。

#### Scenario: 应用启动可枚举 clarify skill

- **WHEN** 应用启动且未做任何额外安装
- **THEN** 系统能枚举出 `clarify` skill 的 name 与 description，与 docx / pptx 等并列

#### Scenario: skill_read 工具描述包含 clarify

- **WHEN** Agent 读取 `skill_read` 工具的描述文本
- **THEN** 描述中的可用 skill 枚举包含 `clarify`

---

### Requirement: 系统提示词包含 clarify 触发指示

系统 SHALL 在每轮对话的 system prompt 中注入 clarify 触发指示：收到全新文档创作请求且需求信息不完整（缺少主题/受众/结构/风格中 ≥ 2 项）时，MUST 先 `skill_read clarify` 并按流程执行澄清，不得直接进入 `skill_run`。

#### Scenario: 触发指示可见

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含「模糊文档创作请求前 MUST skill_read clarify」的指示，与已有的 Office deliverable `skill_read` 指示共存

#### Scenario: 触发指示不替换原有指示

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 同时包含「生成 Office 交付物前 MUST skill_read 对应格式 skill」和「模糊创作请求前 MUST skill_read clarify」两条指示
