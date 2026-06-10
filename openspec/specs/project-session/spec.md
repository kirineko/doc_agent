# project-session Specification

## Purpose
TBD - created by archiving change bootstrap-doc-agent-mvp. Update Purpose after archive.
## Requirements
### Requirement: 以目录建立项目
系统 SHALL 允许用户选择一个本地目录作为项目，并以该目录作为操作与沙箱的基本单位。

#### Scenario: 选择目录创建项目
- **WHEN** 用户选择一个本地文件夹
- **THEN** 系统创建以该目录为根的项目并持久化，后续 Agent 操作以此为根目录

### Requirement: 项目下的多会话
系统 SHALL 允许在同一项目下创建多个会话，并在它们之间切换。

#### Scenario: 新建与切换会话
- **WHEN** 用户在某项目下新建第二个会话
- **THEN** 系统创建独立会话并可在会话列表中切换，互不影响

### Requirement: 会话上下文隔离
系统 SHALL 保证每个会话是独立上下文，会话之间不共享记忆，构造请求时仅使用当前会话的历史。

#### Scenario: 会话互不串话
- **WHEN** 会话 A 中讨论的内容未在会话 B 提及
- **THEN** 在会话 B 请求模型时，上下文不包含会话 A 的任何消息

### Requirement: 会话历史与工具调用持久化
系统 SHALL 将会话消息（含 assistant 的 `reasoning_content`）与过程中的工具调用持久化到本地数据库，并在重启后可恢复。

#### Scenario: 重启后恢复
- **WHEN** 用户关闭并重新打开应用
- **THEN** 项目、会话、历史消息与工具调用记录均可被重新加载查看

### Requirement: 目录沙箱约束
系统 SHALL 强制 Agent 的所有文件操作落在项目根目录内，拒绝越界访问（含 `..` 与符号链接穿越）。

#### Scenario: 拒绝越界写入
- **WHEN** 工具尝试写入项目根目录之外的路径
- **THEN** 系统拒绝该操作并返回错误，不修改沙箱外任何文件

### Requirement: 项目隐藏与同目录恢复
系统 SHALL 支持将项目从列表中隐藏（仅隐藏显示，不删除任何数据）；用户重新选择同一目录创建项目时，系统 MUST 复用既有项目记录并自动取消隐藏，该项目的全部历史会话保持可见。

#### Scenario: 隐藏项目
- **WHEN** 用户在项目列表对某项目执行移除操作
- **THEN** 该项目从列表消失，但其会话、消息与工具调用记录在数据库中完整保留

#### Scenario: 重选同目录自动恢复
- **WHEN** 用户再次通过目录选择创建指向同一 `root_path` 的项目
- **THEN** 系统不新建记录，而是恢复显示原项目（id 不变），其会话列表完整可见

#### Scenario: 隐藏当前激活项目
- **WHEN** 用户隐藏的是当前激活项目
- **THEN** 界面自动切换到剩余项目中的第一个；若无剩余项目则回到未选择状态

