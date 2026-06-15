## Context

同一轮 assistant 可返回多个 `tool_call`。SSE 流式阶段通过 `tool_call_stream` 为每个 index 创建 `streaming-{index}` 占位；执行阶段 `loop_runner` 逐个 emit `ToolCall { status: running }` 并 await handler。

前端 `agentEvents.ts` 在收到首个 `tool_call` 时用 `filter(!streaming-*)` 删除**全部** streaming 占位，导致 3 个 `pdf_read` 同批时 UI 从 3 张卡片骤减为 1 张。后端对 `pdf_read` 串行执行，读 3 份 PDF 时总耗时为 3 倍。

## Goals / Non-Goals

**Goals:**

- 同批多工具 streaming→running 过渡时，工具链卡片数量不骤减、不出现整栏清空感
- `ToolCall` 事件与 `tool_call_stream` 通过 `index` 精确对应
- 执行前 broadcast 本轮全部 running 状态
- 同轮 `pdf_read` 并行执行，并发上限 3，DB / working_messages 顺序不变
- clarify 混合批次与非 `pdf_read` 工具保持现有串行语义

**Non-Goals:**

- 跨多步 loop 的全局并发池
- 其他只读工具（`office_read`、`web_search` 等）并行
- 修改 Provider SSE 解析逻辑（除 emit 字段外）
- 工具链展示耗时/结果摘要（已有能力不变）

## Decisions

### D1：ToolCall 事件携带 index

在 `AgentEvent::ToolCall` 增加 `index: usize`，与 OpenAI SSE `tool_calls[].index` 及前端 `streaming-{index}` 对齐。

**备选**：用 tool name + 参数 hash 匹配 — 同批多个同名 `pdf_read` 无法区分，放弃。

### D2：前端就地升级 streaming 占位

`tool_call` handler 优先查找 `streaming-{event.index}`，将其 id 替换为真实 `call_id`、status 改为 running、写入 args；若无对应占位则 append。不再 bulk filter 所有 streaming。

**备选**：ToolChainPanel 用 slot index 作 React key — 可作为补充，但状态机修复是根因。

### D3：两阶段工具执行（broadcast → execute）

`loop_runner` 在 parse 完本轮全部 calls 后：

1. 先 emit 全部非 clarify-pending 的 `ToolCall { running }`（含 index）
2. 再执行：普通工具串行；连续 `pdf_read` 段用 `Semaphore(3)` 并行

clarify 路径仍在 broadcast 阶段单独 emit `awaiting_user`，执行阶段逻辑不变。

### D4：pdf_read 并行范围与顺序

仅对**同一轮** `tool_calls` 列表中**连续**的 `pdf_read` 子序列并行（实际上同批多个 pdf_read 即连续）。执行完成后按原始下标顺序 persist result 与 push tool message。

使用 `tokio::sync::Semaphore::new(3)` + `futures::future::join_all` 或手动 spawn 带 permit 的 task。非 pdf_read 打断连续段时，前段并行、中间串行、后段再并行。

**备选**：全部 pdf_read 无视位置并行 — 可能破坏 `[fs_write, pdf_read, pdf_read]` 中 write 必须先完成的隐含顺序，不采用。

### D5：Mock Provider 同步 index

Mock 单工具 emit 时 `index: 0`，保证测试与前端契约一致。

## Risks / Trade-offs

- **[Risk] 并行 pdf_read 争用 PDFium / 磁盘** → 上限 3；只读操作，无写冲突
- **[Risk] vision 子调用 API 限流** → Semaphore 与 max 3 一致；Judge 失败时仍保守全量 vision
- **[Risk] index 缺失的旧事件** → 后端始终填充；前端 fallback 为 append + 删首个 streaming（兼容）

## Migration Plan

1. 后端 types + loop_runner 先发版（新字段 `index` 对旧前端无害，serde default 0）
2. 前端 agentEvents 升级
3. 跑 Rust + Vitest 全量
4. 归档 change 合并 spec delta

无数据迁移；无 breaking IPC。

## Open Questions

（无 — explore 阶段已确认复现条件为同轮 3× `pdf_read`）
