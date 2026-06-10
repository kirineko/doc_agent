# 提案：工作区 UX 与 LLM 推荐问（add-workspace-ux-suggestions）

## Why

MVP 的工作区交互存在四个体验短板：①项目列表展示区过小且无法移除不再关注的项目（且 `root_path UNIQUE` 导致重选同目录会报错）；②输入文件路径全靠手打，无 @ 引用与模糊查询；③新会话冷启动时用户不知道能问什么，需要大量手动输入；④对话结束后没有后续引导。本变更面向「分析与生成 Word / Excel / PPT」的核心用户，降低输入成本、提高智能感。

## What Changes

- **项目隐藏**：`projects` 表新增 `hidden` 字段；列表仅显示未隐藏项目；重选同目录时 upsert（自动恢复显示，历史会话保留）。项目列表展示高度加大。不提供「查看已隐藏」入口。
- **@ 文件引用**：新增 `list_project_files` IPC（walkdir，限深度/数量/忽略规则）；输入框检测 `@` 触发弹层，fzf 式模糊匹配（纯 TS 实现），选中后插入 `@相对路径`；system prompt 增加 `@路径` 语义说明。
- **首次会话推荐问（LLM）**：空会话打开时进入「会话初始化」状态（输入框禁用、显示进度提示），后端扫描项目文档并读取最近 ≤3 个文档的内容摘要，调用 **DeepSeek Flash（非思考模式）** 生成 3–4 条围绕文档分析/生成的推荐问，以卡片展示、点击即发送。
- **后续推荐问（LLM）**：每轮对话结束（turn_complete）后，基于近期上下文异步调用 DeepSeek Flash 生成 2–3 条 follow-up 推荐，以胶囊按钮展示在最新回复下方，不阻塞输入。
- **Key 门控**：未配置 DeepSeek key 时推荐问功能整体关闭（不初始化、不调用）；用户首次保存 DeepSeek key 后立即对当前空会话触发初始化；仅配置了非 DeepSeek key 同样不启用。

## Capabilities

### New Capabilities

- `smart-suggestions`: 基于 DeepSeek Flash 的首次会话推荐问与后续推荐问生成，含 key 门控与失败降级。

### Modified Capabilities

- `project-session`: 新增项目隐藏与同目录 upsert 恢复要求。
- `workspace-ui`: 新增项目列表隐藏交互、@ 文件引用选择器、会话初始化交互（输入禁用与进度提示）、推荐问展示交互。

## 纳入 / 排除

**纳入**：上述 4 项功能与对应测试。

**排除**：

- 已隐藏项目的管理/恢复入口（重选目录即恢复）
- 推荐问的规则化降级方案（无 key 即关闭，不做规则兜底）
- 推荐模型可配置（固定 DeepSeek Flash 非思考）
- @ 引用自动注入文件内容（仅插入文本标记，读取仍由 Agent 工具完成）
- 推荐问持久化与跨会话缓存

## Impact

- **代码**：`store.rs`（migration + upsert）、`ipc/mod.rs`（2 个新命令）、新增 `agent/suggest.rs`；前端 `Sidebar` / `ChatPanel` / `App` 改造，新增 `FileMentionPopup`、`SuggestionCards` 组件与 `lib/fuzzy.ts`。
- **依赖**：零新增（walkdir / reqwest / 现有 Provider 复用；前端模糊匹配自实现）。
- **风险**：①推荐问 LLM 调用失败或超时必须静默降级并解锁输入框；②文档内容摘要需截断控制 token 成本；③`hidden` migration 需兼容已有数据库（`ALTER TABLE` 容错）；④初始化期间用户切换会话/项目需正确取消或丢弃过期结果。
