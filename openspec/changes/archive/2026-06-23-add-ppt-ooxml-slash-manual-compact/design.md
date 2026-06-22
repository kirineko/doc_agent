## Context

- 斜杠注册表：`src/lib/slashCommands.ts`；`command` 组现有 `init`（填入 `/init` → `send_message` + `profile_init`）。
- PPT：`pptx/SKILL.md` 将 PptxGenJS 置于 Quick Reference 首位；`editing.md` 描述 OOXML 流程。`word:edit` 已用 prompt 锁定 OOXML。
- 压缩：`compaction.rs` 中 `compact_session_if_needed` 在达阈值后执行；`prepare_compaction_split` 保留最近 2 条 user/assistant；`prepare=None` 时自动路径走 `truncate_fallback` 并仍 emit `context_compacted`。

## Goals / Non-Goals

**Goals:**

- PPT 双路径斜杠：`ppt:edit`（脚本）与 `ppt:edit-ooxml`（OOXML 精准），prompt 与 `word:edit` 对称
- 手动 `/compact`：任意上下文占用下可尝试压缩；与自动压缩共享摘要管线
- 手动与自动持久化一致：归档旧消息 + 写入摘要 user 消息；不写 `/compact` 指令消息
- 仅两处分叉：自动阈值门；`prepare=None` 时自动 truncate_fallback vs 手动 no-op

**Non-Goals:**

- 改变自动压缩触发算法或摘要 prompt
- 压缩进度 UI、归档浏览器
- `/compact` 尾部参数

## Decisions

### 1. `ppt:edit-ooxml` 与 `ppt:edit` 分工

| id | label | prompt 要点 |
|----|-------|-------------|
| `ppt:edit-ooxml` | 精准修改 PPT | `请精准修改 {{文件名.pptx}}：{{改动说明}}。ooxml 解包改 slide XML 再回包，勿用 JS 脚本。` |
| `ppt:edit` | 脚本编辑 PPT | `请用脚本修改 {{文件名.pptx}}：{{改动说明}}。先读 pptx/pptxgenjs.md 再 skill_run。` |

增删页等结构操作：OOXML 路径通过 prompt 隐含 + Agent `skill_read pptx/editing.md`（不写入 100 字限制内的 prompt）。

### 2. 压缩核心：`compact_session_core`

抽取共享函数，参数：

```text
compact_session_core(
  skip_threshold: bool,           // 手动 true
  on_prepare_none: TruncateFallback | NoOp,  // 自动 TruncateFallback，手动 NoOp
  ...
)
```

共享步骤：`prepare_compaction_split` →（`Some`）`run_compaction_llm` → 归档 + `add_compaction_summary` → `rebuild_working_messages` → emit 事件。

LLM 失败：`truncate_fallback_compact_only(prepared.to_compact)`（两边一致）。

入口：

- `compact_session_if_needed`：`skip_threshold=false`，`on_prepare_none=TruncateFallback`
- `force_compact_session`（IPC）：`skip_threshold=true`，`on_prepare_none=NoOp`

### 3. 手动压缩不写指令消息

`/compact` **不得** `add_message`；仅 DB 层归档与摘要（与自动相同）。用户消息列表不出现 `/compact`。

自动压缩同样不写「系统指令」类 user 消息，只写摘要气泡。

### 4. `prepare=None` 时手动 no-op

当活跃消息不足 3 条 user/assistant（或仅 1 轮对话等导致 `to_compact` 为空）：

- 手动：返回 `{ compacted: false, reason: "nothing_to_compact" }`，前端 toast「当前上下文较短，无需压缩」
- 自动（已达阈值）：保持现有 `truncate_fallback` 行为

### 5. `/compact` 前端拦截

与 `send_message` 分叉，不走 Agent turn：

```text
trimmed === "/compact" → invoke("compact_session", { session_id })
```

阻断条件（与 `init` 对齐并加强）：

- `activeClarify` → 阻断
- session `running` / `stopping` → 阻断
- 无项目 / 无 API Key / 并行满 → 同 `getSendBlocker`

菜单选中 `compact` 仅填入 `/compact`，Enter 触发拦截逻辑。

### 6. `context_compacted.trigger`

Rust `AgentEvent::ContextCompacted` 增加 `trigger: "auto" | "manual"`（序列化为字符串）。

- 自动：`trigger: "auto"` →「已自动压缩较早的对话历史以节省上下文」
- 手动：`trigger: "manual"` →「已手动压缩对话历史」

`before_tokens`：手动用会话 `token_count`（无 turn 时 `pending=0`）；自动保持 `token_count + pending_estimate`。

### 7. 注册表计数

- 模板 **23** 条（+`ppt:edit-ooxml`）
- command **2** 条（`init`、`compact`）
- 测试 `slashFuzzy` 总条目 **25**

## Risks / Trade-offs

- **[Risk] 手动压缩调 LLM 消耗 API** → 用户主动操作；无操作时明确提示，不调 LLM
- **[Risk] 压缩与 turn 并发改历史** → 前端 + 后端双重校验 session 非 running
- **[Risk] `prepare=None` 自动仍 emit compacted 而实际无归档** → 本 change 不改自动行为；手动避免假阳性
- **[Trade-off] 手动无法在极短历史下 truncate** → 符合预期；用户需等更多对话或依赖自动兜底

## Migration Plan

纯增量：无数据迁移。发版后用户即可使用新斜杠命令；旧 `ppt:edit` prompt 变更仅影响新填入的模板。

## Open Questions

（无 — 讨论已闭合。）
