# 项目上下文（doc_agent）

## 目的

面向**办公人员**的跨平台桌面 AI Agent：用户选择一个本地目录作为「项目」，Agent 在该目录沙箱内自主读取、编辑、生成 Office 文档（Word / Excel 为主），通过「Agent Loop + 工具调用」完成办公文档任务。定位类比：「办公文档界的 Cursor / Claude Code」。

需求来源见仓库根目录 `spec.md`。

## 技术栈（已经调研 + spike 验证后确定）

| 层 | 选型 | 说明 |
|---|---|---|
| 桌面框架 | **Tauri 2** | 系统 WebView + Rust 单二进制，安装包 3–15MB，满足「体积尽量小」 |
| 前端 | **React + TypeScript** | 三栏 UI；Markdown 流式渲染 + 工具调用链可视化 |
| 核心后端 | **Rust** | Tauri Core：安全边界、持久化、Agent Loop、原生工具 |
| Agent Loop | **自研 `reqwest` + SSE** | 不依赖第三方 agent 框架（`ds-api` 已不维护，弃用） |
| 模型 | **DeepSeek V4 Flash / V4 Pro**、**Kimi K2.6** | 均 OpenAI 兼容，统一 Provider 抽象 |
| Office 读取/抽取 | **office_oxide** | 6 格式统一 → Markdown / text / HTML / IR |
| Word 生成/编辑 | **docx-rs** + **office_oxide**（`EditableDocument` / `create_from_markdown`） | |
| Excel 读写 | **umya-spreadsheet**（写）+ **calamine**（快读） | |
| 持久化 | **SQLite**（`rusqlite` / `sqlx`） | 项目 / 会话 / 消息 / 工具调用 |
| 工具 schema / MCP | **rmcp**（官方 Rust MCP SDK，预留） | MVP 工具 in-process，schema 体系对齐 MCP |

> spike 结论：上述 Rust 库已实测可用，纯 Rust 编译快、生成合法 OOXML；**唯独 PPT 生成视觉保真不达标，MVP 不做 PPT 生成**。

## 关键约束

1. **目录沙箱**：Agent 的所有文件操作必须限定在用户选定的项目根目录内。每次工具调用前对路径执行 `canonicalize` + 项目根前缀校验，拒绝 `..` 越界与软链穿越。
2. **体积尽量小**：优先纯 Rust、避免把重运行时（Python / Chromium / LibreOffice）打进默认安装包。
3. **会话隔离**：同一项目下可建多个会话，每个会话是独立上下文，**会话间不共享记忆**。会话历史与过程中的工具调用必须持久化。
4. **思考模型回填**：DeepSeek / Kimi 在「发生工具调用的轮次」必须把 assistant 消息的 `reasoning_content` 原样回传，否则 API 返回 400。持久化层必须随 assistant 消息存储 `reasoning_content`。
5. **思考强度差异**：DeepSeek 支持 thinking 开关 + `reasoning_effort: high/max`；Kimi K2.6 **只有开关、无强度**。UI 与 Provider 抽象必须体现该差异。

## 能力扩展：Document Skill（脚本型，MVP 之后）

复杂 / 长尾文档能力（PPT 精排、复杂排版、特殊格式转换等）**不写死在 Rust 原生工具**，改由「Document Skill」机制扩展：

- Skill = `SKILL.md`（元数据 + 指令）+ 可执行脚本（Python / Node）+ 资源。
- Agent 按 skill 指引，在**沙箱脚本执行器**中运行脚本完成任务（对标 Anthropic / Cursor 的 skill 范式）。
- 执行载体定为**脚本运行时**（已与产品负责人确认）；该运行时作为可选 / 后续引入的「skill 引擎」，MVP 仅在架构上预留接口，不实现执行器。

## 术语

- **项目（Project）**：用户选定的一个本地目录，是操作与沙箱的基本单位。
- **会话（Session）**：项目下的一段独立对话上下文。
- **工具调用链（Tool Call Chain）**：一次回答过程中 Agent 发起的工具调用序列，右栏可视化。
