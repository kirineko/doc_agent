## Context

- 自动标题在 `turn_complete`（无 tool_calls 返回路径）调用 `maybe_autotitle_session` → `summarize_session_title`（~400 行启发式 + 18 字 `truncate_title`）。
- 仅 `user_count ∈ {1,2}` 且标题为默认时触发；第 2 轮不用助手回复。
- 前端 `SessionList` 已有 CSS `truncate`；`plainSessionTitle` 剥离存量 Markdown。
- 独立 LLM 短请求先例：`agent/suggest.rs`（DeepSeek、thinking off、timeout、JSON 输出）。
- 侧栏宽度已可拖拽（`resize-workspace-panels`），固定字符截断不再合理。

## Goals / Non-Goals

**Goals:**

- 第 1 轮：默认标题 → 清洗后的首条 user 文本入库（超长按存储上限截断）
- 第 2 轮：当前 session 模型、thinking off，LLM 总结**前两轮**对话生成标题，**覆盖**第 1 轮标题；**仅 1 次**
- 第 3 轮+及历史 ≥3 轮会话：永不自动改名、不补跑 LLM
- 用户手动改标题后：跳过第 2 轮 LLM
- 前端：存储完整标题 + 侧栏 CSS ellipsis + tooltip

**Non-Goals:**

- 每轮更新标题
- 第 3 轮及以后补跑 LLM（含历史会话 backfill 命名）
- 侧栏内联编辑标题 UI（可后续独立 change）
- 改用固定 DeepSeek 而非 session 模型

## Decisions

### D1：两轮触发窗口（取代启发式）

| `user_count` at turn_complete | 条件 | 动作 |
|-------------------------------|------|------|
| 1 | `is_default_session_title(title)` | `title = normalize_first_turn(user_text)`，截断至 `MAX_STORED_TITLE_CHARS` |
| 2 | `!autotitle_llm_done && !title_user_edited` | 异步 LLM → 覆盖 title → `autotitle_llm_done = true` |
| ≥3 | — | 跳过 |

**说明**：第 1 轮结束后标题已非默认，第 2 轮**不能**用 `is_default_session_title` 作为 LLM 条件。

### D2：DB 字段

`sessions` 表新增：

| 列 | 类型 | 默认 | 含义 |
|----|------|------|------|
| `autotitle_llm_done` | INTEGER (bool) | 0 | 第 2 轮 LLM 标题是否已执行 |
| `title_user_edited` | INTEGER (bool) | 0 | 用户手动改过标题（`update_session` 改 title 时置 1） |

**迁移**：对已有会话，若 `user_message_count >= 2`（按 messages 表统计），设 `autotitle_llm_done = 1`，避免升级后第 3+ 轮误触发。

### D3：存储 vs 展示截断

- `MAX_STORED_TITLE_CHARS = 120`（应用层常量，SQLite TEXT 无硬限）
- `normalize_first_turn`：去 Markdown/空白，**不**做 18 字截断
- 侧栏展示：`truncate` + `title={full}` tooltip

### D4：LLM 标题生成（`title_gen.rs`）

仿 `suggest.rs`：

```text
ChatRequest {
  model: session.model,
  thinking: { enabled: false, ... },
  tools: [],
  max_tokens: 64,
  messages: [system, user_prompt_with_round1_and_round2_snippets]
}
```

- Prompt：单行标题，≤40 字建议，无引号/Markdown，概括前两轮意图
- 输入：前两轮 user + assistant 文本 snippet（每段上限 ~500 字，仿 followup）
- 超时 15s；失败静默，保留第 1 轮标题
- **异步** `tokio::spawn`，不阻塞 `turn_complete` emit

### D5：事件与前端刷新

新增 `AgentEvent::SessionTitleUpdated { session_id, title }`。

- LLM 完成后 emit + `update_session`
- 前端 `useWorkspace`：patch `sessions` 列表或触发 `list_sessions`
- 第 1 轮同步写入仍依赖现有 `turn_complete` → `list_sessions`

### D6：删除/保留模块

| 模块 | 动作 |
|------|------|
| `session_title.rs` | 删除启发式；保留 `is_default_session_title`、`normalize_first_turn`、`truncate_for_storage` |
| `title_gen.rs` | 新增 |
| `loop_support.rs` | `maybe_autotitle_session` 重写为上述三分支 |
| `formatTitle.ts` | 保留 `plainSessionTitle` 兼容旧数据 |

### D7：`resume_turn` autotitle

autotitle 仍在该 turn 的 `turn_complete` 时执行；`user_count` 以 DB 中 user 消息数为准（与 resume 无关）。删除 spec 中「以最后一条 user 为 user_text 依据」的启发式描述，改为两轮窗口 + `user_count`。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 第 1 轮「你好」作标题 | 第 2 轮 LLM 覆盖 |
| LLM 延迟 | 异步 + 事件；不拖 turn_complete |
| 额外 API 成本 | 每会话最多 1 次；max_tokens 小 |
| 历史 2 条消息仍「新会话」 | 迁移设 `autotitle_llm_done`；不 backfill LLM |
| 第 2 轮覆盖第 1 轮长标题 | 符合产品确认 |

## Migration Plan

1. Schema migration 增加两列 + 回填 `autotitle_llm_done`
2. 部署新逻辑；旧启发式测试删除/替换
3. 回滚：还原 migration + 旧 `session_title.rs`（标题数据无需改）

## Open Questions

（无——两轮策略、LLM 单次、第 2 轮覆盖第 1 轮、存储 120 字均已确认）
