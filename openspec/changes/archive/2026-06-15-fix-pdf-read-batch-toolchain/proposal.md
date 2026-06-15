## Why

当模型在同一轮 assistant 响应中同时发起 3 个 `pdf_read` 时，右侧工具调用链会在参数生成完成后「整栏闪空再重建」：首个工具开始执行时前端误删全部 `streaming-*` 占位，仅显示 1 张 running 卡片。与此同时，3 个 `pdf_read` 在 `loop_runner` 中串行执行，读多份 PDF 时耗时过长。需在修复 UI 状态机的同时，对同轮 `pdf_read` 启用有限并行（上限 3）。

## What Changes

- `ToolCall` Agent 事件新增 `index` 字段，用于与 `tool_call_stream` 占位一一对应
- 前端 `agentEvents.ts`：`tool_call` 到达时按 `index` 就地升级对应 `streaming-{index}`，不再批量删除所有 streaming 占位
- `loop_runner.rs`：工具执行前先 broadcast 本轮全部 `tool_call { running }` 事件
- `loop_runner.rs`：同轮多个 `pdf_read` 并行执行，全局并发上限 3；结果 persist 顺序与原始 `tool_calls` 顺序一致
- 非 `pdf_read` 工具、含 `clarify_ask` 的混合批次保持现有串行语义
- 补充 Rust / 前端单测

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-loop`：同轮 `pdf_read` 最大并行 3；工具执行前 broadcast running 事件；`ToolCall` 事件携带 `index`
- `workspace-ui`：多工具同批 streaming→running 过渡不得整栏清空；按 index 平滑升级占位卡片

## Impact

- Rust：`src-tauri/src/agent/types.rs`、`loop_runner.rs`、Mock provider 事件序列
- 前端：`src/types.ts`、`src/lib/agentEvents.ts` 及测试
- OpenSpec：`openspec/specs/agent-loop/spec.md`、`openspec/specs/workspace-ui/spec.md`（归档时合并）
