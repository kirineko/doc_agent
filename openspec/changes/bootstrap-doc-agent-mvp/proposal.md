# 提案：搭建 doc_agent MVP 基础

## 为什么（Why）

`spec.md` 定义了一个面向办公人员的桌面 Office 文档 Agent。当前仓库为空（仅有需求文档），需要一个**可运行的 MVP 骨架**来验证核心价值闭环：在本地目录沙箱内，由 AI Agent 自主读取 / 编辑 / 生成 Office 文档。

经过技术调研与 spike 验证，已确定纯 Rust（Tauri 2 + 自研 Agent Loop）路线可行，且满足「体积尽量小」。同时确认 **PPT 生成在 Rust 生态下视觉保真不达标**，因此 MVP 聚焦「Agent + 文件系统 + Word / Excel」，把复杂文档能力（含 PPT）留给后续的 **Document Skill（脚本型）** 机制。

本提案的目标是**先把地基做扎实**：跑通 Agent Loop、基础 Office 工具、项目 / 会话 / 持久化、三栏 UI，形成一个端到端可用的最小产品。

## 改了什么（What Changes）

### 纳入 MVP 范围

- **桌面应用骨架**：Tauri 2 + React/TS，三栏布局（左：项目 / 会话 / 模型配置；中：会话 Markdown 流式；右：工具调用链）。
- **项目与会话**：选择目录作为项目；项目下多会话、上下文隔离；会话历史与工具调用持久化到 SQLite。
- **模型配置**：DeepSeek V4 Flash / V4 Pro、Kimi K2.6 切换；thinking 开关；DeepSeek 思考强度（high / max），Kimi 仅开关。API Key 安全存储。
- **Agent Loop**（自研 `reqwest` + SSE）：统一 OpenAI 兼容 Provider；流式输出 `reasoning_content` 与 `content`；多轮工具调用；工具调用轮 `reasoning_content` 回填。
- **原生工具（目录沙箱内）**：
  - 文件系统：列目录 / 读 / 写 / 搜索。
  - Office 读取：任意 Word / Excel / PPT / 旧格式 → Markdown（`office_oxide`）。
  - Word：生成（`docx-rs`）、保格式编辑（`office_oxide` `EditableDocument`）、Markdown→Word（`create_from_markdown`）。
  - Excel：生成 / 写单元格（`umya-spreadsheet`）、快速读取（`calamine`）。
- **安全沙箱**：所有文件操作路径强制限定项目根目录内。
- **交互体验**：Markdown 渲染（代码高亮 / 表格 / 数学）、工具调用链卡片（名称 / 参数 / 状态 / 结果 / 耗时）。

### 明确排除（非本提案范围）

- **PPT 生成 / 精排**：MVP 仅支持「PPT 读取 → Markdown」，不生成 PPT。
- **Document Skill 执行器**：仅在架构与工具接口层预留，不实现脚本运行时与沙箱执行器。
- 多项目跨会话记忆共享、协同 / 云同步、自动更新、插件市场等。

## 影响（Impact）

- 这是**奠基性变更**：从空仓库建立完整工程结构（`src-tauri/` Rust 后端 + `src/` React 前端 + SQLite schema）。
- 引入受影响能力（spec deltas）：`agent-loop`、`model-config`、`office-tools`、`project-session`、`workspace-ui`。
- 为后续 `document-skill`（脚本型）扩展预留工具 schema 与执行接口。
- 依赖外部模型 API（DeepSeek / Kimi），需要用户提供 API Key 方可端到端联调。
