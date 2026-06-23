# project-backlog Specification

## Purpose

记录 Doc Agent 与当前实现或理想状态之间的已知差距、性能优化与产品增强项。本文件为**规划与跟踪**用途；条目实现后应通过 OpenSpec change 归档，并移至「已完成」或从开放列表删除。

来源包括：代码探索（2026-06）、[ROI 探索会话](53bf0a6f-d612-4383-bcdd-65320a46130d)、[stop turn 后优先级梳理](b3c3a6da-23b0-4346-a080-86ad39ba6807)。

## Requirements

### Requirement: 待办项跟踪

系统维护者 SHALL 在本 spec 中维护按优先级分组的待办项；每项 MUST 包含：标题、现状、建议方向、建议 OpenSpec change 名（若适用）、关联 spec/模块。

#### Scenario: 新差距发现

- **WHEN** 代码审查或用户反馈发现 spec 未覆盖或未实现的差距
- **THEN** 维护者将条目追加至对应优先级分组，并通过 OpenSpec change 实现后移至「已完成」

#### Scenario: 已完成条目归档

- **WHEN** 某 backlog 条目随 change 实现并入主 spec
- **THEN** 开放列表中删除或标注完成日期，并在「已完成」表记录摘要

---

## 建议立项顺序（2026-06-22 更新）

```text
下一批 P1  → project-agent-profile → session-title-inline-edit（快赢）→ file-change-diff-undo
稳定性 P2  → clarify-cancel-race → skill-run-interrupt-on-cancel
性能/架构  → BL-002～BL-005、BL-101～BL-109（按用户反馈择机）
卫生       → BL-201～BL-208
```

---

## 高优先级 — 产品能力（P1）

### BL-006 项目级 AGENTS.md / profile 注入 — 已实现（`openspec/changes/project-agent-profile`）

- **现状**：已实现项目根 `AGENTS.md` 每 turn 读盘注入；`/init` 真斜杠 command + `confirm_agents_md` clarify。
- **能力 A**：存在即注入（≤3000 字符）；手写/外部编辑生效，无需 `/init`。
- **能力 B**：`/init` → `send_message` 占 turn → `profile` skill + clarify → 写 `AGENTS.md`；clarify pending 时禁止；仅 init turn 允许 Agent 写该文件。
- **代码**：`agent/agents_md.rs`、`loop_support.rs`、`assets/skills/profile/SKILL.md`、`slashCommands.ts`（`kind: command`）
- **关联**：`project-agent-profile`、`clarify-interaction`、`slash-commands`

### BL-007 Agent 文件变更 diff / 撤销（构建产物信任）

- **现状**：MVP 列表+打开已实现（change `build-artifacts-panel`，2026-06-23）。右侧栏上半区新增「构建产物」Tab（与工具调用链切换），按 turn 累积 `changed_paths` 去重展示交付物（`.cache` 中间产物已过滤）；每项支持「用默认程序打开」（复用 `open_project_file`）与「在文件夹中显示」（新增 `reveal_project_file` IPC）。纯前端累积，刷新/重载后丢失。
- **目标**：办公场景信任感——用户可见「本 turn 构建产物/变更列表」，MVP（列表+定位）已完成；进阶为 diff 预览与一键撤销（需快照策略，留待后续 change）。
- **建议 change**：`file-change-diff-undo`（diff/undo 范围在 proposal 明确；MVP 已由 `build-artifacts-panel` 覆盖）
- **关联**：`workspace-ui/spec.md`、`agent/loop_tool_batch.rs`、`useProjectFiles.ts`、`tools/changed_paths.rs`

### BL-008 侧栏会话标题 inline 编辑

- **现状**：`store` 已有 `title_user_edited`；`update_session` 支持改 title；`SessionList` 仅展示 + 删除 + tooltip，无重命名入口。
- **目标**：侧栏双击或菜单重命名；与自动标题（首条 user / LLM 两轮）共存，用户编辑后不再被 autotitle 覆盖。
- **建议 change**：`session-title-inline-edit`
- **关联**：`project-session/spec.md`、`SessionList.tsx`、`ipc/mod.rs`

---

## 高优先级 — 性能与稳定性（P1/P2）

### BL-001 文件占用错误 UI（file_busy） — 已完成（2026-06-22）

- **实现**：`ToolChainPanel` 在工具失败时展示 `summary`；`fileBusy.ts` 解析 `file_busy` JSON 并格式化后端 `message`。
- **关联**：`workspace-ui/spec.md`、`src/lib/fileBusy.ts`、`src/components/ToolChainPanel.tsx`

### BL-002 长会话消息全量 reload

- **现状**：`turn_complete` / `context_compacted` / `turn_cancelled` 时前端全量 `list_messages`；`store.list_messages` 无分页。
- **建议**：增量 merge 新消息或分页加载；turn 结束仅 patch 差异。
- **关联**：`useWorkspace.ts`、`core/store.rs`

### BL-003 重 IO 工具阻塞 async runtime

- **现状**：多数工具 handler 同步执行；仅 Typst 等少数使用 `spawn_blocking`。
- **建议**：Office/PDF/FS 重 IO 统一 `spawn_blocking` 或专用 blocking pool。
- **关联**：`tools/registry.rs`

### BL-004 关键错误仅 console.error

- **现状**：`useWorkspace.ts` 多处 `.catch(console.error)`，发送/加载失败用户不可见。
- **建议**：关键路径 surfaced 为 toast 或 `SendHintBanner`。
- **关联**：`useWorkspace.ts`、`workspace-ui/spec.md`

### BL-005 SQLite 读写竞争

- **现状**：全局 `Mutex<Store>`；未启用 WAL；多 turn 并行时锁竞争加剧。
- **建议**：启用 `PRAGMA journal_mode=WAL` + busy_timeout；长期考虑缩小锁粒度。
- **关联**：`core/store.rs`、`state.rs`

### BL-009 Clarify cancel 与 submit 竞态

- **现状**：用户点「取消澄清」时，若 submit 已越过 `delete_clarify_pending`，答案仍可能落库；resume 再因 cancel 变 `turn_cancelled`，出现「已取消但答案已写入」。cancel 先于 submit 的场景已覆盖。
- **建议**：submit 与 cancel 互斥（乐观锁 / 状态机）；或 cancel 后丢弃迟到的 submit。
- **建议 change**：`fix-clarify-cancel-race`
- **关联**：`agent/clarify_interaction.rs`

### BL-010 Stop 期间 skill_run / html_to_pdf 可中断

- **现状**：用户点 Stop 后，若正在执行 `skill_run` 或 `html_to_pdf`，需等 handler 返回或 30s 超时；stop turn design 的 Non-Goal 遗留。
- **建议**：线程安全的中断信号 + runtime/print 协作式取消；复杂度高，单独评估。
- **建议 change**：`skill-run-interrupt-on-cancel`
- **关联**：`agent/turn_control.rs`、`tools/runtime/mod.rs`

### BL-011 OOXML libxml 全量 XSD 校验

- **现状**：`2026.6.19` 已实现零 native 的 well-formed + bundled XSD **规则**校验（`ooxml/validate/`）；**libxml 从未接入**。原探索项指进一步减少「Office 打不开」类失败。
- **建议**：评估规则集覆盖率 vs libxml 成本；缺口大时再立项 native XSD。
- **关联**：`ooxml-toolchain/spec.md`、`tools/ooxml/validate/`

### BL-012 boa heap 内存上限

- **现状**：design 提过内存上限；代码仅 32MB **栈**，无 heap 限制；大 exceljs 脚本有 OOM 拖垮进程风险。
- **建议**：可配置 heap 上限 + 友好错误；或文档化脚本规模限制。
- **关联**：`script-runtime/spec.md`、`tools/runtime/mod.rs`

---

## 中优先级 — 体验延伸

### BL-110 后台 turn 完成通知

- **现状**：per-session running 指示已有；用户在会话 B 时，后台会话 A 完成可能无感知（侧栏 spinner 消失但无 toast/badge）。
- **建议**：非 active session `turn_complete` 时轻量 toast 或侧栏完成标记。
- **关联**：`useWorkspace.ts`、`SessionList.tsx`

### BL-111 Token / 费用估算

- **现状**：有 `context_usage`（token 比例）；无 cost 维度；`provider-balance` 仅 DeepSeek/Kimi 余额。
- **建议**：按模型单价估算单轮/会话费用（可选展示）；依赖 models.dev 或自建单价表。
- **关联**：`provider-balance/spec.md`、`context-compaction/spec.md`

### BL-112 会话导出 Markdown

- **现状**：有 html-report 导出能力；无「导出对话为 Markdown/HTML」。
- **建议**：基于 `list_messages` 生成可读 Markdown；可选含工具调用摘要。
- **关联**：`html-report/spec.md`

---

## 中优先级 — 架构 / 可维护性

### BL-101 拆分 useWorkspace 上帝 Hook

- **现状**：`src/hooks/useWorkspace.ts` 约 1000+ 行，聚合会话、发送、事件、推荐问等。
- **建议**：拆为 `useSessionRuns`、`useAgentEvents`、`useSendMessage`、`useSuggestions` 等。
- **关联**：`.cursor/rules/frontend-quality.mdc`

### BL-102 拆分 store.rs

- **现状**：`core/store.rs` 超过 1000 行，违反 Rust 硬上限 500。
- **建议**：按 projects / sessions / messages / tool_calls / clarify 拆子模块。
- **关联**：`.cursor/rules/maintainability-size.mdc`

### BL-103 拆分 agent loop 文件

- **现状**：`loop_tool_batch.rs`、`loop_runner.rs`、`compaction.rs` 均超 500 行。
- **建议**：clarify 批处理、reserved resume、compaction 触发各自独立文件。
- **关联**：`agent/`

### BL-104 拆分 ipc/mod.rs

- **现状**：全部 Tauri command 集中单文件。
- **建议**：按 projects / sessions / messages / attachments 拆模块。
- **关联**：`ipc/mod.rs`

### BL-105 前端层级耦合

- **现状**：`useWorkspace` 从 `ToolChainPanel` 导入 `formatCharCount`；`agentEvents` 依赖 UI 组件类型。
- **建议**：共用类型与工具函数迁至 `src/lib/` 或 `types.ts`。
- **关联**：`hooks/`、`components/`

### BL-106 core 依赖 tools 方向违规

- **现状**：`core/file_locks` 测试引用 `tools/runtime/write_gate`。
- **建议**：依赖反转或通过 trait 在 core 定义、tools 实现。
- **关联**：`core/file_locks.rs`

### BL-107 只读工具有限并行

- **现状**：仅 `pdf_read` 有 batch 并行（max 3）；其余工具串行。
- **建议**：在文件锁框架下评估 `fs_read`、`office_read_to_markdown` 等只读工具并行。
- **关联**：`agent/loop_tool_batch.rs`、`file-governance/spec.md`

### BL-108 项目文件索引全量 WalkDir

- **现状**：`list_project_files_cmd` 每次同步 WalkDir（MAX_DEPTH=6, MAX_ENTRIES=2000）。
- **建议**：mtime 缓存或增量 manifest。
- **关联**：`core/project_files.rs`

### BL-109 同步 IPC 阻塞 UI

- **现状**：`list_messages`、`save_upload`、`list_project_files_cmd` 等为 sync command。
- **建议**：async command + `spawn_blocking`。
- **关联**：`ipc/mod.rs`

---

## 低优先级（卫生 / 微优化）

### BL-201 Spec Purpose 段落补全

- **现状**：多数 `openspec/specs/*/spec.md` 的 Purpose 仍为归档时的 `TBD`。
- **建议**：按能力域逐批补全 Purpose。

### BL-202 清理 deprecated 前端 API

- **现状**：`projectFiles.ts` 中 `sameStringArrays`、`mergeProjectFilePaths` 标记 `@deprecated`。
- **建议**：确认无引用后删除。

### BL-203 拆分 tools/tests.rs

- **现状**：单文件 3000+ 行测试。
- **建议**：按工具域拆分为并列测试模块。

### BL-204 Tool 双路径注册清理

- **现状**：`pdf_read` / web 工具在 ToolSpec 与 registry 各有 handler 路径。
- **建议**：统一 async handler 注册，移除 dead stub。

### BL-205 Reserved resume 轮询

- **现状**：`loop_runner` 固定 120×500ms sleep 等待 slot。
- **建议**：channel/事件通知 slot 释放。

### BL-206 mark_messages_archived 批量 UPDATE

- **现状**：逐条 UPDATE archived 标记。
- **建议**：`WHERE id IN (...)` 批量更新。

### BL-207 含图消息压缩 token 估算

- **现状**：`compaction` 对 attachment 仅文本 token 估算（spec 已接受 trade-off）。
- **建议**：若用户反馈压缩偏晚，对含 attachment 消息加保守系数。

### BL-208 MiMo Provider 余额

- **现状**：`provider-balance` 仅 DeepSeek/Kimi。
- **建议**：MiMo 官方 API 支持余额查询时再扩展。

---

## 建议暂不做

与 [ROI 探索](53bf0a6f-d612-4383-bcdd-65320a46130d) 结论一致，默认不立项：

| 项 | 原因 |
|----|------|
| 全量跨会话 LLM memory | 与「会话隔离」产品立场冲突 |
| 跨 project 全局 turn 队列 UI | 已有 3 并行上限 + 文件锁；非当前痛点 |
| rustyscript / V8 引擎回切 | 依赖链短期无解 |
| 云同步 / 插件市场 / Linux 安装包 / 本地模型 | MVP / 发布矩阵已排除 |
| TypeScript 转译进 skill_run | 成本高；优先文档化 + import normalize |

---

## 已完成

| 日期 | 项 | 摘要 |
|------|-----|------|
| 2026.6.18 | Stop turn + 会话运行态 | `TurnRegistry`、`cancel_turn`、per-session running/stopping、侧栏 spinner、SSE/压缩可取消 |
| 2026.6.18 | skill_run 运行时补 op + spec 修正 | `skill_read runtime`、`doc_exists`/`doc_list`、import normalize、bundle 收紧；change `skill-run-runtime-ops` |
| 2026.6.19 | 有界并行与文件治理 | 全局 3 并行、`FileLockRegistry`、`.cache` 路径统一；取代「同项目单 turn 互斥」 |
| 2026.6.19 | OOXML pack 结构规则校验 | well-formed + XSD 规则门禁（零 libxml） |
| 2026.6.19 | skill_run 脚本跨 turn 保留 | 成功脚本保留于 `.cache/skill-run/<session_key>/`，便于续改构建产物 |
| 2026-06-22 | BL-001 file_busy UI | 工具链卡片展示文件占用错误 |
| 2026-06-22 | Spec 与实现对齐 | API Key → `config.toml`；office-tools PPT 条款更新 |
| 2026-06-23 | BL-007 构建产物面板 MVP | 右侧栏「构建产物」Tab，按 turn 累积 changed_paths（过滤 `.cache`）；打开 + reveal；change `build-artifacts-panel` |

---

## 已对齐（文档 / spec）

| 项 | 说明 |
|----|------|
| API Key 存储 | 应用数据目录 `config.toml`（`[api_keys]`），非 OS keychain；见 `model-config/spec.md` |
| PPT 生成 | 经 `skill_run` + pptx skill；已移除 office-tools「MVP 排除 PPT」 |
| InitCapsule | 空会话推荐问初始化（`InitCapsule`）已实现；与 BL-006 项目 profile **不同**——后者为持久化项目偏好文件 |
