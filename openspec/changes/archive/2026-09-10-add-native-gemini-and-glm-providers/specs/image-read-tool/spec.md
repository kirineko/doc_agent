## MODIFIED Requirements

### Requirement: 按模型条件注册

当会话模型 `supports_vision=false` 时，系统 MUST NOT 向 Agent 暴露 `image_read` 工具定义（动态工具列表过滤）。

#### Scenario: DeepSeek 会话无 image_read

- **WHEN** 会话模型为 DeepSeek V4 Pro
- **THEN** `tools` 列表不包含 `image_read`

#### Scenario: DeepSeek Flash 有 image_read

- **WHEN** 会话模型为 DeepSeek Flash
- **THEN** `tools` 列表包含 `image_read`

#### Scenario: MiMo v2.5 有 image_read

- **WHEN** 会话模型为 MiMo v2.5
- **THEN** `tools` 列表包含 `image_read`
