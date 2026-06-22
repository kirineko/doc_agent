## 1. 斜杠命令注册表

- [x] 1.1 在 `slashCommands.ts` 新增 `ppt:edit-ooxml` 模板（精准修改 PPT，OOXML prompt）
- [x] 1.2 更新 `ppt:edit` 的 label/description/prompt 为脚本编辑路径
- [x] 1.3 在 `SLASH_COMMAND_ENTRIES` 新增 `compact` command（与 `init` 同组）
- [x] 1.4 更新 `slash.test.ts`：模板 23 条、总条目 25、`ppt:edit-ooxml` prompt 长度与模糊搜索

## 2. 压缩核心与 IPC

- [x] 2.1 从 `compact_session_if_needed` 抽取 `compact_session_core`（`skip_threshold`、`on_prepare_none` 参数）
- [x] 2.2 实现 `force_compact_session` / `compact_session` IPC 与响应类型
- [x] 2.3 `AgentEvent::ContextCompacted` 增加 `trigger: auto | manual`；自动路径 emit `trigger: auto`
- [x] 2.4 `compaction_tests.rs`：手动跳过阈值、prepare=None no-op、与自动共享摘要路径
- [x] 2.5 IPC 契约测试（`compact_session` 成功 / nothing_to_compact / turn running 拒绝）

## 3. 前端发送与 UI

- [x] 3.1 新增 `isCompactMessage`（`src/lib/compactMessage.ts`）及单元测试
- [x] 3.2 `useWorkspace.sendMessageContent`：`/compact` 分支 invoke `compact_session`，不写 optimistic user 消息
- [x] 3.3 `types.ts` / `agentEvents.ts`：`context_compacted.trigger`；按 trigger 区分 compaction notice 文案
- [x] 3.4 `agentEvents.test.ts`：manual trigger 文案
- [x] 3.5 `ChatPanel`：`compact` command 在 clarify pending 时阻断（对齐 `init`）；turn running 阻断
- [x] 3.6 `compact_session` 返回 `compacted: false` 时 toast「无需压缩」

## 4. 验证

- [x] 4.1 `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [x] 4.2 `npm run typecheck && npm test && npm run build`
