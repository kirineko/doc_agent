## ADDED Requirements

### Requirement: 全局并行上限提示

前端 SHALL 基于 per-session running map 派生当前 running/stopping 数量。当本地已知数量达到 3 时，输入区 MUST 阻止新发送并提示「当前已有 3 个任务正在执行，请稍后重试」。后端仍 MUST 作为权威校验；若后端返回全局满额错误，前端 MUST 保留用户输入，不得清空草稿。

#### Scenario: 本地满额禁用发送

- **WHEN** 前端已知 3 个 session 处于 running 或 stopping
- **THEN** 当前输入区发送按钮 disabled 或点击后展示满额提示

#### Scenario: 后端满额错误保留输入

- **WHEN** 用户发送时后端返回全局满额错误
- **THEN** 输入框内容仍保留，用户可稍后重试

### Requirement: 文件占用错误展示

前端 SHALL 能展示后端文件锁冲突错误。错误文案 MUST 包含被占用路径；当后端提供 blocking session 标题或 id 时，前端 SHOULD 展示「当前 xxx 已被会话 yyy 占用，请稍后重试」。

#### Scenario: 工具结果 file_busy

- **WHEN** tool_result 内容表示 `file_busy`
- **THEN** UI 在工具链卡片或 toast 中展示占用路径与重试建议

### Requirement: 后台 session terminal 同步

前端 SHALL 对非 active session 的 `turn_complete`、`turn_cancelled`、`turn_awaiting_user` 事件更新对应 session running 状态。若事件所属 project 是当前 active project，前端 SHOULD 刷新 session list 与项目文件浏览状态；但 MUST NOT 用后台 session 的 messages 覆盖当前 active session 的消息列表。

#### Scenario: 后台完成不覆盖当前消息

- **WHEN** active session 为 B
- **AND** 后台 session A 收到 `turn_complete`
- **THEN** A 的侧栏 running 指示消失
- **AND** 当前中间区仍显示 B 的消息

#### Scenario: 后台文件变更刷新项目文件区

- **WHEN** 后台 session A 的 tool_result 含 `changed_paths`
- **AND** A 属于当前 active project
- **THEN** 项目文件浏览区按现有规则刷新当前目录或文件索引
