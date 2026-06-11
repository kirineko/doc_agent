## ADDED Requirements

### Requirement: 网络工具条件注册
系统 SHALL 支持按运行时条件过滤注册到 LLM 的工具子集；当 Tavily Key 未配置时，`web_search` 与 `web_extract` MUST 从 tool definitions 中排除。

#### Scenario: 过滤在每回合生效
- **WHEN** Agent loop 开始处理用户消息并构造 LLM 请求
- **THEN** 系统根据当前 `has_api_key("tavily")` 决定是否包含 web 工具，而非启动时静态固定

### Requirement: 异步工具执行
系统 SHALL 支持异步工具 handler；`loop_runner` 在沙箱/Secrets 上下文内 await 工具执行结果，网络 I/O MUST NOT 阻塞 tokio worker 以外的同步阻塞调用（禁止在 async 上下文中对 Tavily 使用 `block_on`）。

#### Scenario: web 工具异步完成
- **WHEN** 模型调用 `web_search` 且 Tavily API 需要网络等待
- **THEN** loop 异步等待 handler 完成后再 persist tool 结果并继续下一轮 LLM

### Requirement: Web 能力 system prompt 注入
系统 SHALL 在 Tavily Key 已配置时，于 system prompt 追加简短说明：可用 `web_search` 获取外部信息、可用 `web_extract` 读取已知 URL 正文。

#### Scenario: 有 Key 时 prompt 含 Web 说明
- **WHEN** 用户已配置 Tavily Key 且 Agent 构造 system 消息
- **THEN** system 内容包含 Web 搜索工具的使用提示

#### Scenario: 无 Key 时不注入
- **WHEN** 用户未配置 Tavily Key
- **THEN** system 消息不包含 Web 搜索相关说明

## MODIFIED Requirements

### Requirement: 工具在目录沙箱内执行
系统 SHALL 通过统一的工具分发器执行所有工具调用；**文件系统类工具**对文件系统的访问被限定在当前项目根目录内。已启用的 Web 搜索类工具（`web_search` / `web_extract`）作为例外 MAY 访问外部 HTTP 服务：其 URL 与查询参数 MUST NOT 被校验为项目相对路径，且成功执行时 `tool_result.changed_paths` MUST 为空。

#### Scenario: 越界路径被拒绝
- **WHEN** 模型请求的工具参数包含指向项目根目录之外的路径
- **THEN** 系统拒绝执行并返回错误结果，循环继续而不中断

#### Scenario: web_search 无 changed_paths
- **WHEN** `web_search` 成功返回
- **THEN** 对应 `tool_result` 的 `changed_paths` 为空或省略
