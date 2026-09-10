## ADDED Requirements

### Requirement: 工具错误详情折叠展示

工具链卡片在 `tool_result.ok=false` 且 `summary` 为 JSON 时，MUST 以 `error`（或 `message`）为醒目标题行，并在其下提供可折叠的 `detail` 区域（默认收起）展示 `detail` 字段全文；`detail` 超过 600 字符时 MUST 截断并保留末尾 200 字符（中间以 `…` 标示）。`hint` 字段存在时 SHOULD 以次级样式展示在 `detail` 之下。现有 `file_busy` 展示行为不变。

#### Scenario: 超时错误展示

- **WHEN** `skill_run` 返回 `{"error":"script timeout","detail":"script timeout","hint":"脚本在 120s 内未完成…"}`
- **THEN** 卡片标题行为 `script timeout`，展开后可见 `hint` 文案

#### Scenario: 运行时异常展示 detail

- **WHEN** `skill_run` 返回 `{"error":"JavaScript runtime error","detail":"TypeError: slide.addImage is not a function\n  at main (script.js:12)"}`
- **THEN** 卡片标题行为 `JavaScript runtime error`，展开 detail 后可见 `TypeError` 与行号

#### Scenario: 非 JSON summary 回退

- **WHEN** `tool_result.summary` 不是 JSON
- **THEN** 卡片按既有逻辑展示截断后的纯文本，无折叠区
