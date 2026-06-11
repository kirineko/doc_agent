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
系统 SHALL 允许在同一项目下创建多个会话，并在它们之间切换。侧栏 MUST 提供「新建」按钮以显式创建空会话；显式新建 MUST NOT 自动触发 starter 推荐问生成。

#### Scenario: 新建与切换会话
- **WHEN** 用户在某项目下新建第二个会话
- **THEN** 系统创建独立会话并可在会话列表中切换，互不影响

#### Scenario: 新建不自动初始化
- **WHEN** 用户点击侧栏「新建」
- **THEN** 创建标题为默认值的空会话并选中，不触发 starter

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

### Requirement: 发送时懒创建会话
系统 SHALL 支持在用户已选项目、尚无 activeSession 时，于首次发送消息时创建会话并关联该消息；懒创建使用的 model 与 thinking 配置 MUST 与前端 pending 配置一致，默认 DeepSeek V4 Flash、thinking enabled、effort high。

#### Scenario: 首次发送创建会话
- **WHEN** 用户在草稿态发送第一条消息
- **THEN** 持久化新会话记录，消息写入该会话，会话出现在侧栏列表

### Requirement: 切换项目选中最近会话
系统 SHALL 在用户切换 activeProject 时，将 activeSession 设为该项目下 `updated_at` 最新的一条会话；若列表为空则不选中任何会话（草稿态）。切换时 MUST NOT 删除或修改其他项目的会话数据。

#### Scenario: 自动选中最近会话
- **WHEN** 用户点击项目列表中的某项目且该项目有 2 个以上会话
- **THEN** 选中 updated_at 最新的会话并加载其历史

#### Scenario: 无会话时草稿态
- **WHEN** 用户点击尚无会话的项目
- **THEN** 不创建会话，activeSession 为空，用户可输入或稍后新建/发送

### Requirement: 首轮会话自动标题
系统 SHALL 在用户完成首轮或第二轮对话回合后，为仍保持默认标题（「新会话」或「会话 N」）的会话自动生成简短标题；标题 MUST 为用户意图摘要，长度不超过 18 个字符（含省略号）；自动命名 MUST NOT 覆盖用户已修改的非默认标题。

#### Scenario: 首轮实质提问生成标题
- **WHEN** 用户首条消息为「请帮我分析 SK1002 课程归档资料」且该轮对话完成
- **THEN** 会话标题更新为不超过 18 字的意图摘要（如含 `SK1002` 等关键信息），侧栏列表刷新可见

#### Scenario: 首轮泛化开场保持默认
- **WHEN** 用户首条消息仅为「你好」且该轮对话完成
- **THEN** 会话标题保持「新会话」，不写入寒暄或助手自我介绍

#### Scenario: 第二轮重试命名
- **WHEN** 首轮仅为泛化开场、标题仍为默认，且用户第二条消息为「分析 SK1002 归档资料」并完成该轮对话
- **THEN** 系统根据第二条用户消息生成不超过 18 字的标题；该轮不使用助手回复作为标题来源

#### Scenario: 第二轮仍泛化则保持默认
- **WHEN** 首轮为「你好」、第二轮仍为「在吗」，且标题仍为默认
- **THEN** 标题保持「新会话」，不再继续自动尝试

#### Scenario: 已有非默认标题不覆盖
- **WHEN** 会话标题已被设为「课程资料」且用户发送第二条消息
- **THEN** 系统不自动修改该标题

#### Scenario: 第三轮及以后不再自动命名
- **WHEN** 会话已有 3 条及以上用户消息且标题仍为默认
- **THEN** 系统不再尝试自动标题

