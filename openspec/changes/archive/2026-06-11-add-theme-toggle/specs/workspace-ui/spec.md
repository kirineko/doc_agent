## ADDED Requirements

### Requirement: 顶栏主题切换 Toggle

系统 SHALL 在应用顶栏右上角提供主题切换 **toggle** 控件，用于在 `dark` 与 `light` 两档主题间切换；该控件 MUST 位于顶栏最右侧（`Doc Agent` 品牌与项目名区域保持在左侧），且 MUST NOT 遮挡或替换现有 Logo 与标题展示。

#### Scenario: 顶栏右上角展示 Toggle

- **WHEN** 用户打开应用主窗口
- **THEN** 顶栏右侧可见主题 toggle，左侧仍显示定制 Logo 与「Doc Agent」文字

#### Scenario: 点击 Toggle 切换主题

- **WHEN** 用户点击顶栏主题 toggle
- **THEN** 应用主题在 `dark` 与 `light` 间立即切换，toggle 视觉状态对应当前主题

#### Scenario: Toggle 可访问性

- **WHEN** 辅助技术聚焦主题 toggle
- **THEN** 控件具备描述当前操作或目标主题的 accessible 名称（如 `aria-label`），且可通过键盘激活
