## Why

`add-clarify-skill` 已实现基于文本对话的需求澄清流程，但用户仍需在输入框自由打字作答，缺少结构化选项、排版/样式类问题的可视化交互，且回答无法以统一格式回传给 Agent。需要将 clarify 与专用 tool + 前端交互卡片结合，并在 Agent loop 中支持「等人作答」的暂停与恢复。

## What Changes

- **新增** `clarify_ask` 工具：Agent 通过结构化 schema 出题，触发前端交互卡片
- **新增** Agent loop 暂停/恢复机制：`clarify_ask` 调用后 loop 提前结束（不 emit `turn_complete`），用户作答后 `resume_turn` 继续同一 `turn_id`
- **新增** 持久化 `clarify_pending` 状态：刷新/重启后可恢复未答问题
- **新增** IPC：`submit_clarify_answer`（必选）、`cancel_clarify`（可选 MVP）
- **新增** AgentEvent：`clarify_question`、`turn_awaiting_user`
- **新增** 前端 `ClarifyQuestionCard`：支持 single / multi / text / confirm_brief 四种题型，均支持用户自定义输入；已答题以只读卡片嵌入消息流（答案以 tool result 为唯一载体，不写 user 消息，避免破坏 OpenAI 消息序列与 user 统计）
- **更新** `clarify/SKILL.md`：澄清问题 MUST 通过 `clarify_ask` 出题，禁止纯文本问卷
- **更新** 前端 busy/pending 状态：澄清进行中 suppress 推荐问，前后端双重阻断绕过卡片直接发送
- **更新** Mock Provider：新增 clarify 关键词场景，支持无 Key 端到端测试

## Capabilities

### New Capabilities

- `clarify-interaction`：`clarify_ask` 工具 schema、pending 持久化、loop 暂停/恢复、IPC 与 AgentEvent、前端交互卡片及回答回传

### Modified Capabilities

- `agent-loop`：支持 human-in-the-loop 断点（`awaiting_user` tool call 状态、`turn_awaiting_user`、`resume_turn`）
- `workspace-ui`：澄清问题卡片 UI、pending 状态下的输入约束、用户回答写入对话流
- `clarify-skill`：必须通过 `clarify_ask` 出题；创作简报确认使用 `confirm_brief` 题型

## Impact

- `src-tauri/src/tools/clarify.rs`（新）、`registry.rs`
- `src-tauri/src/agent/loop_runner.rs`（pause/resume 分支）
- `src-tauri/src/core/store.rs`（`clarify_pending` 表 + tool_call status 扩展）
- `src-tauri/src/ipc/mod.rs`（新 command）
- `src-tauri/src/agent/types.rs`（新 AgentEvent）
- `src-tauri/assets/skills/clarify/SKILL.md`
- `src/types.ts`、`src/components/ClarifyQuestionCard.tsx`、`src/hooks/useWorkspace.ts`
- 无新外部依赖
