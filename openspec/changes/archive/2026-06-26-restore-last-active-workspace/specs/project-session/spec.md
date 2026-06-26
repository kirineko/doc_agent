## ADDED Requirements

### Requirement: 冷启动恢复上次工作区
系统 SHALL 将上次 active 项目与会话 id 持久化于前端 `localStorage`（键 `doc-agent-last-active-workspace`）。应用启动并完成 `list_projects` 后，若缓存的项目 id 仍存在于项目列表，MUST 自动选中该项目；加载会话列表后，若缓存的 session id 仍属于该项目，MUST 选中该会话，否则 MUST 回退为 `updated_at` 最新会话。用户手动切换项目或会话时 MUST 更新缓存；项目自列表移除或 active 项目清空时 MUST 清除或忽略无效缓存。

#### Scenario: 重启后恢复项目与会话
- **WHEN** 用户上次在 project A / session S 工作并退出应用
- **THEN** 再次启动后自动选中 project A 与 session S（若二者仍存在）

#### Scenario: 会话已删除时回退
- **WHEN** 缓存的 session id 已不存在但 project 仍存在
- **THEN** 选中该项目下 `updated_at` 最新的会话

#### Scenario: 项目已移除
- **WHEN** 缓存的 project id 不在 `list_projects` 结果中
- **THEN** 清除无效缓存并保持未选项目空态

#### Scenario: 项目无会话
- **WHEN** 缓存指向有效项目且 session id 为空或已失效
- **THEN** 选中该项目并进入草稿态（不选中会话）

#### Scenario: 无任何项目
- **WHEN** `list_projects` 返回空列表
- **THEN** 不尝试恢复，保持添加项目引导空态
