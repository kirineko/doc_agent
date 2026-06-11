## Context

当前状态：

| 数据源 | 加载时机 | IPC |
|--------|----------|-----|
| `@` 的 `filePaths` | `selectProject` 一次 | `list_project_files_cmd`（walkdir ≤2000） |
| 资源管理器 | `projectId` 变化 / 用户导航 | `list_project_dir_cmd`（单层 readdir） |

Agent 写文件（`fs_write`、`word_create`、`skill_run` 等）后无任何刷新。原 `workspace-product-polish` design D5 已预留「工具链完成后 refresh」但未落地。

`ooxml_unpack` 一次可写入 15–100+ XML 部件；若全部进入 `@` 索引会导致候选污染并加速触达 2000 上限。Agent 常用目录名：`unpacked`、`contract_unpacked` 等。

## Goals / Non-Goals

**Goals:**

- Agent 在本 turn 内创建/修改项目文件后，用户可在 `@` 与资源管理器（当前目录）看到更新
- 刷新策略事件驱动 + turn 边界 debounce，避免频繁全量 walkdir
- `@` 索引忽略 OOXML 解压工作目录整棵子树
- `skill_run` 间接写文件可被追踪

**Non-Goals:**

- 监听 OS 级 fs 事件（用户在外部 Finder 新建文件）— 可通过手动刷新缓解，后续 change 再加
- 资源管理器内隐藏解压目录内容 — 用户仍可手动点进浏览
- 修改 Agent 工具语义或 sandbox 规则

## Decisions

### D1：分层刷新，不全量轮询

```
tool_result (changed_paths) ──► 前端增量 merge filePaths（按忽略规则过滤）
                             └──► bump fileRevision → explorer reload currentPath

turn_complete ──► debounce 500ms ──► list_project_files_cmd（每 turn 最多 1 次）
                                  └──► 清单与前次不同才 bump fileRevision（兜底未登记工具）
```

| 操作 | 成本 | 触发 |
|------|------|------|
| `list_project_dir` | 单次 readdir | 手动刷新 / revision bump |
| 前端 `mergePaths` | O(n) 内存 | 每个带 `changed_paths` 的 tool_result |
| `list_project_files` | walkdir ≤2000 | turn 末 debounce 1 次 |

turn 末兜底刷新拿到的清单与当前 `filePaths` 逐项比较，仅在实际变化时 bump revision，
避免每个 turn 都触发 explorer 重读目录。

**否决**：定时 poll（空闲浪费）；每个 tool_result 全量 walk（解压场景 N 倍开销）。

### D2：`ToolResult.changed_paths`

在 `AgentEvent::ToolResult` 增加可选字段 `changed_paths: Option<Vec<String>>`（相对路径，POSIX `/`）。

提取策略（`loop_runner` 或 `tools` helper）：

| 工具 | 路径来源 |
|------|----------|
| `fs_write`, `word_create`, `excel_write` | args `path` |
| `ooxml_unpack` | args `out_dir`（目录级，不展开内部 XML） |
| `ooxml_pack`, `pdf_*`, `data_*` 等 | args `out_path` / `out_dir`（result 中的绝对路径不采用） |
| `office_convert` | result `path`（相对路径） |
| `skill_run` | runtime 线程内（thread_local）记录 `__doc_write` 写入，随脚本结果返回；`skill_run` 的 result JSON 携带 `written_paths`，提取时读取该字段 |

仅 `ok == true` 时填充；失败 emit `None` 或空 vec。

前端：`tool_result.changed_paths` → 过滤已在忽略规则内的路径 → `setFilePaths` 去重 merge。

### D3：OOXML 解压目录忽略规则

在 `project_files.rs` 的 `should_skip_entry`（walkdir `filter_entry`）中：

- 若某路径段 **等于** `unpacked`（忽略大小写），或 **以** `_unpacked` **结尾**，则跳过该目录项且不 descend
- 仅影响 `list_project_files`；`list_project_dir` 不做过滤（单层列举仍可见 `unpacked/` 文件夹名）

**理由**：与现有 Agent/测试命名一致；比「跳过所有 `.xml`」更安全（不误伤用户业务 XML）。

**替代方案否决**：仅前端过滤 — 后端 walk 仍慢且 truncated；glob `**/_unpacked/**` 用户不可配置 — MVP 不需要。

### D4：前端 `useProjectFiles` 内聚

从 `useWorkspace` 抽出（或内联模块）：

```ts
useProjectFiles(projectId)
  filePaths, fileRevision
  refreshAll()      // list_project_files_cmd
  onAgentEvent(e)   // merge + schedule debounced refreshAll
```

`ProjectFileExplorer` 接收 `projectId` + `fileRevision`；`revision` 变化时 `loadDir(projectId, currentPath)`（不 reset 到根）。

手动刷新按钮：调用 `loadDir` 当前路径，不 bump 全局 revision。

### D5：并发与 stale 防护

- 延续 `ProjectFileExplorer` 的 `loadSeqRef` 模式
- `refreshAll` 用 `projectId` + seq ref，切换项目时丢弃过期结果
- debounce timer 在 `projectId` 变化或 unmount 时 clear

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| debounce 窗口内 `@` 仍缺最新文件 | turn 末必全量 refresh；turn 中靠 `changed_paths` 增量 |
| 忽略规则误伤用户自建 `foo_unpacked/` 目录 | 命名与 Agent 工作流一致；文档说明；后续可改可配置 |
| `changed_paths` 提取遗漏新工具 | turn 末全量 refresh 兜底；新工具注册时补充映射 |
| 外部新建文件不可见 | 手动刷新按钮；spec 非目标明确排除 watcher |
| 2000 上限仍可能 truncated | 忽略解压目录显著减压；保留现有 `truncated` 语义 |

## Migration Plan

1. 后端先加忽略规则 + 单测（行为兼容，仅少列解压内部文件）
2. 后端 `changed_paths` + 前端事件处理
3. Explorer revision + 手动刷新
4. 无 DB 迁移；回滚移除事件字段与 hook 即可

## Open Questions

（无 — 用户已确认忽略 OOXML 解压工作目录）
