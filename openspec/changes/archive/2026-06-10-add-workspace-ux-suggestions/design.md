# 设计：工作区 UX 与 LLM 推荐问

## 1. 项目隐藏

### 数据层（store.rs）

- migration：`ALTER TABLE projects ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0`，包一层「duplicate column 则忽略」容错，兼容旧库。
- `create_project` 改 upsert：先按 `root_path` 查询；命中则 `UPDATE hidden = 0` 并返回现有记录（id 不变 → 历史会话保留）；未命中则 INSERT。
- `list_projects` 默认 `WHERE hidden = 0`。
- 新增 `hide_project(id)`：`UPDATE projects SET hidden = 1`。

### IPC 与前端

- 新命令 `hide_project(project_id)`。
- Sidebar：项目卡片 hover 显示 `×`（复用会话删除按钮样式）；点击后从列表移除，若是当前激活项目则切换到剩余第一个（或清空选择）。
- 项目列表容器 `max-h-28` → `max-h-52`。

## 2. @ 文件引用

### 后端：`list_project_files` IPC

- 入参 `project_id`；用 `walkdir` 遍历项目根：
  - 深度 ≤ 6；总数上限 2000（截断并返回 `truncated: true`）
  - 忽略：以 `.` 开头的目录、`node_modules`、`target`、`~$*` Office 临时文件
- 返回 `Vec<{ path: String /* 相对路径 */, is_dir: bool }>`。
- 仅做展示用列举，不经 Sandbox 工具链（路径来自 walkdir 根内遍历，天然不越界）。

### 前端

- `lib/fuzzy.ts`：subsequence 匹配 + 评分（连续命中加分、路径段首字符加分、短路径加分），返回排序结果与命中位置（用于高亮）。纯函数、可单测。
- `FileMentionPopup.tsx`：受控弹层，props 为 `query / items / onPick / onClose`；最多渲染 8 条；`↑↓` 选择、`Enter/Tab` 确认、`Esc` 关闭。
- ChatPanel 集成：
  - 触发：光标前最近 `@` 与光标之间无空白 → 激活，`@` 后文本为 query
  - 文件列表在选中项目时拉取一次并缓存（项目切换时失效重拉）
  - 确认后将 `@query` 替换为 `@相对路径 `（尾随空格），关闭弹层
- system prompt（loop_runner `build_working_messages`）追加一句：「用户消息中 `@路径` 指代项目内文件，可直接用 fs / office 工具读取」。

## 3. LLM 推荐问（starter + followup）

### 统一后端：`agent/suggest.rs`

```
generate_suggestions(state, session_id, kind)
  kind = "starter" | "followup"
  │
  ├─ 前置：secrets 无 deepseek key → Err("suggestions disabled")
  │        （前端正常情况下不会调到这里，双保险）
  │
  ├─ starter 上下文构造：
  │    walkdir 找文档文件（docx/xlsx/pptx/pdf/md/csv），按 mtime 取最近 ≤3 个
  │    复用 office 读取逻辑提取文本，每个截断 ≤2000 字符
  │    + 项目文件清单（≤50 条路径）
  │
  ├─ followup 上下文构造：
  │    当前会话最近 ≤6 条 user/assistant 消息（每条截断 ≤1000 字符）
  │    + 本会话最近工具调用名列表
  │
  ├─ 调用：DeepSeekProvider，model = DeepSeekV4Flash，
  │    thinking.enabled = false，tools = []，no-op on_event（收流但不转发）
  │    超时 20s（tokio::time::timeout）
  │
  └─ 输出解析：提示词要求「仅输出 JSON 字符串数组」；
       serde_json 解析失败 → 尝试剥离 ```json 围栏再解析 → 仍失败返回空数组
       条数裁剪：starter ≤4，followup ≤3；每条 ≤40 字
```

- 提示词要点（starter）：「你是办公文档助手的推荐问生成器。基于以下项目文件清单与文档内容摘要，生成 N 条用户最可能提出的、围绕文档分析/生成（Word/Excel/PPT/PDF）的具体问题，问题须可直接执行、提及具体文件名。仅输出 JSON 数组。」
- followup 同理，基于对话上下文生成「下一步」问题。
- office 文本提取：将 `tools/office.rs` 中读取转换逻辑提为可复用函数（如 `read_document_text(path) -> Result<String>`），tool handler 与 suggest 共用，避免复制。

### IPC

- `generate_suggestions { session_id, kind } -> Vec<String>`（async command；与 `send_message` 同样克隆 AppState）。
- 不新增「是否可用」命令：前端用已有 `has_api_key("deepseek")` 状态门控。

### 前端状态机（App.tsx）

```
选中会话
  │
  ├─ 无 deepseek key ──────────────▶ 正常空会话（无推荐、输入可用）
  │
  └─ 有 key 且会话无消息
        │
        ▼
   initializing = true ──▶ ChatPanel: 输入框 disabled
        │                  中央显示「会话初始化中…正在阅读项目文档」(spinner)
        ▼
   invoke generate_suggestions(starter)
        │
        ├─ 成功 ──▶ starterSuggestions 卡片（点击即发送）
        ├─ 失败/超时 ──▶ 静默清空（无卡片）
        └─ 无论成败 ──▶ initializing = false，输入框解锁（finally 保证）
```

- 过期保护：发起时记录 `session_id`，返回后若 active session 已变化则丢弃结果。
- **首次配置 key 触发**：`onApiKeyStatusChange("deepseek", true)` 且当前会话为空且尚无推荐 → 触发上述初始化流程。
- followup：`turn_complete` 事件处理中（busy 解除后）异步 `generate_suggestions(followup)`；结果以胶囊按钮渲染在消息流末尾；期间输入框**不**禁用；用户先行发送新消息则丢弃迟到结果（校验 busy 与消息数）。
- 组件：`SuggestionCards.tsx`（starter 卡片 + followup 胶囊两种形态，≤150 行）。

## 4. 失败与边界

| 场景 | 行为 |
|---|---|
| flash 调用失败 / 超时 / 解析失败 | 返回空数组，前端不展示，输入框必须解锁 |
| 初始化中切换会话/项目 | 丢弃过期结果，新会话重新走状态机 |
| 初始化中用户删除 key | 当次结果照常返回展示；后续不再触发 |
| 会话已有消息（历史会话） | 不触发 starter；followup 照常 |
| 项目文件为空 | starter 仍可生成通用文档创建类推荐（提示词允许） |

## 5. 体量与拆分

- `agent/suggest.rs` 预计 ~250 行（含提示词常量），不超 300 行上限；超出则拆 `suggest/prompt.rs`。
- 前端新组件均 ≤150 行；ChatPanel 增量控制在阈值内，@ 检测逻辑放 `lib/mention.ts` 纯函数便于单测。
