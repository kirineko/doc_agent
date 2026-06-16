## 1. 数据库与类型

- [x] 1.1 `sessions` 表 migration：新增 `autotitle_llm_done`、`title_user_edited`；对 `user_message_count >= 2` 的会话回填 `autotitle_llm_done = 1`
- [x] 1.2 更新 Rust `Session` 结构与 `store` CRUD；`update_session` 改 title 时设 `title_user_edited = true`
- [x] 1.3 同步 `src/types.ts` 与 IPC 序列化（若字段暴露给前端）

## 2. 标题生成逻辑（Rust）

- [x] 2.1 瘦身 `session_title.rs`：`is_default_session_title`、`normalize_first_turn`、`truncate_for_storage`（120 字）；删除启发式
- [x] 2.2 新增 `title_gen.rs`：LLM 总结前两轮（session 模型、thinking off、timeout）；单测 mock/解析
- [x] 2.3 重写 `maybe_autotitle_session`：user_count 1/2/≥3 三分支；第 2 轮 spawn 异步
- [x] 2.4 新增 `AgentEvent::SessionTitleUpdated`；LLM 完成后 emit
- [x] 2.5 更新/替换 `session_title.rs` 与 `loop_support` 相关测试；删除过时启发式用例

## 3. Agent loop 集成

- [x] 3.1 确认 `loop_runner` turn_complete 路径调用新 autotitle（含 tool 多步后最终 complete）
- [x] 3.2 更新 `agent-loop` 相关 Rust 测试（若有）

## 4. 前端

- [x] 4.1 `SessionList`：保留 `truncate`，加 `title` tooltip；移除对 18 字截断假设
- [x] 4.2 `useWorkspace` / `agentEvents`：处理 `session_title_updated`，刷新 sessions 列表
- [x] 4.3 `formatTitle.test.ts`（若有）与展示相关测试

## 5. 验证

- [x] 5.1 `cargo test` + `npm run typecheck && npm test && npm run build` 通过
- [x] 5.2 手动验证：第 1 轮写首条文本 → 第 2 轮 LLM 覆盖 → 第 3 轮不变；历史 ≥3 轮不触发；侧栏截断与 tooltip
