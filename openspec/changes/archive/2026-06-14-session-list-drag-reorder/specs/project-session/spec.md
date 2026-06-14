## ADDED Requirements

### Requirement: 会话列表展示顺序与选中逻辑解耦

系统 SHALL 在用户切换 activeProject 时，将 activeSession 设为该项目下 `updated_at` 最新的一条会话；该选取逻辑 MUST 与侧栏会话列表的展示顺序无关。展示顺序规则（自动序 vs 手动序）见 workspace-ui spec「会话列表顺序懒激活与前端持久化」。

#### Scenario: 手动序下仍选中最近更新会话

- **WHEN** 用户切换到处于手动序模式的项目，且 `updated_at` 最新的会话不在列表顶部
- **THEN** 系统选中 `updated_at` 最新的会话并加载其历史，而非列表第一项

#### Scenario: 自动序下行为不变

- **WHEN** 用户切换到从未拖动排序的项目
- **THEN** 列表按 `updated_at` 降序展示，且选中列表首项（即最近更新会话）

## MODIFIED Requirements

### Requirement: 切换项目选中最近会话

系统 SHALL 在用户切换 activeProject 时，将 activeSession 设为该项目下 `updated_at` 最新的一条会话；若列表为空则不选中任何会话（草稿态）。切换时 MUST NOT 删除或修改其他项目的会话数据。选取 MUST 基于各会话的 `updated_at` 字段比较，MUST NOT 依赖侧栏列表的展示顺序或数组下标。

#### Scenario: 自动选中最近会话

- **WHEN** 用户点击项目列表中的某项目且该项目有 2 个以上会话
- **THEN** 选中 `updated_at` 最新的会话并加载其历史

#### Scenario: 无会话时草稿态

- **WHEN** 用户点击尚无会话的项目
- **THEN** 不创建会话，activeSession 为空，用户可输入或稍后新建/发送
