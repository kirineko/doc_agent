# app-ui-scale Specification

## Purpose
TBD - created by archiving change add-ui-scale. Update Purpose after archive.
## Requirements
### Requirement: 界面缩放档位与默认值

系统 SHALL 支持主窗口界面缩放，缩放因子（scale factor）默认 **1.0**（100%）。合法缩放因子 MUST 为 **0.2** 步进，且 MUST 限制在 **1.0**（100%）至 **2.0**（200%）之间。合法档位为：100%、120%、140%、160%、180%、200%。

#### Scenario: 首次启动默认 100%

- **WHEN** 用户首次打开应用且本地无已保存界面缩放偏好
- **THEN** 主窗口 UI 以 100% 缩放渲染

#### Scenario: 非法存储值回退

- **WHEN** `localStorage` 中 `doc-agent-ui-scale` 值无法解析为有限数字
- **THEN** 系统回退为 100% 并正常展示

#### Scenario: 超出范围 clamp 并 snap

- **WHEN** 持久化值为 2.5 或 0.9
- **THEN** 系统 MUST 将其 clamp 到 [1.0, 2.0] 并按 0.2 步进 snap 到最近合法档位

### Requirement: WebView 整页缩放

系统 SHALL 通过 Tauri 主窗口 WebView 的 zoom API（等效 `setZoom(scaleFactor)`）实现整页等比缩放。缩放 MUST 作用于主窗口全部 WebView 内容（顶栏、三栏工作区、Drawer、Overlay、Markdown 与弹层），MUST NOT 仅放大字体或单一面板。

#### Scenario: 放大后全局 UI 变大

- **WHEN** 用户将缩放设为 140%
- **THEN** 顶栏、侧栏、会话区与设置抽屉内文字与间距相对 100% 均 visibly 放大

#### Scenario: 隐藏导出 WebView 不受影响

- **WHEN** 用户将主窗口缩放设为 200% 且 Agent 触发 HTML/PDF 导出用隐藏 WebviewWindow
- **THEN** 隐藏 WebviewWindow MUST 保持其自身默认缩放（不因主窗口设置联动）

### Requirement: 缩放偏好持久化

系统 SHALL 将用户当前缩放因子写入浏览器 `localStorage`（键名 `doc-agent-ui-scale`）；应用重启后 MUST 自动恢复上次有效缩放并应用到主 WebView。

#### Scenario: 重启后保持 160%

- **WHEN** 用户已将缩放设为 160% 并关闭后重新打开应用
- **THEN** 主窗口以 160% 缩放启动，无需用户重新设置

### Requirement: 全局缩放快捷键

系统 SHALL 在应用层提供以下全局快捷键，并与设置 UI 使用 **相同** 的 snap 逻辑与持久化：

- ⌘/Ctrl + `=` 或 `+`：放大一个步进（+0.2），已达 200% 时 MUST NOT 继续放大
- ⌘/Ctrl + `-`：缩小一个步进（−0.2），已达 100% 时 MUST NOT 继续缩小
- ⌘/Ctrl + `0`：重置为 100%

上述快捷键 MUST NOT 与现有 ⌘/Ctrl+K、N、O 冲突。用户在输入法组合输入（`isComposing` 或 `keyCode === 229`）时 MUST NOT 触发缩放快捷键。

#### Scenario: 快捷键放大并持久化

- **WHEN** 当前缩放为 120% 且用户按下 ⌘/Ctrl + `=`
- **THEN** 缩放变为 140%，界面立即更新，且 `localStorage` 写入 1.4

#### Scenario: 已达上限不再放大

- **WHEN** 当前缩放为 200% 且用户按下 ⌘/Ctrl + `=`
- **THEN** 缩放保持 200%

#### Scenario: 重置为 100%

- **WHEN** 当前缩放为 180% 且用户按下 ⌘/Ctrl + `0`
- **THEN** 缩放变为 100% 并持久化

### Requirement: 禁用 WebView 内置 zoom 热键

系统 SHALL 在主窗口 WebView 配置中设置 `zoomHotkeysEnabled: false`，以避免 WebView 内置 zoom 热键与应用层缩放逻辑双重生效。应用层 ⌘/Ctrl ±/0 MUST 仍可用（见「全局缩放快捷键」）。

#### Scenario: 单次按键仅一级步进

- **WHEN** 用户按下 ⌘/Ctrl + `-` 一次
- **THEN** 缩放因子 MUST 恰好减少 0.2（一个步进），MUST NOT 出现一次按键触发两次缩放的跳变

