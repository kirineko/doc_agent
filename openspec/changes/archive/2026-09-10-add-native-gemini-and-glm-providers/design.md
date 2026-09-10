## Context

本变更接入 Gemini/GLM 并更新模型目录。Google 最终采用 OpenAI Chat Completions 兼容接口，共享现有客户端、消息与工具循环。用户确认 Interactions 实现尚未发版，不存在需要兼容的会话；删除其编码器、步骤累积器与 wire/local ID 映射，不保留双协议或迁移分支。

### 2026-09-09 实测依据

使用环境变量 GOOGLE_API_KEY、合成文本/工具和内存生成的图片；没有发送项目内容或读取产品数据库。

| 探测 | 结果与设计影响 |
|---|---|
| 文本、双工具→第三工具→错误结果→最终回答→追问 | 兼容接口成功，共享 messages/tool_calls/tool 格式 |
| 删除 tool_calls.extra_content.google.thought_signature | HTTP 400；签名持久化不可删除 |
| JSON 经内存 SQLite 往返，工具 ID 改本地 UUID | 续答成功，不需要 wire ID 映射；不等于产品恢复验收 |
| 流式并行工具 | 每个调用有 id，没有 index；结束标记为 stop；必须适配 |
| 思考摘要 | content 中带 thought 标记和 <thought> 标签；没有 reasoning_content，需分离 |
| high 用量 | prompt=38/completion=345/total=1353，输出统计包含 total-prompt |
| low/medium/high、两张 Data URL 图片、json_object | 接受，图片识别 red/blue |
| 原 oneOf/not 合成结构 | 原样接受，删除定向 schema 剥离；不能据此宣称全部 schema 服务端约束受支持 |
| 小输出预算 | finish_reason=length；截断不执行工具，不成功完成 |

官方来源：[OpenAI compatibility](https://ai.google.dev/gemini-api/docs/openai)、[签名回传](https://ai.google.dev/gemini-api/docs/generate-content/thought-signatures#signatures-for-openai-compatibility)。模型目录参数仍沿用本变更已核验的快照，协议探测不代表所有平台或打包应用已验收。

## Goals / Non-Goals

目标：共享 Chat Completions 管线、减少重复状态、保持签名/恢复/压缩正确性、保留既有 provider 行为。范围不包含 Interactions 会话兼容、Files API、音视频、远端内置工具、服务端会话、模型自动切换或新增代理 UI。

## Decisions

### D1. 单一 Chat Completions 管线

所有真实 provider 复用 OpenAiCompatClient 的请求、Bearer 鉴权、SSE 和回调；GeminiProvider 每次调用构建遵循系统代理的 HTTPS 客户端，传入完整地址 https://generativelanguage.googleapis.com/v1beta/openai/chat/completions。Google 的小型适配层仅处理请求参数、额外元数据、思考摘要与结束语义。stream 和非 stream 路径应用相同的参数及元数据契约。

不发送 Interactions 的 input/steps/store/previous_interaction_id；不用 x-goog-api-key。其他 provider 地址与参数不变。

### D2. 模型目录与能力真源

扩展现有 ModelCatalog，包含 `selectable`、`availability`（available/retired）、`supports_vision`、`supports_thinking_toggle`、`thinking_efforts`、默认思考配置、`max_context`、可选 `max_output_tokens`。`supports_effort` 可继续输出以兼容前端，但必须由 efforts 非空推导，不能维护矛盾值。

`list_models` 返回可识别的真实模型（包含历史条目），新建选择器仅过滤 selectable。Mock 只供显式测试，不放入公共新建目录。前端启动 fallback 含相同元数据，并有与 Rust catalog 的一致性测试。

| 新建顺序 / id | provider | vision | toggle | efforts | 默认 |
|---|---|---|---|---|---|
| deepseek-flash | deepseek | true | true | low,high,max | enabled/high |
| mimo-v2.5 | mimo | true | true | 空 | enabled |
| mimo-v2.5-pro | mimo | false | true | 空 | enabled |
| kimi-k3 | kimi | true | false | low,high,max | enabled/max |
| gemini-3.8-flash | google | true | false | low,medium,high | enabled/medium |
| glm-5.3-flash | zhipu | true | false | low,high,max | enabled/max |

DeepSeek Flash：产品 id、会话写入和请求 `api_model` 统一为 `deepseek-flash`，不绑定即将替换的 V4 快照名。展示名 **DeepSeek Flash**。历史会话或草稿中的 `deepseek-v4-flash` MUST 仍解析为同一模型，不批量 UPDATE `sessions.model`。2026-09-10 用环境 Key 实测：`deepseek-flash` / `deepseek-v4-flash` 均可用；对 Flash 发送 1×1 PNG 得到颜色识别，故 `supports_vision=true`，工具列表含 `image_read`。独立实验模型 `deepseek-v4-flash-vision-exp` 本轮不加入新建目录。

上下文策略：DeepSeek/MiMo/GLM/Kimi K3 采用 1,000,000 的产品保守预算；Google 采用官方精确 1,048,576。旧 Kimi K2.6 保留 256,000，其他历史预算保留原值。不得把 1M 市场描述混同为所有厂商都确认的 1,048,576。

输出元数据：DeepSeek 384K 按 393,216 记录，MiMo/GLM 128K 按 131,072 记录，Google 65,536，Kimi K3 API 参数上限 1,048,576。它是参数上界，不代表有输入时可实际输出同样长度；实现前按官方接口定义核对 K/M 的整数解释。常规主 loop 不主动申请该最大值，继续省略输出限制。显式设置必须受模型上界约束；上下文余量不足走现有压缩或明确错误，不自动静默减少用户配置。

### D3. 历史模型兼容

| 旧 id | selectable | availability | 行为 |
|---|---|---|---|
| deepseek-v4-pro | false | available | 原模型、原路由、原开关与 high/max 历史值继续有效 |
| kimi-k2.6 | false | available | 原模型、thinking.type/keep:all，保留关闭思考能力 |
| mimo-v2.5-pro-ultraspeed | false | retired | 可读取；send/resume/manual compact 不发远端请求，提示新建 MiMo Pro 会话 |

不将 `kimi-k2.6` 解析成 `KimiK3`；不对 sessions.model 做 UPDATE。未知名称可用原字符串展示历史，但发送、resume、压缩、辅助调用明确拒绝，绝不 fallback Mock。后台标题对 retired/unknown 跳过，避免反复无效请求。

新建/空会话修改只允许 selectable 模型；已有非空会话继续锁定。localStorage 中“上次新建配置”指向旧模型时整体回退既有默认 DeepSeek Flash enabled/high；这不影响已有 session。旧空会话可通过新建目录显式选择新模型。

### D4. UI 与配置规范化

UI 示例（示意，不新增术语到用户流程）：

```text
Gemini 3.8 Flash
思考强度  [low | medium* | high]
此模型始终启用思考

MiMo v2.5
[x] 启用思考模式

DeepSeek Flash
[x] 启用思考模式
思考强度  [low | high* | max]

MiMo v2.5 Pro Ultraspeed（历史模型）
该模型已不可调用。请新建会话选择 MiMo v2.5 Pro。
```

模型切换时应用目标模型默认配置，而不是继承不兼容的 max/off。同一模型有效草稿值保留。Kimi K3/GLM/Gemini 的 IPC 创建/更新请求若显式传 disabled 或非法档位则拒绝，不能假装关闭成功；遗漏配置则由后端填该模型默认。MiMo 无 effort，保留数据库已有字符串但不发送、不显示；缺省采用既有 high 占位以避免列迁移。

标签格式统一：可关闭且关闭时显示“思考关闭”；有档位显示选中档位；始终思考模型不显示虚假的关闭状态。只读历史标签由历史 catalog 解析。

密钥 Drawer 增加 Google Gemini（key=google）和智谱 GLM（key=zhipu）。API_PROVIDERS、首次启动 Key 状态、缺 Key 跳转同步覆盖，无项目也可设置。已有 `[api_keys]` 保持；不得自动保存调研中提供的凭据。新 provider 不加入余额请求列表。仅 Google/Zhipu Key 可正常聊天，智能建议仍遵循既有 DeepSeek 缺 Key 行为。

### D5. 请求参数与消息

Google body 使用 model/messages/tools/stream/stream_options.include_usage。开启摘要时使用 extra_body.google.thinking_config={thinking_level:low|medium|high,include_thoughts:true}，不同时设置 reasoning_effort。显式输出上限用 max_tokens，主 loop 省略上限。标题、压缩和视觉子调用使用同一 provider，保留辅助 low 与预算策略。

message.content 为正文；reasoning_content 存储用于 UI，但 Google 回传时不发送该非 Google 字段。user 图片复用 image_url Data URL。工具定义原样发送（包括 oneOf/not），本地工具参数校验仍为执行门槛，不静默删约束。response_format 原样传入兼容接口；不增加产品结构化输出入口。

完整 UTF-8 JSON 仍使用 20,000,000 字节的保守产品限制（非声称已实测精确服务端边界），包含全部历史、图片与签名；超限请求前拒绝，不上传 Files、不静默删图。

### D6. SSE、思考与终态

共享字节 frame 解码器处理 UTF-8 跨块、LF/CRLF/混合换行、注释、多行 data 与 data: 无空格。使用相同累积器收束文本与工具。工具有 index 时按 index；无 index 时按 id 分配稳定本地槽位；缺少两者且存在多个候选调用则明确报错，不能都合并到 0。UI 工具进度携带规范化 index，不包含额外元数据。

Google 摘要用 extra_content.google.thought 标记与 <thought>...</thought> 定界分流，支持标签跨 chunk；普通正文中的同名文字不能被无条件删除。签名作为完整不透明值保存，不当文本增量拼接、不展示或日志输出。显示用 thought 布尔标记不回传为正文的语义属性。

Google 接受 stop 或 tool_calls 作为完整终态，结合工具列表判断是否继续；每个调用名、参数对象和必需签名均完整才交给 loop。length 规范化为 incomplete，清空调用及其元数据，保存正文和截断提示；循环在任何工具处理前进入截断分支。错误事件、未知终态或无终态 EOF 返回错误。读流、等待响应头和错误响应体都可取消；不依赖服务器继续发包。

用量保留服务端 total，Google completion=max(服务端 completion,total-prompt)，包含隐藏思考；其他 provider 原统计不变。重复 usage 替换而非累加。

### D7. 数据库字段评估与轻量状态

保留 messages.provider_state_json 这一可空 TEXT 列；不新增第二列，不改 sessions/tool_calls 表。当前已有字段没有签名位置，删除此列而只存内存会破坏重启/clarify 恢复。也不把签名编码进 ID、正文、reasoning_content 或工具参数。旧 DeepSeek/MiMo/Kimi 数据保持 NULL 与原查询语义，迁移幂等并明确报告失败。

字段只保存 {protocol:google_openai,version:1,model,message_extra_content,tool_extra_content}。message_extra_content 为对象（无扩展时为空对象）；tool_extra_content 是与该 assistant 工具列表同序的对象数组（无扩展时空对象占位）。无正文、摘要、参数、完整步骤或 wire ID 副本。工具记录按消息 seq、同消息内插入 rowid 排序，保证持久化/读取顺序稳定；ID 正常去重不影响同序元数据。第一项工具必须有有效非空签名，其余项允许没有；有扩展时原样回传。

普通文本 assistant 允许无状态，工具 assistant 缺失签名或元数据数量不匹配时请求前报错。未知版本/协议/损坏 JSON 明确失败，无旧 Interactions 转换路径。内部状态不进入 IPC 历史/事件/Debug 日志，不注入其他 provider 请求。不要为消除一个可空列而重建用户数据库。

### D8. Loop 与恢复

loop 不再解析 GoogleProviderState 或绑定 wire ID。仍先持久化 assistant 正文、可读思考、轻量状态，再执行工具。工具结果使用相同本地 call id 和普通字符串 content；同批全部调用先于全部结果。

从 store 读取后，Google 请求编码器按工具顺序附加 extra_content；clarify 答案是匹配 tool 结果，不构造新 user turn。现有 pending 恢复机制不扩大成任意崩溃点重跑。产品 store 往返测试必须验证并行同名调用、ID 碰撞规范化、第三调用与人工回答，证明签名确实到达下一请求。

### D9. 压缩

保留 Google 当前真实 user 输入起的完整活动工具链及轻量状态，避免仅保留最后一对。归档与读取仍跟随消息；摘要输入只包含正文/可读思考/工具文本，不发送额外元数据、签名或附件二进制。pending 估算忽略签名字节，不复制计算正文。摘要作为普通 user 输入，新回复按正常兼容协议保存状态。

### D10. 实现边界

删除 gemini/schema.rs 与 Interactions 累积器。gemini/{mod,request,state,stream}.rs 分别负责工厂入口、请求元数据附加、轻量状态、摘要分离。公共 SSE frame/工具累积可拆分成独立小模块。OpenAiCompatClient 共用请求构建和传输；Google 分支收敛到 provider 层，loop/store 不知道 Google 步骤类型。

### D11. 验证和交付

离线：真实 store 往返、clarify、压缩、UTF-8/无 index/思考标记/签名/取消/截断/EOF、全工具目录 schema 原样保持、旧 provider 请求快照、旧库重复迁移及 IPC 脱敏。在线：显式环境开关开启合成 Gemini provider smoke，调用产品 Rust 客户端并通过真实 store 重载，失败必须使测试失败，缺 Key 不能冒充成功。

运行 npm typecheck/test/build、cargo fmt/clippy/test 与 OpenSpec strict validation。测试库和合成图片均为临时数据，Key 只从环境读取。完整项目 schema 远端验收与 macOS/Windows 打包 GUI/TUN 验收分别列明，不用协议探测代替。本次修改不伪造旧待办完成状态。

## Migration Plan

首次启动按原幂等方式增加唯一可空列；已存在该列时复用。不存在 Interactions 会话，不做协议迁移、自动摘要转换或旧状态兼容。现有其他 provider 历史模型/配置不变。新 Gemini 请求仅写轻量元数据。回滚到不识别新模型的旧二进制仍不支持续聊。

## 补充决策 D12：Gemini 跟随系统代理与 VPN

### 当前证据与支持边界

项目直接依赖 reqwest 0.12，锁定版本为 0.12.28，Cargo.toml 使用 `default-features=false`，仅显式开启 json/stream/rustls-tls；现有兼容客户端使用 `Client::new()`。这不足以把桌面启动时跟随系统代理当作已保证的产品能力。Cargo feature 可以被传递依赖统一开启，因此验收还必须检查实际 feature graph，不能仅凭配置推断当前二进制一定不支持代理。

[reqwest 0.12.28 features](https://docs.rs/crate/reqwest/0.12.28/features) 提供 system-proxy，用于 macOS/Windows 系统代理发现；本地对应源码也已核对。环境代理与系统代理的发现由底层 matcher 处理。[Clash Verge 快速入门](https://www.clashverge.dev/guide/quickstart.html) 区分系统代理与 TUN；全局模式只控制已进入 Clash 的连接如何出站，不自动使所有应用进入 Clash。

MVP 必须支持 macOS/Windows 的 Clash Verge「系统代理开启 + 全局模式」，以及由操作系统正确接管的 TUN/VPN。Linux 支持进程环境代理和 TUN，桌面环境各异的系统代理/PAC 自动发现不作承诺。本次不实现 PAC 脚本执行、代理订阅管理或应用内 VPN，也不要求普通桌面用户设置 shell 环境变量。

### 客户端策略

1. 在现有 reqwest 依赖中显式启用 `system-proxy`，保留 rustls 与证书校验。该特性属于已有依赖，但可能引入平台传递依赖；检查 Cargo.lock 与平台构建结果，不增加常驻代理进程。
2. Gemini 统一客户端工厂使用 reqwest 默认自动代理发现，不调用 `no_proxy()`、不硬编码端口、不绑定指定网卡/IP，不为 Google 单独配置直连。使用系统设置中实际的 HTTP/HTTPS 代理，HTTPS 通过 CONNECT 隧道发送，Google API Key 仍在 TLS 内。Clash HTTP/mixed 监听地址可能是 `http://127.0.0.1:<实际端口>`，不得假定 7890/7897。
3. 有进程环境代理时沿 reqwest 的 HTTPS_PROXY/ALL_PROXY 与 NO_PROXY（含库支持的大小写变体）语义处理；保留用户显式绕过规则。不声称环境变量永远存在或一定优先于所有系统字段；以锁定版本 matcher 的行为做契约测试。MVP 支持 HTTP 代理端点；SOCKS-only 环境代理不在本次范围，发现不支持的 scheme 应明确报错，提示使用 Clash HTTP/mixed 端口或 TUN。
4. 无应用层代理时走操作系统网络栈，TUN/VPN 是否接管由系统路由决定；不能因「没有 HTTP 代理」就断言未走 VPN。无代理且网络本身可达 Google 时也可正常调用。
5. 主聊天、工具结果续答、clarify 恢复、标题、压缩和视觉子调用都使用同一工厂策略。每次独立 HTTP 模型调用重新构建自动代理客户端，读取最新系统代理设置；正在进行的 SSE 不切换路径。这样应用启动后再开启 Clash 或修改端口，下一次调用可生效；MVP 接受连接池复用减少的成本，后续优化不能改变该行为。
6. 已选择代理但连接失败时返回网络错误，不自动重试直连。区分代理连接/CONNECT 错误、超时、TLS 错误与 Google HTTP 权限/额度错误；不给所有失败统一贴「Key 错误」标签。错误提示可建议检查 Clash 系统代理/TUN、节点及环境绕过规则，但不把检测到本地代理等价为 Google 可用。不得输出 Key、代理凭据或模型请求正文。

**对其他 provider 的影响：** Cargo feature 是同一依赖版本的共享能力，启用 system-proxy 可能使现有 `Client::new()` 也开始遵循 OS 代理。这是明确的传输行为增补：它们遵循用户系统代理与 Clash 规则，不强制直连、不强制使用某个节点；模型协议、endpoint 与密钥选择保持原契约。检查其他 reqwest 使用点并回归无代理、环境代理、系统代理三种情况，不能写成「只影响 Gemini」或通过全局 no_proxy 破坏原环境代理。Clash 全局模式可能使国内 provider 也走代理，其规则分流由用户控制。

### UI 与可验收示例

Google 密钥区域增加简短说明：「自动跟随系统代理或 VPN。使用 Clash Verge 时，请开启系统代理或 TUN。」无需新增代理输入框、持久化配置或 SQLite 字段，数据库仍仅新增 provider_state_json 一列。

示例：用户通过 Finder/开始菜单启动 Doc Agent，未设置 HTTPS_PROXY；开启 Clash Verge 系统代理并选择全局模式后，发送 Gemini 请求。Clash 连接记录应出现 `generativelanguage.googleapis.com:443`，正文与工具续答持续输出。只有 API 请求成功而没有代理侧连接证据，不能作为「经过 Clash」的充分证明。

| 验收环境 | 预期与证据 |
|---|---|
| macOS/Windows 打包 GUI 启动，无代理环境变量，Clash 系统代理+全局 | 主聊天及续答通过实际配置端口；代理连接记录与响应共同证明，不以终端 curl 替代 |
| 系统代理关闭、TUN 开启 | 全部 Gemini 调用正常；以 Clash/TUN 连接记录证明系统接管 |
| Doc Agent 启动后开启系统代理或更改端口 | 下一次模型 HTTP 调用使用新设置，已有 SSE 不被主动重放 |
| 环境 HTTP 代理与 NO_PROXY | 隔离子进程测试与锁定库语义一致，不污染测试宿主环境 |
| 代理端口不可连接、CONNECT 失败、SSE 中断 | 错误明确、不直连回退、不执行残缺工具；不把网络故障当鉴权失败 |
| 未配置代理、网络可直达 | 文本与工具循环正常，不强制用户安装 Clash |
| DeepSeek/MiMo/Kimi/GLM | endpoint/参数不变；系统代理共享影响有记录，无代理/环境代理回归通过 |

离线测试使用本地可观测 HTTP CONNECT 代理与合成 TLS/SSE 服务（测试专用 CA，只在测试客户端信任），断言目标 host/port 和数据持续流动；测试也验证代理不可用时未连接直连目标。GUI/TUN 路径需真实打包应用验收，不能由离线 mock 取代。当前补充仅完成代码与文档核对，尚未验证实现后的打包应用经 Clash 路由。
