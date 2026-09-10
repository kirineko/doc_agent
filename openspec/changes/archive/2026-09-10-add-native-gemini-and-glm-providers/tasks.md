## 1. 模型目录与配置契约

- [x] 1.1 扩展 Rust 模型 ID、Low/Medium 强度及 catalog 元数据，加入六个 selectable 和三个历史条目；用目录测试验证顺序、默认值、vision、toggle、efforts、availability 和上下文预算符合 design D2/D3。
- [x] 1.2 按官方参数定义核对 design D2 的输出上限整数，记录对应来源与最终值；用边界测试验证显式预算不超过上限，主 loop 继续省略输出上限。
- [x] 1.3 替换未知模型 fallback Mock 和未知 effort fallback high，新增按模型校验与辅助请求配置 helper；测试未知值、非法 disabled/max、遗漏默认值以及显式 Mock 测试入口。
- [x] 1.4 更新 IPC 创建/空会话修改契约，区分新建可选性与历史可调用性；测试非空会话锁定、旧 K2.6 设置保持、retired/unknown 在发送、恢复和手动压缩前明确失败。
- [x] 1.5 将 DeepSeek Flash 产品 id、展示名与请求名统一为 `deepseek-flash` 并开启视觉；`deepseek-v4-flash` 仅作历史别名；不批量改写旧会话；目录/UI/工具列表与 OpenSpec 同步。

## 2. Chat Completions provider 调整

- [x] 2.1 将兼容客户端配置改为完整 endpoint 并增加 zhipu 路由与密钥读取；请求快照验证既有三家地址不变、GLM 地址没有多余 /v1，鉴权按 provider 隔离。
- [x] 2.2 实现 GLM thinking、clear_thinking、reasoning_effort、tool_stream 和 max_tokens 编码；用请求与 SSE fixtures 验证空/非空 reasoning_content、usage、视觉及工具续答。
- [x] 2.3 新增 Kimi K3 参数编码并保留旧 K2.6 路径，修订 DeepSeek 档位和 MiMo selectable 状态；快照验证 K3 不发送 thinking、K2.6 保留原模式、MiMo 不发送 effort、Ultraspeed 不发 HTTP。

## 3. Google OpenAI 兼容重设计

- [x] 3.1 更新 proposal/design/specs，记录无 Interactions 历史、保留唯一签名元数据列的原因；移除 steps/schema 剥离/wire 映射设计。
- [x] 3.2 复用 OpenAiCompatClient 接入 Google Bearer endpoint，统一 stream/non-stream 请求与取消，保留系统代理、输出预算、图片/大小校验和 response_format。
- [x] 3.3 共享 SSE 字节解码与工具累积，覆盖无 index 双工具、元数据、跨块 UTF-8、摘要分离、usage、截断/EOF/取消。
- [x] 3.4 用轻量 extra_content 状态替代 GoogleStep；移除 loop 的协议绑定，验证真实 store 重载、ID 碰撞、第三工具、clarify 恢复及 IPC 隔离。
- [x] 3.5 保持单列幂等迁移与其他 provider 行为，验证压缩的完整活动链及签名不进入摘要，完整工具目录 schema 原样发送。
- [x] 3.6 运行产品 Rust 客户端的显式环境 Key 合成 smoke，覆盖工具闭环/重载、摘要和图片；失败必须报告为失败。
- [ ] 3.7 完成前后端 gates、OpenSpec 校验并更新 verification，单列未删除原因和未测 GUI/平台边界写清。

## 7. 前端能力 UI 与历史记录

- [x] 7.1 更新 TypeScript catalog 类型及 fallback，与后端目录形成一致性测试；验证新下拉恰好六项、历史三项仍可查询、Mock 不出现在新建目录。
- [x] 7.2 改造 ModelConfigSection 为能力驱动开关/强度选项并统一只读标签；Testing Library 覆盖 Gemini low/medium/high、K3/GLM low/high/max、MiMo 仅开关和非空会话锁定。
- [x] 7.3 更新 sessionConfig 与模型切换规范化，保留同模型合法草稿、切换采用目标默认；单测覆盖 low/medium 重载、旧草稿回退 DeepSeek、已有历史 session 不被替换。
- [x] 7.4 扩展全局密钥 Drawer、API_PROVIDERS 和启动 Key 状态到 google/zhipu；组件测试覆盖无项目保存、缺 Key 定位、仅新 provider Key 可发送，且余额/智能建议策略不变。
- [x] 7.5 更新 sendReadiness、历史标签和退役提示，阻止 unknown/retired 绕过校验；测试旧 Pro/K2.6 可继续、Ultraspeed 可读但不可发、原模型名称始终保留。
- [x] 7.6 统一附件按钮、粘贴提示及发送能力检查；UI/IPC 测试覆盖四个新建视觉模型、历史 K2.6、非视觉拒绝、仅图片发送和缺失历史附件降级。
- [x] 7.7 修复历史 DeepSeek Flash 别名在配置组件中的能力解析与选中判断；回归覆盖旧空会话开关/强度、重复点击不重置及锁定会话标签。

## 8. 跨模块验收与交付

- [x] 8.1 执行旧数据库升级与旧 provider 回归场景，覆盖原会话 model/thinking、消息、工具结果和附件；交付升级前后断言结果，确认无需批量回填或改名。
- [x] 8.2 增加显式环境变量开启的合成 API smoke，覆盖六模型文本、思考、流式工具闭环及四模型视觉；默认 CI 不联网，报告每项结果和账号/网络限制，真实项目 schema 外发仅在另行获授权时执行。
- [x] 8.3 运行 npm run typecheck、npm test、npm run build，以及 src-tauri 下 cargo fmt --check、cargo clippy -- -D warnings、cargo test；全部通过或明确记录未通过项并修复后复验。
- [ ] 8.4 按 design D4 验收模型选择、密钥 Drawer、始终思考和历史退役 UI，保存不含凭据的截图；验证 Gemini/GLM 主聊天、历史读取与工具恢复完整流程。
- [ ] 8.5 更新用户说明及实现后的验证记录，区分协议探测与产品验收、注明单列迁移和旧二进制降级限制；运行 openspec validate add-native-gemini-and-glm-providers --strict 与 git diff --check 并确认通过。

## 9. Gemini 代理与 VPN（补充，客户端任务 4.1 的前置要求）

- [x] 9.1 显式启用 reqwest system-proxy 并审计共享 feature 影响；用 cargo tree -e features、macOS/Windows 构建及既有 provider 请求测试验证依赖、endpoint 和环境代理行为，记录系统代理新增影响。
- [x] 9.2 实现 Gemini 统一自动代理客户端工厂，每次模型 HTTP 调用重新读取设置，覆盖主聊天与全部辅助路径；工厂测试验证无硬编码端口、代理变更生效、TLS 校验开启、无直连重试。
- [ ] 9.3 增加本地 CONNECT 代理与合成 TLS/SSE 集成测试及环境变量隔离子进程测试；验证目标地址、流式工具续答、NO_PROXY、不可用代理不直连、超时/断流错误和凭据脱敏。
- [x] 9.4 在 Google 密钥区域加入设计 D12 的系统代理/TUN 说明并改进网络错误提示；组件测试验证用户文案，无额外数据库字段或必填代理配置。
- [ ] 9.5 对 macOS/Windows 打包应用执行 D12 验收矩阵，包含无环境变量的 GUI 启动、系统代理+全局、TUN、运行中启用/改端口、辅助调用及旧 provider 回归；交付脱敏代理连接证据与结果，未测平台明确列为未验收，不以 API 成功或终端测试代替路径证明。
