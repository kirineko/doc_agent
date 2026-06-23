## 1. 类型与 reducer 基础

- [x] 1.1 `src/types.ts`：为 `LiveToolCall` 增加可选字段 `changed_paths?: string[]`（事件类型已具备，仅补 stream 侧类型）
- [x] 1.2 `src/lib/agentEvents.ts`：定义 `TurnArtifact { path; sourceToolCallId; sourceToolLabel }` 类型，并在 `AgentStreamState` 增加 `turnArtifacts: TurnArtifact[]`
- [x] 1.3 `src/lib/agentEvents.ts`：`tool_result` 分支在更新 status/summary 后，当 `ok && changed_paths?.length` 时合并进 `turnArtifacts`（按 path 去重，保留首个来源工具）
- [x] 1.4 `src/lib/agentEvents.ts`：`markAgentBusy`（新 user 消息发起）清空 `turnArtifacts`；验证 clarify 回答不清空（沿用同 turn）

## 2. changed_paths 过滤 .cache（后端）

- [x] 2.1 `src-tauri/src/tools/changed_paths.rs`：在 `extract_changed_paths` 最终去重前，用 `core::cache_paths::is_cache_path` 过滤掉 `.cache/` 下路径（复用现有函数，不新增依赖）
- [x] 2.2 `src-tauri/src/tools/changed_paths.rs`：补单测——`ooxml_unpack` 返回 `.cache/ooxml/<hash>/` 时 changed_paths 为空；`ooxml_pack` 的 `report.docx` 正常保留
- [x] 2.3 验证：确认 `useProjectFiles` 的 `@` 索引行为不受影响（`mergeProjectFileEntries` 本就丢弃 `.cache`，后端过滤属冗余消除）

## 3. reveal IPC（后端）

- [x] 3.1 `src-tauri/src/ipc/mod.rs`：新增 `reveal_project_file(path)` 命令，经 sandbox 校验路径在项目根内（复用 `open_project_file` 的 resolve 逻辑）
- [x] 3.2 `src-tauri/src/ipc/mod.rs`：按 `#[cfg(target_os)]` 分支执行——macOS `open -R <abs>`、Windows `explorer.exe /select,<abs>`、Linux `xdg-open <parent_dir>`
- [x] 3.3 `src-tauri/src/lib.rs`：注册 `reveal_project_file` 到 Tauri command 列表
- [x] 3.4 `src-tauri/src/ipc/mod.rs` 或 `capabilities/default.json`：确认 reveal 命令在权限能力范围内（若需新增 capability scope）

## 4. 产物面板组件（前端）

- [x] 4.1 新建 `src/components/BuildArtifactsPanel.tsx`：接收 `artifacts: TurnArtifact[]`，渲染路径列表（图标 + 路径 + 来源工具标签）
- [x] 4.2 `BuildArtifactsPanel.tsx`：每项提供「打开」（`invoke("open_project_file")`）与「在文件夹中显示」（`invoke("reveal_project_file")`）动作；失败时 toast 或内联错误
- [x] 4.3 `BuildArtifactsPanel.tsx`：空态文案「本轮没有产生或修改文件」；目录类产物用目录图标标注（isDir 推断），不递归展开子文件
- [x] 4.4 `BuildArtifactsPanel.tsx`：复用 ToolChainPanel 的贴底滚动 / 空态视觉风格，保持一致

## 5. Tab 切换容器（前端）

- [x] 5.1 新建 Tab 容器（或在现有承载工具链的面板外层）：「工具调用链」「构建产物 (N)」两个 Tab，徽标 N=去重后产物数
- [x] 5.2 Tab 容器：默认选中「工具调用链」；Tab 切换不触碰上下分割条 / 高度比例 / 折叠状态
- [x] 5.3 Tab 栏跟随工具链折叠态收起（折叠时仅留标题行）
- [x] 5.4 把 `turnArtifacts` 从 `useWorkspace` 经 stream state 传入 Tab 容器与 BuildArtifactsPanel

## 6. 测试

- [x] 6.1 `src/lib/agentEvents.test.ts`：`tool_result` 带 `changed_paths` 时正确累积到 `turnArtifacts`；同路径去重
- [x] 6.2 `src/lib/agentEvents.test.ts`：`markAgentBusy`（新消息）清空；`turn_awaiting_user`/clarify 不清空
- [x] 6.3 `src/components/BuildArtifactsPanel.test.tsx`：渲染列表、空态、徽标计数
- [x] 6.4 Rust：`reveal_project_file` sandbox 越界路径返回错误（单测）；`extract_changed_paths` 过滤 `.cache` 单测；平台命令分支编译验证
- [x] 6.5 `npm run typecheck && npm test && npm run build` 通过；`cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test` 通过

## 7. 验收与对齐

- [x] 7.1 `npm run dev` 手测：跑一轮 skill_run 产出 docx+xlsx，产物 Tab 列出三项、徽标为 3、打开与定位均生效
- [x] 7.2 手测：`ooxml_unpack`→`ooxml_pack` 流程，产物列表只含最终 docx，不含 `.cache/ooxml/` 中间目录
- [x] 7.3 手测：Tab 切换不改上下布局；折叠工具链时 Tab 栏收起
- [x] 7.4 手测：新消息发送后产物清空、徽标归零；刷新后产物丢失（符合 MVP-0 预期）
- [x] 7.5 更新 `openspec/specs/project-backlog/spec.md` BL-007：标注「MVP 列表+打开已实现（build-artifacts-panel）」，diff/undo 仍开放
