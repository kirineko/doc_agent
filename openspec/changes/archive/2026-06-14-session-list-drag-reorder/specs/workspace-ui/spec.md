## ADDED Requirements

### Requirement: 侧栏会话列表拖动排序

系统 SHALL 允许用户在当前项目的侧栏会话列表中通过拖动调整会话展示顺序。拖动 MUST 使用独立 drag handle，不得与点击选中、删除按钮冲突。排序范围 MUST 限定于当前 activeProject 下的会话。

#### Scenario: 拖动手柄重排

- **WHEN** 用户按住某会话项左侧 drag handle 并拖动到新位置后释放
- **THEN** 会话列表立即按新顺序展示

#### Scenario: 拖动不影响选中

- **WHEN** 用户点击会话标题区域（非 drag handle）
- **THEN** 选中该会话并加载消息，不触发拖动

#### Scenario: 删除按钮仍可用

- **WHEN** 用户 hover 会话项并点击删除
- **THEN** 删除该会话，不触发拖动

### Requirement: 会话列表顺序懒激活与前端持久化

系统 SHALL 按项目隔离持久化会话展示顺序于前端 `localStorage`。某项目**从未**被用户拖动排序时，列表 MUST 按后端 `updated_at` 降序展示（与改动前一致）。用户在某项目下**首次**完成拖动排序后，该项目 MUST 进入手动序模式：顺序写入 `localStorage` 并在应用重启后恢复。手动序模式下，后端刷新会话元数据（如 `turn_complete` 后 `list_sessions`）MUST NOT 改变用户设定的展示顺序。

#### Scenario: 未拖动时保持自动序

- **WHEN** 用户在某项目下从未拖动排序，且某会话因新消息导致 `updated_at` 更新
- **THEN** 该会话在列表中按 `updated_at` 规则上移（与改动前一致）

#### Scenario: 首次拖动激活手动序

- **WHEN** 用户在某项目下首次完成拖动排序
- **THEN** 顺序写入 `localStorage` 并立即生效；此后该项目不再因 `updated_at` 变化自动重排

#### Scenario: 重启后恢复手动序

- **WHEN** 用户曾拖动排序并重启应用
- **THEN** 打开同一项目时会话列表按上次保存的顺序展示

#### Scenario: turn_complete 不改变手动序

- **WHEN** 项目处于手动序模式且某会话回合结束触发 `list_sessions` 刷新
- **THEN** 列表顺序保持不变，仅会话标题等元数据可更新

#### Scenario: 手动序下新建仍置顶

- **WHEN** 项目处于手动序模式且用户点击「新建」
- **THEN** 新会话出现在列表顶部，并写入持久化顺序的首位

#### Scenario: 删除会话同步顺序

- **WHEN** 项目处于手动序模式且用户删除某会话
- **THEN** 该会话从列表与持久化顺序中移除，其余顺序不变

#### Scenario: 项目隔离

- **WHEN** 用户在项目 A 拖动排序后切换到项目 B
- **THEN** 项目 B 展示其自身顺序（自动序或各自的手动序），不受项目 A 影响
