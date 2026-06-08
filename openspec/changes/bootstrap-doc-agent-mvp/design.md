# 设计：doc_agent MVP

## 1. 架构总览

```
┌───────────────────────────────────────────────────────────────┐
│                   前端 React + TS (Tauri WebView)              │
│  左:项目/会话/模型·思考配置 │ 中:Markdown流式 │ 右:工具调用链   │
└───────────────▲───────────────────────────────────────────────┘
                │ Tauri command(请求) + Event(流式四事件)
┌───────────────┴───────────────────────────────────────────────┐
│                    Tauri Core (Rust, 单一二进制)               │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ Agent Orchestrator (reqwest + SSE 自研)                   │ │
│  │   Provider 抽象: DeepSeek / Kimi (OpenAI 兼容)            │ │
│  │   loop: 拼上下文 → 流式请求 → 解析 tool_calls →           │ │
│  │         沙箱执行 → 回填(含 reasoning_content) → 重复       │ │
│  └───────────────┬──────────────────────────────────────────┘ │
│                  │ 工具分发(schema 化, 对齐 rmcp)              │
│  ┌───────────────▼──────────────────────────────────────────┐ │
│  │ 原生工具层 (in-process)                                   │ │
│  │   fs.list/read/write/search                               │ │
│  │   office.read_to_markdown   (office_oxide, 6 格式)        │ │
│  │   word.create / word.edit   (docx-rs / office_oxide)      │ │
│  │   excel.read / excel.write  (calamine / umya)             │ │
│  │   [预留] skill.run → Document Skill 脚本执行器            │ │
│  └───────────────────────────────────────────────────────────┘ │
│  安全边界: 路径 canonicalize + 项目根前缀校验                  │
│  持久化:   SQLite (projects/sessions/messages/tool_calls)      │
│  密钥:     OS Keychain (keyring crate)                         │
└────────────────────────────────────────────────────────────────┘
                  │ HTTPS
                  ▼   api.deepseek.com / api.moonshot.ai
```

模块拆分（Rust crate / 模块）：

- `core::sandbox` — 路径校验与项目根约束。
- `core::store` — SQLite 持久化（projects / sessions / messages / tool_calls / settings）。
- `core::secrets` — API Key 经 OS keychain 存取。
- `agent::provider` — `LlmProvider` trait + DeepSeek / Kimi 实现；SSE 流式解析。
- `agent::loop` — Agent 编排循环、工具分发、事件发射。
- `tools::*` — fs / office / word / excel 工具，统一 `Tool` 接口（name + JSON Schema + handler）。
- `ipc` — Tauri command + event 通道。

## 2. 关键设计决策

### 2.1 桌面框架：Tauri 2（而非 Electron）
- **理由**：spec 第 1 条「体积尽量小」。Tauri 包 3–15MB vs Electron 150MB+。Rust 后端天然承载沙箱 / 工具 / 持久化。
- **代价**：需写 Rust；WebView 渲染差异需测试。可接受。

### 2.2 Agent Loop：自研 reqwest + SSE（不用第三方 agent 框架）
- **理由**：两个模型均 OpenAI 兼容，loop 本质简单；`ds-api` 已不维护，规避小众库弃坑风险；完全可控的流式事件协议利于三栏 UI。
- **循环**：`messages → POST(stream) → 累积 delta.reasoning_content / delta.content / tool_calls → 若有 tool_calls 则沙箱执行并 append tool 结果(含本轮 assistant 的 reasoning_content) → 否则结束`。

### 2.3 思考模型接入（已核实官方文档）

| 维度 | DeepSeek V4 Flash / Pro | Kimi K2.6 |
|---|---|---|
| 思考开关 | `extra_body.thinking = {type: enabled/disabled}` | 同左（默认 enabled） |
| 思考强度 | `reasoning_effort: high / max` | 无（仅开关） |
| 思考内容字段 | `reasoning_content` | `reasoning_content` |
| 工具调用轮 | 必须回传 `reasoning_content`，否则 400 | 同左；另支持 `thinking.keep: all` |
| base_url | `https://api.deepseek.com` | `https://api.moonshot.ai`（OpenAI 兼容端点） |

- **Provider 抽象**：统一 `chat_stream(request) -> Stream<Event>`，由各 Provider 负责把统一的 `ThinkingConfig{enabled, effort}` 序列化为各自的 `extra_body`。Kimi 忽略 `effort`。
- **reasoning_content 回填**：assistant 消息持久化时存 `reasoning_content`；构造下一轮请求时，对「本轮内含 tool_calls 的 assistant 消息」必须带上 `reasoning_content`。

### 2.4 Office 能力选型（spike 实测）
- **读取/抽取**：`office_oxide::Document::open().to_markdown()` 统一处理 docx/xlsx/pptx + 旧格式 → 喂给 LLM 的上下文。
- **Word 生成**：`docx-rs`（细粒度）或 `office_oxide::create::create_from_markdown(md, Docx, path)`（LLM 产 Markdown 直转）。
- **Word 保格式编辑**：`office_oxide::edit::EditableDocument`（保留未改动 OPC 部件）。
- **Excel**：写用 `umya-spreadsheet`（`new_file` / `get_cell_mut().set_value*` / `writer::xlsx::write`），读用 `calamine`（快）。
- **PPT**：MVP 仅 `office.read_to_markdown` 支持读取；**不生成**。

### 2.5 工具接口：in-process 直调，schema 对齐 rmcp
- MVP 工具在 Rust 进程内直接调用（最轻、无 IPC），但工具定义采用 JSON Schema 描述，与 `rmcp` 工具体系对齐，便于未来：(a) 暴露为 MCP server；(b) 接入外部 MCP 工具。

### 2.6 安全沙箱
- 每个工具入参中的路径，执行前：解析为绝对路径 → `canonicalize` → 校验是否以项目根的 canonical 路径为前缀；否则拒绝。
- 写操作额外校验目标父目录在沙箱内；禁止跟随指向沙箱外的符号链接。

### 2.7 流式事件协议（Core → 前端）
统一四类事件（借鉴成熟 agent 事件模型），通过 Tauri event 推送，按 `session_id` + `turn_id` 路由：

| 事件 | 载荷 | 前端用途 |
|---|---|---|
| `reasoning_token` | `{delta}` | 中栏「思考中」折叠区 |
| `content_token` | `{delta}` | 中栏正文 Markdown 流式 |
| `tool_call` | `{id, name, args, status}` | 右栏工具卡片（pending/running） |
| `tool_result` | `{id, ok, summary, ms}` | 右栏工具卡片（done/error + 耗时） |

## 3. 数据模型（SQLite）

```sql
projects(    id, name, root_path UNIQUE, created_at )
sessions(    id, project_id, title, model, thinking_enabled,
             thinking_effort, created_at, updated_at )
messages(    id, session_id, role,            -- system|user|assistant|tool
             content, reasoning_content,      -- 思考内容随 assistant 存储
             tool_call_id, seq, created_at )
tool_calls(  id, message_id, name, args_json,
             result_json, status, duration_ms, created_at )
settings(    key, value )                      -- 非密钥配置；API Key 走 keychain
```

- 会话隔离：消息查询始终按 `session_id` 过滤；不跨会话拼接上下文。
- 重建上下文：按 `seq` 取某会话 messages，按 §2.3 规则决定 `reasoning_content` 是否回填。

## 4. Agent Loop 时序

```
用户发送 → 存 user message → 取本会话上下文(隔离)
   └─► Provider.chat_stream(messages, tools, thinking)
         ├─ 流: reasoning_token / content_token  → 前端
         ├─ 收到 tool_calls:
         │     存 assistant(含 reasoning_content + tool_calls)
         │     逐个: 沙箱执行 → 存 tool_calls → append tool message → 发事件
         │     回到 Provider.chat_stream(已 append, 带回 reasoning_content)
         └─ 无 tool_calls: 存最终 assistant message → 结束
```

## 5. 前端结构（React）
- 状态：项目 / 会话列表、当前会话消息流、工具调用链、模型配置。
- 流式：订阅 Tauri events，按 turn 聚合到消息与工具链视图。
- 渲染：`react-markdown` + `remark-gfm` + 代码高亮（shiki）+ KaTeX；右栏工具卡片组件。
- 模型配置区：模型下拉；thinking 开关；强度选择（仅 DeepSeek 显示，Kimi 隐藏/置灰）。

## 6. 风险与缓解

| 风险 | 缓解 |
|---|---|
| `office_oxide` v0.1.x 较新，复杂模板保格式编辑保真未知 | 限定 MVP 编辑场景（文本替换 / 单元格）；复杂场景留给 Document Skill |
| 模型 thinking + 工具调用回填漏传致 400 | Provider 层强约束 + 单元测试覆盖回填路径 |
| 沙箱绕过（`..` / 软链） | canonicalize + 前缀校验 + 软链拒绝；安全用例测试 |
| Tauri WebView 跨平台渲染差异 | Win/macOS 双平台冒烟测试纳入流程 |
| 无 API Key 无法端到端 | 提供 Mock Provider 跑通 UI / loop / 工具，再接真实模型 |

## 7. 后续扩展（非 MVP）：Document Skill（脚本型）
- 新增 `skill.run` 工具 + 沙箱脚本执行器（Python / Node 运行时，可选安装的「skill 引擎」）。
- Skill 仓库分层：内置 / 项目级（`<project>/.docagent/skills`）/ 用户级。
- 复杂能力（PPT 精排、复杂排版、特殊转换）通过 skill 提供，Agent 按 `SKILL.md` 指引调用脚本。
- MVP 阶段仅预留工具 schema 与执行接口位，不实现执行器。
