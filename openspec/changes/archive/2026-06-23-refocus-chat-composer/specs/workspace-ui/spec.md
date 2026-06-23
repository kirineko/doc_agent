## ADDED Requirements

### Requirement: Chat 输入区焦点策略

系统 SHALL 使 Chat composer（textarea）在回合结束、切换会话后保持聚焦，无需用户手动点击。当 Settings/Credentials Drawer、图片预览、斜杠或 @ 弹层、Model Flyout、应用更新遮罩打开，或 composer 不可编辑、未选项目时，系统 MUST NOT 自动 refocus。

#### Scenario: 回合结束后自动聚焦

- **WHEN** 用户提交消息且 Agent 回合结束，composer 从 disabled 恢复为可编辑，且无 Overlay 抑制条件
- **THEN** textarea 获得焦点，用户可直接键入下一条消息

#### Scenario: 切换会话后自动聚焦

- **WHEN** 用户在侧栏选择或切换到另一会话，且 composer 可编辑、无 Overlay 抑制
- **THEN** textarea 获得焦点，用户可直接在该会话输入

#### Scenario: 文件导入完成后不强制聚焦

- **WHEN** 用户通过 composer 导入文件，`importing` 从 true 变为 false，且导入流程已通过 `onFocusInput` 设置光标位置
- **THEN** 系统 MUST NOT 因 `composerDisabled` 恢复而额外 refocus 至 `(0, 0)` 重置光标

#### Scenario: Overlay 打开时不聚焦

- **WHEN** Settings 或 Credentials Drawer 打开，或图片预览、Model Flyout、更新遮罩展示中
- **THEN** 系统 MUST NOT 因回合结束或切换会话而 refocus textarea

#### Scenario: 澄清进行中不聚焦

- **WHEN** session 存在 pending clarify 且 composer 为 disabled
- **THEN** 系统 MUST NOT refocus textarea

#### Scenario: IME 组合中 Enter 确认候选词不误发消息

- **WHEN** 用户在 composer 内使用输入法组合输入，按下 Enter 确认候选词（`isComposing` 为 true 或 `keyCode === 229`）
- **THEN** 系统 MUST NOT 触发发送，MUST NOT 执行 mention/斜杠弹层选择或删除占位符；该按键 MUST 交由浏览器/输入法原生处理
