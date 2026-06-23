## Why

用户在办公场景中缺乏「Agent 这一轮到底产出/修改了哪些文件」的可见性。`tool_result.changed_paths` 已从后端一路传到前端事件，却仅被用于刷新文件树索引（`useProjectFiles.ts:79-92`），而在工具链 reducer 中被丢弃（`agentEvents.ts:115-127`）。用户无法在产品内看到本轮构建产物列表，也无法从工具结果直接定位到产物文件，必须自己去文件树里翻找。这是 BL-007 的最小可用切片，也直接对应办公场景的「信任感」诉求。

## What Changes

- **新增「构建产物」面板**：右侧工作区新增一个与「工具调用链」并列的面板，两者通过 Tab 切换展示。产物面板列出「本轮」产生或修改的项目相对路径，按来源工具调用分组/标注。
- **前端累积产物列表**：在 `agentEvents` reducer 的 `tool_result` 分支保留 `changed_paths`，按当前 `turn_id` 累积成「本轮产物」。新 turn 开始时清空。
- **打开产物**：产物项支持「用默认程序打开」（复用现有 `open_project_file` IPC）与「在文件管理器中定位」（reveal，新增 IPC）两种动作。
- **turn 边界由前端推导**：以一条 user 消息后的所有 assistant/tool 事件为一个 turn，从扁平消息流反推 turn 边界，无需后端改动。
- **Non-Goals（明确划出边界，留待后续 change）**：
  - diff 预览（文本或 OOXML 二进制）。
  - 任何形式的回滚 / 撤销 / 文件快照。
  - `changed_paths` 持久化到数据库（历史会话重载后可见产物）。
  - 后端 turn_id 列落库。
  - 本轮产物在文件树中的高亮。

## Capabilities

### New Capabilities
<!-- 无新增能力；本变更是对现有 workspace-ui 面板的扩展 -->

### Modified Capabilities
- `workspace-ui`: 新增「构建产物」面板与 Tab 切换行为；新增 turn 级产物列表的展示与打开/reveal 交互；turn 边界推导由前端完成。

## Impact

- **前端**：
  - `src/lib/agentEvents.ts`：`tool_result` 分支保留 `changed_paths`；`AgentStreamState` 增加 turn 级产物累积字段；新 turn (`markAgentBusy`) 清空。
  - `src/components/`：新增产物面板组件（如 `BuildArtifactsPanel.tsx`）；现有承载工具链的面板容器增加 Tab 切换。
  - `src/hooks/useWorkspace.ts`：可能需要把累积产物状态对接到面板。
  - `src/types.ts`：`LiveToolCall` 增加可选 `changed_paths` 字段（事件类型已具备，无需改）。
- **后端**：新增一个 IPC 命令（reveal in file manager）；`open_project_file` 直接复用。无数据库 schema 变更、无持久化。
- **依赖/插件**：reveal 依赖平台命令（macOS `open -R`、Windows `explorer /select`、Linux `xdg-open` 目录）；可能复用 `tauri-plugin-opener` 或新增薄封装。无新增 crate。
- **关联**：`openspec/specs/project-backlog/spec.md` BL-007（建议 change 名原为 `file-change-diff-undo`，本变更仅覆盖其 MVP 列表+打开部分，diff/undo 显式排除）。
