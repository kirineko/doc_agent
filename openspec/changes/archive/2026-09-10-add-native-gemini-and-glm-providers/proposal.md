## Why

当前应用仅接入 DeepSeek、Kimi、MiMo 的 Chat Completions，不能使用 Gemini 或 GLM；模型目录和统一的思考开关/high-max 控件也已不符合目标模型能力。2026-09-09 的官方资料核验及真实 API 探测已确认新增模型可用，同时发现 Gemini 签名、工具流差异 和旧模型下架需要明确兼容设计。

## What Changes

- 新增 `google` provider / `gemini-3.8-flash`，采用 Google OpenAI 兼容 Chat Completions API，支持流式正文、思考摘要、工具调用、图片、现有视觉子调用、标题和压缩。
- Gemini 自动跟随系统代理或 VPN，覆盖 Clash Verge 系统代理+全局模式和 TUN，支持桌面启动与运行中代理变更；增加打包应用代理路径验收及 Google 密钥区域说明。显式启用 reqwest 系统代理能力的共享影响需回归其他 provider，见 design D12。
- 新增 `zhipu` provider / `glm-5.3-flash`，采用智谱标准 Chat Completions endpoint，支持流式、工具调用、图片和 preserved thinking；不使用智谱 Responses 或 Coding Plan endpoint。
- 新建会话仅展示六个模型：DeepSeek Flash、MiMo v2.5、MiMo v2.5 Pro、Kimi K3、Gemini 3.8 Flash、GLM-5.3-Flash。DeepSeek Flash 的产品 id 与请求名均为 `deepseek-flash`，支持视觉；`deepseek-v4-flash` 仅作历史别名。
- **BREAKING（仅新建模型选择）**：移除 DeepSeek V4 Pro、MiMo v2.5 Pro Ultraspeed、Kimi K2.6 的新建入口。保留历史名称识别、能力、密钥归属和历史展示；仍可调用的旧模型沿原路由续聊，已不可调用的 Ultraspeed 明确阻止续聊，不静默换模型。
- 按模型能力显示思考开关和档位：DeepSeek low/high/max，MiMo 仅开关，Kimi/GLM 始终思考且 low/high/max，Gemini 始终思考且 low/medium/high。同步更新后端校验、草稿配置、会话标签、图片入口和全局密钥入口。
- SQLite 仅为 `messages` 新增一个 nullable TEXT 字段 `provider_state_json`，仅保存 Gemini 消息/工具 extra_content 签名元数据，不复制正文、步骤或 wire ID 映射。已有 provider 使用 NULL，现有 reasoning/tool 数据不改写。
- 增加无 index 工具关联、思考分流、签名回传、终态/用量解析与请求体大小限制，工具 schema 原样发送；保持本地工具执行、文件治理及人工澄清流程。

### 用户可见示例

- 选择 Gemini 3.8 Flash：显示 low/medium/high（默认 medium），不显示“关闭思考”；粘贴图片后通过 Google OpenAI 兼容 API 回答。
- 选择 Kimi K3：显示 low/high/max（默认 max），请求使用 `reasoning_effort`，不发送 K2.6 的 `thinking` 字段。
- 打开旧 Kimi K2.6 会话：仍显示原模型和原思考设置，不升级为 K3；旧 Ultraspeed 会话可阅读，但发送时提示改用新会话。

## Capabilities

### New Capabilities

- `google-gemini-provider`: OpenAI 兼容请求、共享 SSE、签名元数据回传、无 index 工具适配、异常处理与图片限额。
- `zhipu-glm-provider`: GLM Chat Completions 路由、鉴权、思考参数、图片和流式工具调用。

### Modified Capabilities

- `model-config`: 六模型新建目录、历史目录、能力元数据、密钥、思考 UI 与配置校验。
- `agent-loop`: 协议无关 provider 契约、轻量元数据持久化与恢复、终态和历史模型路由。
- `context-compaction`: Gemini 轻量元数据与工具组边界、模型适用的摘要请求和 token 统计。
- `multimodal-input`: 新视觉模型（含 DeepSeek Flash）与 provider 级发送大小校验。
- `image-read-tool`: DeepSeek Flash 动态注册 `image_read`。
- `workspace-ui`: DeepSeek Flash 粘贴图片走视觉入口。

## Impact

- Rust：`agent/provider/`、`model_catalog.rs`、`types.rs`、`loop_runner.rs`、`loop_support.rs`、`title_gen.rs`、`compaction.rs`、`core/store.rs`、IPC 配置校验、`tools/vision_subcall.rs`。
- 前端：`src/types.ts`、模型/草稿/发送校验 utilities、模型配置组件、`useWorkspace` 标签、密钥和附件入口，以及对应测试。
- 数据：一个可空消息列；无 sessions/tool_calls 新列，无旧会话模型批量 UPDATE，无密钥迁入数据库。
- 依赖：优先复用 reqwest、serde_json、现有 SSE/SQLite 基础设施；不引入 Google JS/Python SDK 或独立代理进程。
- 本轮不改 DeepSeek/MiMo/Kimi Chat Completions 协议形状；DeepSeek Flash 仅更新展示名、wire 模型名与视觉能力。不增加 Gemini Files API、音视频、原生 PDF、内置搜索/远程 MCP、服务端会话或余额查询。智能建议继续固定 DeepSeek Flash。
- 调研证据、限制、请求/响应示例、迁移和验证方案详见 `design.md`。所有示例使用占位符，不包含真实密钥。

用户确认 Interactions 尚未发版且无现有会话；本轮删除旧实现，不增加双协议或历史转换。签名仍需持久化，保留唯一可空列的理由见 design D7。
