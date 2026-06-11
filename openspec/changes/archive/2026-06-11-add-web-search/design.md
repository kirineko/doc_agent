# 设计：Agent Web 搜索（Tavily）

## Context

- 现有工具全部为**同步、本地、沙箱内**执行；`ToolContext` 仅含 `Sandbox`。
- LLM 请求已在 `agent/provider` 层用 `reqwest` + async；工具层尚无 HTTP。
- 密钥：`Secrets` 以 `provider` 字符串存于 `config.toml`（Unix 600），IPC 已具备 CRUD。
- 侧栏 `ApiKeySection` 的 `API_PROVIDERS` 从 `MODEL_OPTIONS` 推导，**不应**把 Tavily 混入模型 Key 区。

## Goals / Non-Goals

**Goals：**

- `web_search`、`web_extract` 两个工具；Tavily Key 配置后自动对 Agent 可见。
- 独立侧栏配置区；前端工具链中文标签；Rust 单测覆盖 handler 错误分支。

**Non-Goals：**

- 有 Key 但仍关闭的独立开关；Mock 模拟 Web 搜索；把搜索结果自动写入项目；多搜索引擎；会话级配额 UI。

## Decisions

### D1：`web_search` 使用 Tavily `answer()` 而非 `search()`

| API | 返回 | Agent 适用性 |
|-----|------|-------------|
| `search()` | 结果列表 + snippet | 模型需自行综合，token 多 |
| `answer()` | **合成 answer** + results + follow_up_questions | 直接可用，减少幻觉与推理步 |

**结论**：`web_search` handler 调用 `tavily.answer(query).await`，JSON 返回 `{ query, answer, results[], follow_up_questions? }`。若 API 失败再降级 `search()`（design 实现时二选一即可，优先 answer）。

可选参数：`max_results`（默认 5，上限 10）、`search_depth`（`basic` / `advanced`，默认 `basic`）通过 `SearchRequest` 传入 `call()` 若需细粒度控制。

### D2：`web_extract` 使用 Tavily `extract()`

- 入参：`urls: string[]`（必填，1–5 个 URL）
- 出参：`{ results: [{ url, raw_content, ... }] }`（按 SDK 字段精简映射，截断过长 content 如 >8000 字符并注明 truncated）
- 用途：用户给出链接、或 search 结果中需深读某页时

### D3：条件启用 — 有 Key 即注册

```text
loop_runner::run_turn
    │
    ├─ has_api_key("tavily")? ──no──► definitions(exclude_web: true)
    │
    └─ yes ──► definitions(exclude_web: false)
               + system prompt 追加 web 工具说明
```

未配置 Key 时：工具**不出现在** LLM tool 列表（优于「注册了但执行报错」）。

Tavily Key **不**参与 `getSendBlocker`；无 Key 仍可正常对话，只是无 Web 能力。

### D4：异步工具执行

`ToolRegistry::execute` 改为 `async fn execute(...)`；handler 类型改为：

```rust
type ToolHandler = fn(&ToolContext, Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send + '_>>;
```

或更简：`async fn` + 宏/包装。MVP 可仅对 `web_*` 用 async handler，其余 `async { sync_handler(...) }` 包装，避免大规模改动。

`loop_runner` 第 195 行改为 `.await`；已在 tokio runtime，**禁止** sync handler 内 `block_on`。

### D5：`ToolContext` 扩展

```rust
pub struct ToolContext<'a> {
    pub sandbox: &'a Sandbox,
    pub secrets: &'a Secrets,
}
```

`web_*` handler 读 `secrets.get_api_key("tavily")`；缺失时返回 `ToolError::Execution("Tavily API key not configured")`（防御性，正常路径不应触发）。

### D6：UI — 独立 `WebSearchSection`

- 位置：侧栏 API Key 区块**下方**（或模型配置上方），`<details>` 折叠，标题「Web 搜索 (Tavily)」
- 复用 `ProviderKeyRow` 交互模式（已保存 / 更换 / 清空），provider 固定 `"tavily"`
- 摘要：未配置 →「未启用」；已配置 →「已启用」
- **不**加入 `API_PROVIDERS` / 模型 send blocker

### D7：依赖 `tavily = "2.1"`

官方 SDK，内置 timeout / retry；项目已有 `tokio` + 间接 HTTP 栈，增量可控。不在 tools 层手写 REST。

### D8：System prompt 动态片段

Key 存在时在 `build_working_messages` 的 system 段追加：

```text
Web 搜索已启用：需要项目外实时信息时用 web_search(query)；已知 URL 需读正文时用 web_extract(urls)。
```

### D9：测试策略

- **单元**：handler 在无 key 时错误；args 校验（空 query / 空 urls / urls 超 5）
- **集成**：可选 `#[ignore]` + 环境变量 `TAVILY_API_KEY` 的 live test；CI 不跑
- **前端**：`toolLabels` 覆盖新工具名；`WebSearchSection` 可选组件测试

## 工具 I/O 契约

| 工具 | 入参 | 出参 |
|------|------|------|
| `web_search` | `query` (required), `max_results?` (default 5, max 10), `search_depth?` (`basic`\|`advanced`) | `{ query, answer?, results: [{title, url, content, score}], follow_up_questions? }` |
| `web_extract` | `urls: string[]` (1–5) | `{ results: [{ url, content, ... }] }` |

两者 `changed_paths` 均为空（非文件工具）。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| Agent 连续多次搜索，费用上升 | `max_results` 硬顶 10；tool description 提示「优先一次综合 query」 |
| Tavily API 超时 / 限流 | SDK max_retries + 明确 `ToolError::Execution` 消息 |
| `answer()` 偶发无 answer 字段 | 仍返回 `results[]`，模型可读 snippet |
| extract 正文过长撑爆 context | handler 截断 + `truncated: true` |
| 首个网络工具与「沙箱内执行」spec 张力 | agent-loop delta 明确网络工具例外 |

## Migration Plan

- 纯增量：无 DB 迁移；旧用户无 Tavily Key 行为不变。
- 部署：发版说明侧栏新配置项与 [Tavily](https://tavily.com) 注册方式。

## Open Questions

- （已决）search vs answer → **answer 为主**
- （已决）独立区块 + 自动启用
- （已决）含 web_extract
