# web-search Specification

## Purpose
TBD - created by archiving change add-web-search. Update Purpose after archive.
## Requirements
### Requirement: Tavily API Key 安全存储
系统 SHALL 将 Tavily API Key 以 provider `"tavily"` 存入现有 Secrets 存储（与模型 Key 相同机制），不以明文写入 SQLite 或 Agent 事件日志；界面与日志 MUST NOT 回显明文 Key。Tavily Key 的配置 UI MUST 位于 Header「密钥与服务」Drawer 的搜索服务分区。

#### Scenario: 保存 Tavily Key
- **WHEN** 用户在 Header 密钥 Drawer 的 Tavily 分区输入 API Key 并保存
- **THEN** 系统调用 `set_api_key("tavily", ...)` 持久化，侧栏 Web 搜索摘要变为「已启用」

#### Scenario: 清空 Tavily Key
- **WHEN** 用户在密钥 Drawer 清空已保存的 Tavily Key
- **THEN** Key 从 Secrets 移除，Web 搜索能力对该用户不可用，侧栏摘要变为「未启用」

#### Scenario: 启动即可配置 Tavily
- **WHEN** 用户打开应用且尚未选择项目
- **THEN** 仍可通过 Header 密钥 Drawer 配置 Tavily Key

### Requirement: Web 搜索条件启用
系统 SHALL 仅在 `has_api_key("tavily")` 为 true 时向 LLM 暴露 `web_search` 与 `web_extract` 工具定义；无 Key 时 MUST NOT 将上述工具包含在 tool 列表中。

#### Scenario: 未配置 Key 时工具不可见
- **WHEN** 用户未配置 Tavily Key 并发送需要外部信息的问题
- **THEN** 模型请求中不包含 `web_search` / `web_extract`，Agent 仅使用本地工具

#### Scenario: 配置 Key 后工具可见
- **WHEN** 用户已保存 Tavily Key 并开始新回合
- **THEN** 模型 tool 列表包含 `web_search` 与 `web_extract`

### Requirement: web_search 工具
系统 SHALL 提供 `web_search` 工具，通过 Tavily Answer API 根据查询返回合成摘要与搜索结果列表；handler MUST 在 Key 缺失时返回明确错误（防御性）。

#### Scenario: 成功搜索
- **WHEN** 模型调用 `web_search` 且参数 `query` 非空、Tavily Key 已配置
- **THEN** 系统返回 JSON，至少包含 `query` 与 `results` 数组（每项含 `title`、`url`、`content`）；若 API 提供 `answer` 字段则一并返回

#### Scenario: 空查询被拒绝
- **WHEN** 模型调用 `web_search` 且 `query` 为空或仅空白
- **THEN** 系统返回错误结果，不发起 Tavily 请求

#### Scenario: 结果数量上限
- **WHEN** 模型传入 `max_results` 大于 10
- **THEN** 系统按 10 封顶后再请求 Tavily

### Requirement: web_extract 工具
系统 SHALL 提供 `web_extract` 工具，通过 Tavily Extract API 从 1–5 个 URL 抽取正文内容。

#### Scenario: 成功抽取单 URL
- **WHEN** 模型调用 `web_extract` 且 `urls` 含一个有效 HTTP(S) URL
- **THEN** 系统返回 `{ results: [...] }`，每项含 `url` 与正文 `content`

#### Scenario: URL 数量越界
- **WHEN** 模型传入超过 5 个 URL 或空数组
- **THEN** 系统返回参数错误，不发起 Tavily 请求

#### Scenario: 过长正文截断
- **WHEN** 某 URL 抽取的正文超过配置上限（如 8000 字符）
- **THEN** 返回截断后的 content 并标注 `truncated: true`

### Requirement: Web 搜索不阻断对话
Tavily Key 未配置 MUST NOT 阻止用户发送消息或使用模型 Provider；仅 Web 工具不可用。

#### Scenario: 无 Tavily Key 仍可聊天
- **WHEN** 用户已配置 DeepSeek Key 但未配置 Tavily Key
- **THEN** 发送消息正常进入 Agent loop，不出现 Tavily 相关的 send blocker

