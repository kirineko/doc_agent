## 1. 模型目录与 Provider

- [x] 1.1 在 Rust 新增 `ModelCatalog`（5 模型 + vision/effort/context）与 `ModelId` 扩展（mimo-v2.5、mimo-v2.5-pro）
- [x] 1.2 实现 `mimo.rs` Provider（Bearer、`thinking.type` 无 keep/effort）并注册 `provider_for`
- [x] 1.3 新增 IPC `list_models`，前端改消费该接口（移除 `types.ts` 硬编码重复）
- [x] 1.4 secrets / `API_PROVIDERS` / `sendReadiness` 增加 `mimo` provider

## 2. 输出 token 参数策略

- [x] 2.1 在 `openai_compat` 实现 `apply_output_token_limit(provider, limit)`（DeepSeek→`max_tokens`，Kimi/MiMo→`max_completion_tokens`）
- [x] 2.2 主 Agent loop 默认 `max_tokens: None`（不传输出上限）
- [x] 2.3 压缩摘要、推荐问等窄场景改用映射函数；补充 Provider 字段映射单元测试

## 3. 多模态消息与持久化

- [x] 3.1 `messages` 表增加 `attachments_json`；store CRUD 与 `Message` 类型扩展
- [x] 3.2 `ChatMessage` / `messages_from_store` 支持 user 多模态 content 组装（运行时 base64）
- [x] 3.3 `send_message` IPC 接受 `attachments`；`loop_runner` 持久化 user 消息
- [x] 3.4 附件限制：≤4 张、≤50MB、仅图片 MIME

## 4. image_read 工具

- [x] 4.1 实现 `tools/image_read.rs`（vision 子调用、文本 tool result）
- [x] 4.2 `ToolRegistry::tools_for_model` 按 `supports_vision` 过滤
- [x] 4.3 handler 测试：vision 模型 Mock 子调用、非图片路径报错

## 5. 前端：Drawer 与 vision UI

- [x] 5.1 新增 `ModelSettingsDrawer`（Provider 分组、vision Eye 图标、思考/强度、三 Key）
- [x] 5.2 侧栏改为摘要 + 打开 Drawer；锁定态只读
- [x] 5.3 `ChatPanel`：粘贴图片、`save_upload` IPC、附件 chip、非 vision toast
- [x] 5.4 `MessageList`：user 消息展示附件缩略图

## 6. 测试与文档

- [x] 6.1 前端：`list_models` 契约、`sendReadiness` 含 mimo、toast 行为
- [x] 6.2 Rust：MiMo thinking body、多模态序列化、token 映射测试
- [x] 6.3 更新 `types.test.ts` / model-config 相关测试

## 7. 上下文 token 与压缩（对齐 kimi-cli）

- [x] 7.1 拆分 text-only `estimate_store_messages_tokens`（pending 路径不读附件、不展开 base64）
- [x] 7.2 `build_compact_input` 改为文本专用序列化，禁止经完整 `messages_from_store` 注入 base64
- [x] 7.3 保留 tail 的 `attachments_json` 在压缩后原样持久化；被压缩区附件不进入摘要 prompt
- [x] 7.4 `image_read` 子调用 usage 不写入 `sessions.last_token_count`
- [x] 7.5 loop 发送前校验：non-vision + 附件 → 明确错误
- [x] 7.6 单元测试：含附件 pending 估算、压缩剥离附件、子调用 token 隔离

## 8. 验收（手动）

- [x] 8.1 Kimi K2.6：粘贴图片发送，模型能描述图片
- [x] 8.2 MiMo v2.5：`image_read` 返回描述；v2.5-pro 无该工具
- [x] 8.3 DeepSeek：粘贴图片 toast，文本对话不受影响
- [x] 8.4 长会话含图：压缩后 tail 图片仍可被模型读取；摘要区旧图不重复发送
