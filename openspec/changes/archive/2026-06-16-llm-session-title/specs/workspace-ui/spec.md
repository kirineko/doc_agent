## ADDED Requirements

### Requirement: 侧栏会话标题动态截断展示

系统 SHALL 在侧栏会话列表中展示完整持久化标题，并通过 CSS 文本溢出（`truncate` / `text-overflow: ellipsis`）按当前侧栏宽度动态截断；MUST 为标题元素提供 `title` 属性或等效 tooltip 以展示全文。展示前 MAY 调用 `plainSessionTitle` 去除存量 Markdown 标记。MUST NOT 依赖后端 18 字符固定截断作为唯一展示来源。

#### Scenario: 窄侧栏截断

- **WHEN** 用户收窄侧栏且会话标题较长
- **THEN** 标题在可视区域内 ellipsis 截断，hover 可见完整文本

#### Scenario: 宽侧栏展示更多

- **WHEN** 用户拉宽侧栏
- **THEN** 同一条标题可视字符数增加，无需重新请求后端
