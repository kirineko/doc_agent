## ADDED Requirements

### Requirement: ToolCall 事件携带工具 index

系统 SHALL 在 `ToolCall` Agent 事件中携带 `index` 字段（非负整数），与 SSE `tool_calls[].index` 及 `tool_call_stream.index` 一致。Mock Provider MUST 同步 emit 该字段。

#### Scenario: 同批第二个工具 index 为 1

- **WHEN** 模型同轮返回两个 tool_calls 且第二个开始执行
- **THEN** 第二个 `ToolCall` 事件的 `index` 为 1

### Requirement: 工具执行前 broadcast running 状态

`loop_runner` 在 await 任意工具 handler 之前，SHALL 为本轮全部待执行工具 emit `ToolCall { status: running }`（clarify 路径 emit `awaiting_user` 的规则不变）。

#### Scenario: 三个 pdf_read 先全部 running

- **WHEN** 模型同轮返回三个 `pdf_read` 且参数校验通过
- **THEN** 前端在任一 `pdf_read` handler 开始前收到三条 `status=running` 的 `ToolCall` 事件

### Requirement: 同轮 pdf_read 有限并行

系统 SHALL 对同一轮 `tool_calls` 中的 `pdf_read` 调用并行执行，最大并发数为 3。`working_messages` 与 DB 中 tool result 的写入顺序 MUST 与原始 `tool_calls` 顺序一致。非 `pdf_read` 工具 MUST 保持串行；含 `clarify_ask` 的混合批次 MUST 遵守现有 clarify 执行顺序。

#### Scenario: 三个 pdf_read 并行不超过 3

- **WHEN** 模型同轮返回三个 `pdf_read`
- **THEN** 三个 handler 可同时处于 in-flight 状态，且全部完成后按原顺序写入 tool messages

#### Scenario: pdf_read 与 fs_list 混合仍串行 fs_list

- **WHEN** 模型同轮返回 `[fs_list, pdf_read, pdf_read]`
- **THEN** `fs_list` 完成后两个 `pdf_read` 方可并行，且 fs_list result 在 working_messages 中先于 pdf_read results

#### Scenario: clarify 批次不受影响

- **WHEN** 模型同轮返回 `[pdf_read, clarify_ask]` 且 clarify 校验通过
- **THEN** `pdf_read` 正常完成后再进入 clarify 暂停，行为与变更前一致
