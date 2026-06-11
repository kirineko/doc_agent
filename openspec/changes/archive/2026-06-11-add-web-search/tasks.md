## 1. 依赖与模块骨架

- [x] 1.1 `Cargo.toml` 添加 `tavily = "2.1"`
- [x] 1.2 新增 `src-tauri/src/tools/web.rs`（`web_search` / `web_extract` ToolSpec 占位）
- [x] 1.3 `tools/mod.rs` 导出 `web` 模块

## 2. ToolRegistry 与 ToolContext

- [x] 2.1 `ToolContext` 增加 `secrets: &'a Secrets` 字段，更新所有 `ToolContext::new` 调用点
- [x] 2.2 `ToolRegistry::execute` 改为 async；本地工具用 async 包装 sync handler
- [x] 2.3 `definitions(&self, include_web: bool)` 过滤 `web_*` 工具；`default_tools()` 注册两个 web 工具
- [x] 2.4 `loop_runner`：`has_api_key("tavily")` → `definitions(include_web)`；`execute(...).await`

## 3. Tavily 工具实现

- [x] 3.1 实现 `web_search`：Tavily `answer()`（或 `call` + answer 选项），参数校验与 `max_results` 封顶
- [x] 3.2 实现 `web_extract`：Tavily `extract()`，1–5 URL 校验与正文截断
- [x] 3.3 `build_working_messages`：Key 存在时追加 Web 能力 system prompt 片段
- [x] 3.4 `tools/tests.rs`：无 Key / 空参数 / URL 越界等错误分支单测

## 4. 前端配置 UI

- [x] 4.1 新增 `WebSearchSection.tsx`（provider `"tavily"`，复用 Key 保存交互）
- [x] 4.2 侧栏挂载 Web 搜索区块（与 `ApiKeySection` 分离）
- [x] 4.3 `useWorkspace`（或等价 hook）加载 `has_api_key("tavily")` 供摘要展示

## 5. 前端工具链

- [x] 5.1 `toolLabels.ts` 添加 `web_search` / `web_extract` 中文标签
- [x] 5.2 `toolLabels.test.ts` 的 `EXPECTED_TOOLS` 同步更新

## 6. 验证

- [x] 6.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [x] 6.2 `npm run typecheck && npm test && npm run build`
