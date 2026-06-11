# 提案：Agent Web 搜索能力（add-web-search）

## Why

doc-agent 当前工具链仅覆盖项目内文件与 Office 文档操作，Agent 无法获取项目外的实时信息（法规更新、公开资料、网页正文等）。用户需要可配置的 Web 搜索能力：保存 Tavily API Key 后自动启用，让模型在需要时搜索互联网或抽取指定 URL 正文。

## What Changes

- 新增 2 个 Agent 工具：`web_search`（Tavily Answer API，返回合成摘要 + 搜索结果）与 `web_extract`（Tavily Extract API，从 URL 抽取正文）。
- 复用现有 `Secrets` 体系，以 provider `"tavily"` 存储 API Key；**有 Key 即自动启用**，无 Key 时工具不出现在模型 tool 列表中。
- 侧栏新增独立于「API Key（模型）」的 **「Web 搜索 (Tavily)」** 配置区块。
- `ToolRegistry` 支持按能力过滤 tool definitions；`loop_runner` 在发起 LLM 请求前根据 `has_api_key("tavily")` 决定是否暴露 web 工具。
- 工具执行层引入首个**异步网络工具**：扩展 `ToolContext` 携带 secrets，`execute` 改为 async（`loop_runner` 已是 tokio 上下文）。
- 配置 Tavily Key 后，system prompt 动态追加 Web 搜索能力说明。
- 新增依赖 `tavily = "2.1"`（官方 Rust SDK，含重试）。
- **排除**：独立 on/off 开关（有 Key 即启用）、Mock Provider 模拟 Web 搜索、搜索结果写入项目文件、自定义搜索引擎切换、按会话计费统计 UI。

## Capabilities

### New Capabilities

- `web-search`: Tavily Key 配置、条件启用、web_search / web_extract 工具契约与错误行为。

### Modified Capabilities

- `agent-loop`: 条件 tool 注册、异步工具执行、网络工具不受沙箱约束的例外说明、system prompt 动态注入。
- `workspace-ui`: 侧栏独立 Web 搜索配置区块、工具链中文标签。

## Impact

- **Rust**：新增 `src-tauri/src/tools/web.rs`；修改 `registry.rs`、`loop_runner.rs`、`state` 不变；`Cargo.toml` 加 `tavily`。
- **前端**：新增 `WebSearchSection.tsx`（或等效组件）；`Sidebar` 挂载；`toolLabels.ts` + 测试同步；`useWorkspace` 加载 `has_api_key("tavily")` 状态（仅 UI 摘要，不阻断发送）。
- **IPC**：复用现有 `set_api_key` / `has_api_key` / `clear_api_key`，provider 传 `"tavily"`。
- **风险**：Tavily 按请求计费；Agent 多轮 loop 可能连续搜索 → handler 限制 `max_results` 上限；网络失败须明确错误信息且不 panic。
