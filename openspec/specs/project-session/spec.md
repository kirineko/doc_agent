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
系统 SHALL 在用户切换 activeProject 时，将 activeSession 设为该项目下 `updated_at` 最新的一条会话；若列表为空则不选中任何会话（草稿态）。切换时 MUST NOT 删除或修改其他项目的会话数据。选取 MUST 基于各会话的 `updated_at` 字段比较，MUST NOT 依赖侧栏列表的展示顺序或数组下标。

#### Scenario: 自动选中最近会话
- **WHEN** 用户点击项目列表中的某项目且该项目有 2 个以上会话
- **THEN** 选中 `updated_at` 最新的会话并加载其历史

#### Scenario: 无会话时草稿态
- **WHEN** 用户点击尚无会话的项目
- **THEN** 不创建会话，activeSession 为空，用户可输入或稍后新建/发送

### Requirement: 会话列表展示顺序与选中逻辑解耦
系统 SHALL 在用户切换 activeProject 时，将 activeSession 设为该项目下 `updated_at` 最新的一条会话；该选取逻辑 MUST 与侧栏会话列表的展示顺序无关。展示顺序规则（自动序 vs 手动序）见 workspace-ui spec「会话列表顺序懒激活与前端持久化」。

#### Scenario: 手动序下仍选中最近更新会话
- **WHEN** 用户切换到处于手动序模式的项目，且 `updated_at` 最新的会话不在列表顶部
- **THEN** 系统选中 `updated_at` 最新的会话并加载其历史，而非列表第一项

#### Scenario: 自动序下行为不变
- **WHEN** 用户切换到从未拖动排序的项目
- **THEN** 列表按 `updated_at` 降序展示，且选中列表首项（即最近更新会话）

### Requirement: 会话自动标题（两轮策略）

系统 SHALL 在 `turn_complete` 时按用户消息轮次自动更新仍由系统管理的会话标题，策略如下：

1. **第 1 轮**（`user_count == 1`）：若标题仍为默认（「新会话」或「会话 N」）且 `title_user_edited == false`，将标题设为该轮用户首条消息的清洗纯文本（去除 Markdown 标记与首尾空白）。若清洗后为空则保持默认标题。若文本超过存储上限（`MAX_STORED_TITLE_CHARS`，默认 120 字符），MUST 截断后入库（含省略号）。
2. **第 2 轮**（`user_count == 2`）：若 `autotitle_llm_done == false` 且 `title_user_edited == false`，MUST 使用**当前会话绑定的模型**、**非思考模式**（`thinking_enabled = false`）调用 LLM **一次**，基于**前两轮** user/assistant 对话生成单行标题，并**覆盖**第 1 轮已写入的标题；完成后 MUST 设 `autotitle_llm_done = true`。
3. **第 3 轮及以后**（`user_count >= 3`）：MUST NOT 自动修改标题。
4. 已有 ≥2 条 user 消息的历史会话在升级或后续继续对话时，MUST NOT 补跑 LLM 命名（依赖 `autotitle_llm_done` 迁移与 `user_count == 2` 窗口）。

LLM 标题生成 MUST 异步执行，MUST NOT 阻塞 `turn_complete` 主链路；完成后 MUST 通知前端刷新（如 `session_title_updated` 事件）。自动命名 MUST NOT 覆盖 `title_user_edited == true` 的会话标题。

#### Scenario: 首轮写入用户首条消息

- **WHEN** 用户首条消息为「请帮我分析 SK1002 课程归档资料」且该轮对话完成
- **THEN** 会话标题更新为清洗后的该条文本（不超过存储上限），侧栏可见

#### Scenario: 首轮寒暄也写入标题

- **WHEN** 用户首条消息仅为「你好」且该轮对话完成
- **THEN** 会话标题更新为「你好」（不再保持「新会话」）

#### Scenario: 超长首条消息截断入库

- **WHEN** 用户首条消息超过 `MAX_STORED_TITLE_CHARS` 且该轮对话完成
- **THEN** 标题以截断形式持久化，侧栏以 CSS ellipsis 展示

#### Scenario: 第二轮 LLM 覆盖第一轮标题

- **WHEN** 第 1 轮完成后标题为「请分析 report.docx」，第 2 轮对话完成且 `autotitle_llm_done == false`
- **THEN** 系统异步调用 LLM 生成更短摘要标题并覆盖原标题，且 `autotitle_llm_done` 置为 true

#### Scenario: LLM 仅触发一次

- **WHEN** 第 2 轮 LLM 标题已成功写入，用户继续第 3 轮及以后对话
- **THEN** 标题不再自动更新

#### Scenario: 超过两轮的历史会话不补跑

- **WHEN** 升级前会话已有 3 条及以上 user 消息，用户继续发送新消息
- **THEN** 系统不调用 LLM 生成标题

#### Scenario: 用户手动标题跳过 LLM

- **WHEN** 用户在第 2 轮前手动修改标题（`title_user_edited == true`）
- **THEN** 第 2 轮不调用 LLM，标题保持不变

#### Scenario: LLM 失败保留第一轮标题

- **WHEN** 第 2 轮 LLM 请求超时或失败
- **THEN** 保留第 1 轮清洗标题，不向用户展示错误；第 3 轮及以后 MUST NOT 重试 LLM

#### Scenario: 已有非默认标题不覆盖（用户手动）

- **WHEN** 会话标题已被用户手动设为「课程资料」（`title_user_edited == true`）
- **THEN** 系统不自动修改该标题

### Requirement: 会话标题状态字段

系统 SHALL 在 `sessions` 表持久化 `autotitle_llm_done` 与 `title_user_edited` 布尔字段。用户通过 `update_session` 修改标题时 MUST 将 `title_user_edited` 置为 true。数据库迁移 MUST 对已有 `user_message_count >= 2` 的会话将 `autotitle_llm_done` 设为 true，以避免历史数据误触发 LLM。

#### Scenario: 迁移标记历史会话

- **WHEN** 应用升级并完成 migration
- **THEN** 已有 ≥2 条 user 消息的会话 `autotitle_llm_done == true`

#### Scenario: 手动改标题标记

- **WHEN** 用户通过 `update_session` 修改会话标题
- **THEN** `title_user_edited == true`

