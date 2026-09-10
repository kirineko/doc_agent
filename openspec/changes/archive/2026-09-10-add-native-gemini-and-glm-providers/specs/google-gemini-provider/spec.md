## Purpose

定义 Gemini OpenAI 兼容 Chat Completions 接入及轻量签名持久化；不存在需要兼容的 Interactions 会话。

## ADDED Requirements

### Requirement: Google Chat Completions 请求
系统 SHALL 使用 google/gemini-3.8-flash、官方 /v1beta/openai/chat/completions 地址与 Bearer 鉴权，复用 OpenAiCompatClient。系统 MUST NOT 发送 Interactions 请求或保留其回退/转换分支。

#### Scenario: 共用请求格式
- **WHEN** 发起 Google 主聊天或辅助请求
- **THEN** 使用 messages/tools/stream、合法思考配置与显式 max_tokens；图片使用 image_url Data URL

#### Scenario: 完整请求大小
- **WHEN** 序列化的 Google 请求超过产品保守限制20,000,000字节
- **THEN** 请求前明确拒绝，不静默删除历史图片或签名

### Requirement: 工具 schema 保持
系统 SHALL 原样传递完整工具 schema，包括 oneOf/not；MUST NOT 保留 Interactions 的约束剥离器。本地工具校验 SHALL 继续执行。

#### Scenario: 互斥参数
- **WHEN** 工具 schema 包含 code/path 互斥约束
- **THEN** 请求保持原 schema，本地仍拒绝两个参数同时提供或均缺失

### Requirement: 流式工具与终态
系统 SHALL 共享字节安全的 SSE 解码，按 index 或 id 区分调用；无 index 的同名并行工具 MUST 保持独立。完整 stop/tool_calls 响应 SHALL 结合调用列表决定后续动作。

#### Scenario: 无 index 的双调用
- **WHEN** 两个工具增量分别带不同 id 而无 index
- **THEN** 累积为两个调用，UI 进度有不同本地 index，参数不混合

#### Scenario: 签名与 UTF-8 跨 frame
- **WHEN** UTF-8 字符、LF/CRLF frame 或参数分散在网络块中
- **THEN** 完整解码后累积，不使用有损 UTF-8，签名作为元数据保留

#### Scenario: 截断调用
- **WHEN** length 响应带完整或残缺工具参数
- **THEN** 不执行本批任何工具，移除调用元数据，正文及截断提示可持久化，不报成功

#### Scenario: 断流或取消
- **WHEN** 无有效终态 EOF、错误事件，或取消发生于等待响应/读取流期间
- **THEN** 返回明确错误或 Cancelled，不等待下一网络包，不执行部分调用

### Requirement: 轻量签名存储与恢复
系统 SHALL 复用唯一可空 messages.provider_state_json 保存版本化的消息/工具 extra_content，工具元数据与工具列表同序。MUST NOT 保存 steps、正文副本或 wire ID 映射；MUST NOT 将签名塞进 ID、参数或可见文本。模型响应与工具结果 SHALL 使用相同的本地 ID。

#### Scenario: 同序恢复与 ID 规范化
- **WHEN** 双工具调用的 ID 因冲突被规范化，持久化后重载
- **THEN** 元数据仍附着于原调用顺序，原始签名回传，双工具和第三工具可继续

#### Scenario: 人工澄清恢复
- **WHEN** clarify pending 经重载后提交回答
- **THEN** 签名从数据库恢复，答案是原工具结果，不能当成新 user turn

#### Scenario: 必要签名缺失
- **WHEN** 工具 assistant 缺少首调用签名、元数据数量不匹配或状态损坏
- **THEN** 请求前明确拒绝，不伪造签名；普通文本 assistant 可无元数据

#### Scenario: 状态不泄露
- **WHEN** 历史被显示、记录 Debug 或发送给其他 provider
- **THEN** 不出现 Google 内部元数据或签名

### Requirement: 思考摘要与用量
系统 SHALL 将 Google 的 thought 标记与跨块 <thought> 定界内容分流为 ReasoningToken；正常答案为 ContentToken，签名不展示。系统 SHALL 发送 extra_body.google.thinking_config，允许摘要为空，支持兼容 response_format。

#### Scenario: 摘要与正文同响应
- **WHEN** content 同时包含思考摘要和正文
- **THEN** 两者分别持久化/展示，标签不泄漏，普通正文中的同名标签不被无条件剥离

#### Scenario: 含隐藏思考用量
- **WHEN** prompt_tokens=38、completion_tokens=345、total_tokens=1353
- **THEN** 记录输入38、输出1315、总量1353，不重复累计 usage

### Requirement: Gemini 系统代理与 VPN

系统 SHALL 在 macOS/Windows 桌面启动时自动遵循系统 HTTP/HTTPS 代理，支持 Clash Verge 系统代理与全局模式组合，不依赖 shell 环境变量。系统 SHALL 兼容操作系统接管的 TUN/VPN，所有 Gemini 主请求及辅助请求 MUST 使用一致策略；MUST NOT 硬编码代理端口、关闭 TLS 校验或在选定代理故障时自动回退直连。无应用层代理时允许经操作系统网络栈访问。

#### Scenario: 桌面启动跟随 Clash
- **WHEN** 用户从桌面启动应用，没有代理环境变量，Clash 已启用系统代理及全局模式
- **THEN** Gemini 请求通过系统配置的实际代理端口，HTTPS 使用 CONNECT，代理记录可确认 Google 目标连接，流式文本与工具闭环可用

#### Scenario: TUN 接管
- **WHEN** 系统 HTTP 代理关闭而 TUN/VPN 已正确接管 Google 目标路由
- **THEN** Gemini 请求沿系统路由工作，不要求另填应用代理地址

#### Scenario: 代理配置变更
- **WHEN** 用户在应用启动后开启系统代理或修改监听端口
- **THEN** 下一次 Gemini HTTP 调用读取新配置，不要求重启应用，不主动重放正在进行的 SSE

#### Scenario: 辅助请求一致
- **WHEN** Gemini 执行工具续答、恢复、标题生成、压缩或视觉子调用
- **THEN** 均使用与主聊天一致的自动代理策略，不存在单独直连路径

#### Scenario: 代理故障
- **WHEN** 已选择的代理拒绝连接或 CONNECT 失败
- **THEN** 返回明确网络错误，不回退直连、不误报 Key 错误，不泄露密钥与代理凭据

#### Scenario: 显式环境设置与绕过
- **WHEN** 进程携带 HTTP 代理环境设置或匹配目标的 NO_PROXY
- **THEN** 遵循锁定 reqwest 的环境代理/绕过语义，错误提示能引导用户检查设置，不强制忽略用户绕过配置

#### Scenario: 无代理也可调用
- **WHEN** 未配置代理且操作系统网络可访问 Google
- **THEN** 正常调用，不要求安装 Clash 或配置虚假的本地端口

#### Scenario: 用户入口说明
- **WHEN** 用户查看 Google 密钥区域
- **THEN** 显示自动跟随系统代理或 VPN，以及 Clash 需开启系统代理或 TUN 的简短说明，无需新增代理配置字段
