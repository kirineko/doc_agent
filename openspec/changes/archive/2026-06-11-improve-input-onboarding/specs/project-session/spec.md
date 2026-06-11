## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: 项目下的多会话
系统 SHALL 允许在同一项目下创建多个会话，并在它们之间切换。侧栏 MUST 提供「新建」按钮以显式创建空会话；显式新建 MUST NOT 自动触发 starter 推荐问生成。

#### Scenario: 新建与切换会话
- **WHEN** 用户在某项目下新建第二个会话
- **THEN** 系统创建独立会话并可在会话列表中切换，互不影响

#### Scenario: 新建不自动初始化
- **WHEN** 用户点击侧栏「新建」
- **THEN** 创建标题为默认值的空会话并选中，不触发 starter
