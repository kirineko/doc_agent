## Why

doc-agent 当前仅支持纯文本对话与工具结果，无法利用 Kimi K2.6、MiMo v2.5 等模型的视觉能力；模型目录亦缺少小米 MiMo，侧栏在 3 Provider × 5 模型下配置拥挤。为支撑后续 PDF/图片内容理解（用户粘贴、Agent 读图工具），需先建立多模态输入、按模型 vision 能力门控、以及统一的 Provider/模型配置体系。

现有 `context-compaction` 的 pending 估算与压缩输入仅覆盖纯文本；多模态引入后若通过 `messages_from_store` 展开 base64，会误算 pending 或把巨型 base64 送入摘要 LLM。参考 `reference/kimi-cli` 已验证的策略：**API `usage` 为权威计数、pending 仅估文本、压缩区剥离图片、保留 tail 原样附件**——须在实现前写入本变更。

## What Changes

- 扩展模型目录至 5 个模型、3 个 Provider（DeepSeek、Kimi、MiMo），每模型配置 `supports_vision`
- 新增 MiMo Provider（`https://api.xiaomimimo.com`，Bearer 鉴权，thinking 仅 enable/disable）
- 多模态 user 消息：支持图片附件（粘贴 → 沙箱 `.uploads/` → API `image_url`）；非 vision 模型粘贴时 toast 提示并跳过
- 新增 `image_read` 工具：仅 vision 模型注册；内部 vision 子调用返回文本（兼容 MiMo tool 消息不支持图片的限制）
- 模型与 API Key 配置迁入右侧 Drawer；vision 模型显示视觉标识
- Provider 输出 token 上限：主 Agent 循环**默认不传** `max_tokens` / `max_completion_tokens`（使用厂商默认，约 32K）；仅在压缩摘要、`image_read` 子调用等窄场景按需显式设置，且按 Provider 映射字段（DeepSeek → `max_tokens`；Kimi/MiMo → `max_completion_tokens`）
- 消息持久化：`attachments_json` 存路径与 MIME，历史展示缩略图，API 请求时运行时 base64 编码
- **上下文与图片 token**（对齐 kimi-cli）：pending 估算**不计**附件/base64；压缩摘要输入**仅文本**；被摘要区的图片信息不保留；保留 tail 的 `attachments_json` 原样；`image_read` 子调用 usage **不计入**会话 `token_count`
- 发送前校验：含附件的 user 消息在 non-vision 模型上 MUST 拒绝（不仅 UI toast）

## Capabilities

### New Capabilities

- `multimodal-input`：用户图片粘贴、附件持久化、多模态 user 消息组装与展示、非 vision toast
- `image-read-tool`：`image_read` 工具、vision 子调用、按模型条件注册

### Modified Capabilities

- `model-config`：5 模型、MiMo Provider、vision 按模型、上下文上限（MiMo 1M）、`list_models` IPC、Drawer 配置入口
- `workspace-ui`：侧栏精简、模型与密钥 Drawer、vision 标识、消息附件展示
- `agent-loop`：多模态消息序列化、条件工具注册、输出 token 参数 Provider 映射策略、vision 能力发送前校验
- `context-compaction`：文本专用 pending/压缩输入、附件在压缩区的剥离与 tail 保留、压缩后 token 估算策略

## Impact

- Rust：`agent/types.rs`、`provider/`（+`mimo.rs`）、`openai_compat.rs`、`loop_runner.rs`、`core/store.rs`、`ipc/mod.rs`、`tools/`（+`image_read.rs`）、`registry.rs`
- 前端：`types.ts`、`ModelConfigSection` → Drawer 组件、`ChatPanel` 粘贴与附件条、`useWorkspace` / `send_message` 契约
- Spec delta：`model-config`、`workspace-ui`、`agent-loop`、`context-compaction`；新增 `multimodal-input`、`image-read-tool`
- 测试：Provider token 字段映射、vision 门控、pending 不含 base64、压缩剥离附件、image_read 子调用不改 session token、粘贴 toast 组件测试
- 无新重型依赖；复用现有 `reqwest` + OpenAI 兼容层
