# 实施任务：工作区 UX 与 LLM 推荐问

实现与 `design.md` 冲突时先更新 artifact，再改代码。

## 1. 项目隐藏（specs/project-session、workspace-ui）

- [x] 1.1 `store.rs` migration：`projects` 加 `hidden INTEGER NOT NULL DEFAULT 0`（duplicate column 容错）
- [x] 1.2 `create_project` 改 upsert：按 `root_path` 命中则 `hidden = 0` 并返回原记录；新增 `hide_project`；`list_projects` 过滤 `hidden = 0`；补 store 单测（隐藏→重选恢复→会话保留）
- [x] 1.3 IPC 注册 `hide_project`；Sidebar 项目卡片 hover `×` 按钮；隐藏激活项目时自动切换；项目列表 `max-h-28` → `max-h-52`

## 2. @ 文件引用（specs/workspace-ui）

- [x] 2.1 IPC `list_project_files(project_id)`：walkdir 深度 ≤6、上限 2000、忽略 `.`/`node_modules`/`target`/`~$*`，返回相对路径 + `is_dir`（含 Rust 单测）
- [x] 2.2 `src/lib/fuzzy.ts`：子序列匹配 + 评分 + 命中位置；vitest 单测（中文/路径段/排序）
- [x] 2.3 `src/lib/mention.ts`：光标位置 `@` 检测与替换的纯函数；vitest 单测
- [x] 2.4 `FileMentionPopup.tsx`：候选弹层（≤8 条、键盘导航、高亮）；ChatPanel 集成（项目级文件清单缓存、确认插入 `@路径 `）
- [x] 2.5 `loop_runner` system prompt 追加 `@路径` 语义说明

## 3. 推荐问后端（specs/smart-suggestions）

- [x] 3.1 `tools/office.rs` 提取可复用 `read_document_text(path)`（tool handler 与 suggest 共用）
- [x] 3.2 新建 `agent/suggest.rs`：starter 上下文（最近 ≤3 文档摘要、每个 ≤2000 字符 + ≤50 条文件清单）与 followup 上下文（最近 ≤6 条消息 + 工具足迹）构造
- [x] 3.3 flash 调用：`DeepSeekV4Flash` + `thinking.enabled=false` + 无工具 + no-op on_event + 20s 超时；JSON 数组解析（容忍 ``` 围栏），失败返回空数组；条数/长度裁剪
- [x] 3.4 IPC `generate_suggestions { session_id, kind }`；无 deepseek key 时返回明确错误
- [x] 3.5 单测：上下文构造截断、JSON 解析容错（围栏/坏输出→空数组）

## 4. 首次会话推荐前端（specs/smart-suggestions、workspace-ui）

- [x] 4.1 App 状态机：空会话 + 有 deepseek key → `initializing`；ChatPanel 输入框/发送按钮禁用 + 「会话初始化中…正在阅读项目文档」动效提示
- [x] 4.2 `SuggestionCards.tsx`：starter 卡片形态，点击即发送并清除
- [x] 4.3 过期保护（返回时校验 active session）；finally 解锁输入框；失败静默
- [x] 4.4 key 门控：无 deepseek key 完全跳过；`onApiKeyStatusChange("deepseek", true)` 且当前会话为空 → 补触发初始化
- [x] 4.5 vitest：状态机关键路径（无 key 跳过 / 失败解锁 / 过期丢弃）

## 5. 后续推荐前端（specs/smart-suggestions、workspace-ui）

- [x] 5.1 `turn_complete` 后异步调用 `generate_suggestions(followup)`，不禁用输入
- [x] 5.2 followup 胶囊渲染于消息流末尾；点击即发送；用户先行发送新消息时丢弃迟到结果
- [x] 5.3 vitest：迟到丢弃与点击发送

## 6. 验收

- [x] 6.1 Rust：`cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test` 全绿
- [x] 6.2 前端：`npm test` + `npm run typecheck` + `npm run build` 全绿
- [ ] 6.3 手动冒烟：隐藏→重选恢复；@ 选择中文文件；无 key / 配 key 补触发；初始化锁定与解锁；followup 展示
