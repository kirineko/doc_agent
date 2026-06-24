## MODIFIED Requirements

### Requirement: 工具在目录沙箱内执行
系统 SHALL 通过统一的工具分发器执行所有工具调用；**文件系统类工具**对文件系统的访问被限定在当前项目根目录内。已启用的 Web 搜索类工具（`web_search` / `web_extract`）作为例外 MAY 访问外部 HTTP 服务：其 URL 与查询参数 MUST NOT 被校验为项目相对路径，且成功执行时 `tool_result.changed_paths` MUST 为空。`image_download` 作为**混合工具**：其图片 URL MUST NOT 被校验为项目相对路径（外部网络资源），但其输出目录与写入文件 MUST 限定在项目根目录内（经 `resolve_for_write` 校验），成功下载的本地路径 MUST 进入 `tool_result.changed_paths`。

#### Scenario: 越界路径被拒绝
- **WHEN** 模型请求的工具参数包含指向项目根目录之外的路径
- **THEN** 系统拒绝执行并返回错误结果，循环继续而不中断

#### Scenario: web_search 无 changed_paths
- **WHEN** `web_search` 成功返回
- **THEN** 对应 `tool_result` 的 `changed_paths` 为空或省略

#### Scenario: image_download URL 不做路径校验且输出在沙箱内
- **WHEN** `image_download` 以外部 http(s) URL 下载图片
- **THEN** URL 不被当作项目相对路径校验，但下载文件只写入项目根内的输出目录，成功路径进入 `changed_paths`

## ADDED Requirements

### Requirement: image_download 无条件注册
系统 SHALL 将 `image_download` 工具无条件包含在每回合的 LLM tool definitions 中，不依赖任何 API Key，且 MUST NOT 受 `include_web`（Tavily Key）开关影响。

#### Scenario: 无 Tavily Key 时仍注册 image_download
- **WHEN** 用户未配置 Tavily Key，Agent loop 构造 LLM 请求
- **THEN** tool definitions 不含 `web_search` / `web_extract`，但仍包含 `image_download`
