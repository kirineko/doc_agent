## Context

`clarify` skill 已定义逐问澄清流程与问题库，但当前实现依赖 assistant 纯文本提问 + 用户自由输入。本变更引入 **专用 tool + loop 暂停/恢复**，与 OpenAI function calling 的 human-in-the-loop 模式一致。

当前 `loop_runner` 假设所有 tool 在单轮内同步完成；`clarify_ask` 需要等待用户在前端交互后才能写入 tool result 并继续 loop。

关键协议约束（review 确认）：

- `messages_from_store` 按 `seq` 原样重建消息发给 API；assistant 的 `tool_calls` MUST 被 tool messages 立即应答，中间不得插入其他 role 消息，否则 400
- `user_count`（autotitle）、`session_has_chat_messages`（模型锁）、followup prompt、前端 `countChatMessages` 均统计 `role='user'` 消息

## Goals / Non-Goals

**Goals:**

- 新增 `clarify_ask` tool，Agent 以结构化 schema 出题
- loop 在 `clarify_ask` 后暂停（emit `turn_awaiting_user`，不 emit `turn_complete`）
- 用户在前端选择/自定义作答 → `submit_clarify_answer` → `resume_turn` 继续同一 `turn_id`
- 支持 single / multi / text / confirm_brief 四种题型，均支持自定义输入
- pending 状态持久化，刷新后可恢复

**Non-Goals:**

- Visual Companion / 浏览器 mockup
- 多题并行（同一 session 同时仅一条 pending clarify）
- 澄清流程与 skill 触发条件的重写（沿用现有 clarify skill）

## Decisions

### D1：`clarify_ask` 为 UI gate tool，handler 不阻塞

handler 仅校验 args；暂停逻辑由 **loop_runner** 检测 tool 名后走专用分支，不立即 `finish_tool_call`。

**备选**：handler 内 block 等待用户 → 占住 tokio task，拒绝。

### D2：澄清答案不写 user message，以 tool result 为唯一载体

**Review 修订**（原方案写 user message 有两个致命问题：①打断 assistant→tool 序列导致 API 400；②污染 user_count / 模型锁 / followup / countChatMessages 等统计）。

- 答案只写入 clarify tool_call 的 `result_json` 与对应 tool message
- 前端从 `list_messages` bundle 的 `ToolCallRecord`（`name=clarify_ask`）渲染已答卡片：`args_json`=问题，`result_json`=答案，嵌入消息流展示
- 消息序列保持 `assistant(tool_calls) → tool(result) → assistant(...)`，协议干净

### D3：暂停 = 提前 `return Ok(())`，恢复 = 新函数 `resume_turn`

`run_turn` 处理含 `clarify_ask` 的轮次：

1. persist assistant + 全部 tool_calls（与现状一致，初始 status=`running`）
2. **先按序执行所有非 clarify 工具**，正常写 result / tool message
3. 第一个 `clarify_ask`：校验 args → 改 status=`awaiting_user`、写 `clarify_pending`、emit `ToolCall`（status=`awaiting_user`）、emit `clarify_question`
4. 多余的 `clarify_ask`（若有）：立即写结构化错误 result（`一次只允许一个澄清问题`），不进入 pending
5. emit `turn_awaiting_user`，`return Ok(())`（**不** emit `turn_complete`，**不**执行 `cleanup_skill_run_tmp`——turn 未结束）

> 暂停时刻唯一缺 tool result 的就是 pending 那一条；submit 补齐后整个 assistant 轮的 tool_calls 全部有应答，resume 重建合法。

`submit_clarify_answer`（IPC）：

1. 事务内 `DELETE FROM clarify_pending WHERE session_id=?`；删除 0 行 → 返回「已处理或不存在」错误（防双 submit）
2. 校验 answer 与 `question_json` 匹配（kind、option id、multi 边界、custom 非空性）
3. **后端**组装 `display_text`（option label + custom，前端不传）与 tool result JSON
4. `finish_tool_call`（status=`done`）+ 写 tool message
5. emit `ToolResult`
6. 调用 `resume_turn(session_id, turn_id)`

`resume_turn`：

- 从 DB 全量重建 working_messages（`messages_from_store` + system prompt 注入），**不追加新 user message**
- 在同一 `turn_id` 下继续 `for _step in 0..MAX_TOOL_STEPS`（每次 resume 重置步数预算，可接受）
- autotitle 所需 `user_text` 取 history 中最后一条 `role='user'` 消息
- 直至无 tool_calls（emit `turn_complete`）或再次 clarify 暂停

**turn_id 沿用**：同一用户原始请求视为一个 logical turn，工具链 UI 连贯。

### D4：`clarify_pending` 表（SQLite）

| 列 | 说明 |
|----|------|
| session_id | PK，每 session 最多一条 |
| turn_id | resume 沿用 |
| tool_call_id | 对应 tool call id |
| question_json | 完整 ClarifyQuestion |
| created_at | 超时/清理 |

- 仅供**后端** resume 查询与单 pending 约束；`CREATE TABLE IF NOT EXISTS` 即可，无需迁移框架
- `tool_calls.status` 无 CHECK 约束，新增值 `awaiting_user` 无需 schema 变更
- **前端恢复不需要新 IPC**：`list_messages` bundle 中 `status=awaiting_user` 的 clarify_ask 记录即可还原活跃卡片

### D5：ClarifyQuestion schema

```typescript
type ClarifyKind = "single" | "multi" | "text" | "confirm_brief";

interface ClarifyOption {
  id: string;
  label: string;
  hint?: string;
}

interface ClarifyQuestion {
  id: string;
  kind: ClarifyKind;
  prompt: string;
  description?: string;
  options?: ClarifyOption[];      // single/multi 必填（2–6 项）
  allow_custom?: boolean;          // 默认 true
  custom_label?: string;           // 默认「其他」
  custom_placeholder?: string;
  min_selections?: number;         // multi
  max_selections?: number;
  brief?: Record<string, string>;  // confirm_brief 必填
}
```

**提交请求（前端 → 后端）：**

```json
{ "session_id": "s1", "question_id": "ppt_style", "selected": ["business_dark"], "custom": null }
```

**Tool result（后端组装，给 LLM）：**

```json
{
  "question_id": "ppt_style",
  "selected": ["business_dark"],
  "custom": null,
  "display_text": "商务深色"
}
```

confirm_brief 的「确认」= `selected: ["confirm"]`；「修改」= `custom` 携带修改意见。

### D6：前端交互规则

| kind | UI | 自定义 |
|------|-----|--------|
| single | 选项卡片 + 选中高亮 | 「其他」展开 textarea |
| multi | 多选 chip（min/max 校验） | 「添加自定义」输入 |
| text | textarea + 可选快捷 chip | 主路径即自定义 |
| confirm_brief | 字段预览 +「确认继续」/「需要修改」 | 修改走 textarea |

状态机：

- `clarify_question` 事件或 bundle 恢复 → 展示活跃卡片，`busy=false`
- submit → 卡片切换为已答态（乐观），`busy=true`，等待 resume 流式输出
- 已答卡片随消息流持久展示（数据源：ToolCallRecord）

pending 时：

- suppress `SuggestionCards`
- `send_message` **前后端双重拦截**：前端提示「请先回答上方澄清问题」；后端入口检查 pending 存在则拒绝
- 工具链面板 clarify 卡片显示「等待回答」态（status=`awaiting_user`），submit 后转 done

### D7：Skill 与 system prompt

- `clarify/SKILL.md`：每问 MUST 调用 `clarify_ask`（一次一问）；创作简报 MUST `kind=confirm_brief`；禁止 assistant 纯文本列选项
- system prompt 仅保留：「需求不明确时 skill_read clarify；澄清 MUST 用 clarify_ask」

### D8：Mock provider 支持

`mock.rs` 新增关键词场景（如用户文本含「澄清」）：返回一个 `clarify_ask` tool call，用于无 API Key 的开发与集成测试。

### D9：cancel（MVP 可选）

`cancel_clarify`：同 submit 流程，tool result 写 `{ "cancelled": true }`，resume 后由 Agent 决定结束澄清或改用默认值。

## Risks / Trade-offs

- [模型一次返回多个 clarify_ask] → 仅第一个进入 pending，其余写错误 result；skill 同时约束一次一问
- [用户刷新 mid-clarify] → bundle 中 `awaiting_user` 记录还原卡片；pending 表保证 resume 可用
- [双 submit / send 竞态] → 事务内 delete pending 原子判定 + send_message 后端 pending 检查
- [resume 重置步数预算] → 接受；恶意循环仍受每段 32 步限制
- [实现复杂度] → MVP 含全部四种题型（问题库已有多选场景），cancel 可后置

## Migration Plan

1. store：`clarify_pending` 表（`CREATE TABLE IF NOT EXISTS`）+ CRUD；`tool_calls.status` 直接使用新值
2. 后端 tool + loop 暂停分支 + `resume_turn` + IPC
3. Mock provider clarify 场景
4. 前端卡片 + event + 恢复逻辑
5. 更新 clarify skill
6. 测试：Rust loop 集成测试（pause → submit → resume → complete）+ 前端组件测试

## Open Questions

- pending 超时策略（如 24h 自动 cancel）→ 后续变更
- 已答卡片在消息流中的精确插入位置（按 assistant 消息 seq 对齐）→ 实现时定，不影响契约
