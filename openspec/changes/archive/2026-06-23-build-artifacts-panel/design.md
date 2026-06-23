## Context

`tool_result.changed_paths` 已从后端完整传到前端事件（`loop_support.rs:223-234` → `types.ts:145-154`），但目前唯一消费者是 `useProjectFiles`（用于刷新 `@` 索引与文件树），而 `agentEvents.ts:115-127` 的 `tool_result` 分支在更新 `status`/`summary` 后**丢弃了 `changed_paths`**。`LiveToolCall` 上没有该字段，因此「本轮产出/修改了哪些文件」在产品内完全不可见——用户必须自己去文件树里翻找。

右侧栏上半区当前是「工具调用链」单一面板（`ToolChainPanel.tsx`），与下半区「项目文件浏览」共享纵向空间并有折叠/拖拽语义（见 `workspace-ui/spec.md:161`「右侧工具调用链可视化」）。办公场景的核心诉求是：一眼看到本轮构建产物并能打开。这是 BL-007 的最小切片；diff 与回滚需要文件快照层，成本与风险量级远大于此，显式排除。

## Goals / Non-Goals

**Goals:**
- 在右侧上半区引入「构建产物」视图，与「工具调用链」通过 Tab 切换。
- 前端按 turn 累积 `changed_paths`，展示本轮产生/修改的项目相对路径。
- 产物项可「用默认程序打开」（复用 `open_project_file`）与「在文件管理器中定位」（reveal）。
- 零后端 schema 变更、零持久化；纯前端累积 + 一个新 reveal IPC。

**Non-Goals:**
- diff 预览（文本或 OOXML 二进制）。
- 回滚 / 撤销 / 文件快照（需写入拦截网，后续 change）。
- `changed_paths` 落库或历史会话重载后可见（MVP-1 范围）。
- 后端 `turn_id` 列。
- 本轮产物在文件树中的高亮。

## Decisions

### Decision 1: 产物视图用 Tab 切换，而非新增第三栏

右侧上半区在「工具调用链」与「构建产物」之间提供 Tab 切换，**同一时刻只展示其一**。

```
┌─────────────────────────────────────┐
│ [ 工具调用链 ] [ 构建产物 (3) ]      │  ← Tab 栏（徽标=本轮产物数）
├─────────────────────────────────────┤
│  <当前选中 Tab 的内容>               │
└─────────────────────────────────────┘
```

**Why Tab over 新增栏**: 现有右侧栏已是上下两区（工具链 + 文件浏览）且共享宽度（`spec.md:161`）。再竖切一栏会破坏现有布局契约与拖拽/折叠互斥语义。Tab 复用上半区空间，不碰下半区文件浏览，也不改高度比例与折叠行为——对既有「右侧工具调用链可视化」需求是**纯增量叠加**（在其容器内加 Tab 栏），不修改其布局/折叠/拖拽语义。

**Alternative considered**: 把产物作为工具链卡片内的折叠子区。否决——产物与工具调用是正交视角（一个 turn 多个工具都可能写文件），嵌进工具卡会割裂「本轮产物全集」。

### Decision 2: turn 边界由前端推导，不落库

一个 turn = 一条 user 消息后的所有 assistant/tool 事件。前端在 `agentEvents` reducer 里：
- `markAgentBusy`（新 user 消息发起，`agentEvents.ts:178`）时清空 turn 级产物累积。
- `tool_result` 分支：若 `ok && changed_paths?.length`，合并进当前 turn 产物集（按路径去重，保留来源工具调用 id 与 label）。
- `turn_complete` / `turn_cancelled` / `turn_awaiting_user`：保留产物列表供用户查看，直到下一个 `markAgentBusy` 清空。

**Why 不用后端 turn_id**: `turn_id` 是 UUID，仅存在于事件，未落 `messages`/`tool_calls`（探索结论）。前端按 user 消息切片推导 turn 边界完全可靠，且零后端改动。落库 turn_id 属 MVP-1。

**Turn 产物数据结构**（前端）:
```ts
interface TurnArtifact {
  path: string;            // 项目相对 POSIX 路径
  sourceToolCallId: string;// 产生它的 tool_call id（去重时保留首个）
  sourceToolLabel: string; // 工具中文名（来自 toolLabel）
}
// AgentStreamState 增加: turnArtifacts: TurnArtifact[]
```

### Decision 3: 累积放在 AgentStreamState，与 liveTools 同生命周期

产物累积挂在 `AgentStreamState`（`agentEvents.ts:8-14`），与 `liveTools` 同步重置。理由：产物本质是「当前 turn 的流式副产物」，与工具链卡片同生同灭；放到 `useProjectFiles` 会混淆职责（后者管文件索引，不涉及 turn 概念）。

**Alternative considered**: 在 `useWorkspace` 单独维护 turn 产物 state。否决——`useWorkspace` 已是「上帝 Hook」（BL-101），不应再加职责；放 reducer 与现有事件流耦合更紧、更易测试。

### Decision 4: reveal 用平台命令封装，不依赖 opener 插件的文件语义

现有 `tauri_plugin_opener::open_path(file)` 对**文件**是「用默认程序打开文件」，不是「在 Finder 中定位」。`open_project_root` 只能打开根目录，无法定位具体文件。因此新增一个 IPC `reveal_project_file(path)`：
- macOS: `open -R <abs_path>`
- Windows: `explorer.exe /select,<abs_path>`
- Linux: `xdg-open <parent_dir>`（无统一 select 语义，降级为打开父目录）

通过 sandbox 校验路径在项目根内（复用现有 `resolve` 逻辑，同 `open_project_file`），再 `Command::new(...)` 执行平台命令。

**Why 不用 std::process 直接 shell**: 走 `tauri::process::Command`（或 `std::process::Command`）即可，无需新 crate；平台分支在 Rust 侧集中，前端只调一个命令。`tauri_plugin_opener` 不暴露 reveal 语义，强行用它会在 macOS 上变成「启动文件」而非「定位」。

### Decision 5: Tab 徽标显示本轮产物数

Tab 标题「构建产物」带数字徽标（`(N)`），N=本轮累积的产物路径数（去重后）。N=0 时仍可点击进入（显示空态文案）。让用户在不切 Tab 时就能感知「本轮有产出」。

### Decision 6: 在后端提取层过滤 `.cache`，产物仅含交付物

`changed_paths` 当前会把 `ooxml_unpack` 自动生成的 `.cache/ooxml/<hash>/` 这类**中间工作目录**也返回（`changed_paths.rs:10-16` 从 `result.out_dir` 取）。这是 Agent 内部工序的脚手架（解包的 XML 碎片、渲染的 PNG 页缓存等），用户既看不懂也打不开，显示为「构建产物」是噪音。办公场景的信任感诉求指向**交付物**（最终 docx/xlsx/pdf/md），而非中间物。

因此在 `changed_paths.rs::extract_changed_paths` 的最终去重前增加一行过滤：`is_cache_path(&p)` 的条目全部丢弃（复用 `core/cache_paths.rs:23` 的 `is_cache_path`）。

**Why 后端过滤而非前端**：
- 口径一致——`@` 文件索引（`project_files.rs:6`）与文件树本就已在用 `is_cache_path` 过滤 `.cache`；后端过滤让 `changed_paths` 的语义与它们对齐，消除「`@` 索引丢弃但产物面板却显示」的不一致。
- 零功能损失——验证过 `mergeProjectFileEntries` 现在对 `.cache` 路径也是丢弃的（`projectFiles.ts` 同源过滤），所以后端过滤掉 `.cache` 不会破坏 `@` 索引刷新。
- 改动集中——一处 filter，前端/`@` 索引/产物面板三处都受益，无需散落各处重复过滤。

**Alternative considered**: 前端在 `BuildArtifactsPanel` 内用 `is_cache_path` 过滤。否决——会让 `@` 索引与产物面板口径继续分歧，且每新增一个 `changed_paths` 消费者都要记得过滤，易漏。

### Decision 7: MVP 不展开目录，按 changed_paths 原样显示路径

产物列表按 `changed_paths` 返回的路径逐条展示，**不递归展开目录内的子文件**。理由：
- 过滤掉 `.cache` 后，剩余路径几乎都是**文件**（docx/xlsx/pdf/md/csv 等），目录类只在用户显式指定 `out_dir`（如 `pdf_split` 输出到 `output/`）时出现。
- 显示目录本身是合理的——用户正是让工具输出到该目录，目录名即交付边界。
- 展开成子文件需额外 IPC（列目录内容）与去重逻辑，ROI 低，留作后续增强，不进 MVP。

产物项的「打开」与「定位」对文件和目录统一可用（`open_project_file` 经 `tauri_plugin_opener::open_path` 对目录打开文件管理器、对文件打开默认程序；`reveal_project_file` 对两者均定位）。因此面板**不做目录/文件区分**——所有路径显示「打开」+「定位」，图标按扩展名推断（无扩展名默认 📄）。

**不在路径格式上承载目录语义**：早期尝试给目录路径补尾部 `/`（`output/`）供前端推断，但这会污染被多处消费的共享数据 `changed_paths`——`useProjectFiles` 的 `mergeProjectFileEntries` 会因 `pathBasename("output/")` 为空而出现目录重复或丢失，直到全量刷新。正确做法是让目录/文件区分由需要它的消费侧自行解决（`@` 索引的 `inferIsDirForMergedPath` 启发式、产物面板的扩展名图标），`changed_paths` 保持纯路径。

> 未来若需权威的 isDir，应改事件结构为 `changed_paths: {path, is_dir}[]`（Option B），而非字符串编码。属独立改进，不在本 change 范围。

## Risks / Trade-offs

- **[刷新页面后产物丢失]** → MVP-0 接受此限制；产物仅当前会话、当前 turn 可见。空态文案明确告知「仅本轮，刷新或切换 turn 后重置」，管理预期。持久化留待 MVP-1。
- **[Tab 与现有折叠/拖拽语义冲突]** → Tab 栏只作用于上半区**内容**的横向切换，不触碰上下分割条、高度比例、折叠互斥。现有「右侧工具调用链可视化」需求的语义全部保留；Tab 是叠加层。需在实现时确保 Tab 栏在工具链折叠时也收起（跟随折叠态）。
- **[reveal 平台差异]** → Linux 无统一 `/select`，降级为打开父目录；在 UI 上对 Windows/macOS 标注「在文件夹中显示」，Linux 行为一致但实现不同。用 `#[cfg(target_os)]` 分支隔离。
- **[turn 推导边界与 clarify 交互交叉]** → clarify turn（`turn_awaiting_user`）也会触发 `markAgentBusy`? 需确认 clarify 不清空产物。设计为：仅 user **发送新消息**（非 clarify 回答）才清空；clarify 回答沿用同 turn。实现时验证 `markAgentBusy` 的调用点。
- **[产物路径含 unpacked 目录等中间产物]** → `.cache/` 下的中间工作目录整体过滤；目录类交付物（`out_dir`）不补尾部斜杠，与文件路径统一展示，「打开」对目录打开文件管理器。
