## 1. 事件契约（Rust + TS）

- [x] 1.1 `AgentEvent::ToolCall` 增加 `index` 字段；`loop_runner` / Mock provider emit 时填充
- [x] 1.2 前端 `types.ts` 与 `agentEvents.ts` 处理 `index`；`tool_call` 按 index 就地升级 `streaming-{index}`

## 2. loop_runner 执行策略

- [x] 2.1 工具执行前 broadcast 本轮全部 `ToolCall { running }`（clarify 路径不变）
- [x] 2.2 提取执行逻辑：同轮 `pdf_read` 用 `Semaphore(3)` 并行，结果按原序 persist；其他工具串行

## 3. 测试

- [x] 3.1 前端 `agentEvents.test.ts`：3 个 streaming 占位 + 3 个 tool_call 不减少卡片数
- [x] 3.2 Rust：`loop_runner` 或独立模块测试 pdf_read 并行上限与结果顺序

## 4. 验证

- [x] 4.1 `cargo test` + `npm test` + `npm run typecheck` 通过
