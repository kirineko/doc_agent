## Why

用户在侧栏浏览会话、在右侧栏浏览项目文件时暴露出两类认知负担：自动生成的会话标题过长且常取自助手首句（寒暄或陈述句），难以一眼区分会话；文件浏览的返回入口仅角落里的 `..`，非技术用户不易发现且不理解含义。本次变更在不大改架构的前提下，优化首轮标题生成策略与文件导航可发现性。

## What Changes

- 会话自动标题改为**用户意图摘要**（≤16 字），优先使用用户消息而非助手首句
- 泛化开场白（如「你好」）首轮保持「新会话」；若第二轮用户消息有实质意图且标题仍为默认，再尝试自动命名（仅一次重试）
- 右侧项目文件浏览：移除标题栏角落 `..`；列表首行提供 Finder 风格「返回上级」；路径行展示可点击面包屑，根节点以 `⌂` 符号表示（含无障碍标签）
- 更新 `session_title` 启发式与单元测试；更新 `ProjectFileExplorer` 组件

## Capabilities

### New Capabilities

（无 — 行为增量归入既有能力）

### Modified Capabilities

- `project-session`：新增首轮/第二轮自动标题命名规则与泛化开场跳过逻辑
- `project-file-browser`：返回上级入口形态与面包屑导航要求（由 `..` 改为明示导航）

## Impact

- **Rust**：`src-tauri/src/agent/session_title.rs`（`MAX_TITLE_CHARS`、优先级、泛化检测、`Option` 返回值）、`loop_runner.rs`（`user_count == 2` 重试窗口）
- **前端**：`src/components/ProjectFileExplorer.tsx`（面包屑、列表首行返回、移除角落 `..`）
- **测试**：`session_title` 单元测试扩展；可选 `ProjectFileExplorer` 组件测试
- **依赖**：无新增 crate/npm 包
