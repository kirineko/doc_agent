## Context

当前 `ChatMessage.content` 为 `Option<String>`，`send_message` 仅传文本；`ModelId` 枚举 3 模型、2 Provider（DeepSeek/Kimi）。`openai_compat` 统一 `bearer_auth` 与可选 `max_tokens`。

已通过 API spike 验证（2026-06）：

| 项 | DeepSeek | Kimi (.cn) | MiMo |
|----|----------|------------|------|
| Bearer | ✅ | ✅ | ✅ |
| `stream_options.include_usage` | ✅（已有） | ✅ | ✅ |
| `max_tokens` 生效 | ✅ | ✅（文档称弃用） | ✅ |
| `max_completion_tokens` 生效 | ❌ 忽略 | ✅ | ✅ |
| user 多模态 `image_url` | N/A | ✅ | ✅（仅 mimo-v2.5） |
| tool 多模态 `image_url` | N/A | ✅ 能读图 | ❌ HTTP 400 |
| thinking | effort + type | type + keep:all | 仅 type |

厂商普遍建议输出 token 上限使用默认值（约 32K），非必要不手动设置。

## Goals / Non-Goals

**Goals:**

- 5 模型统一目录（Rust 为单一数据源，IPC `list_models` 供前端消费）
- vision 按模型门控：粘贴、工具注册、`image_read` 子调用
- 用户粘贴图片 → 多模态 user 消息；历史可展示附件
- `image_read` 统一 vision 子调用（文本 tool result）
- Drawer 承载模型 + 思考 + 三 Provider API Key
- 主 Agent 请求省略输出 token 上限；子场景按 Provider 正确映射字段名

**Non-Goals:**

- PDF 转图片、大学数学 skill、视频/音频多模态
- `mimo-v2-omni` 与其他 MiMo 变体
- 用户可配置 max_tokens UI
- Kimi tool 消息注入图片的优化路径（spike 可行但为统一性采用子调用）
- 图片 token 精确本地计数（usage 回报 + **文本**启发式即可；与 kimi-cli 一致，接受 pending 对图片偏低估）

## Reference: kimi-cli 多模态上下文策略

调研 `reference/kimi-cli`（`compaction.py`、`context.py`、`message.py`、`read_media.py`）结论：

| 主题 | kimi-cli | doc-agent 采纳 |
|------|----------|----------------|
| 权威 token | API `usage.total` | 同左 |
| pending 估算 | `estimate_text_tokens` 仅 `TextPart`，忽略 `ImageURLPart` | 同左：`estimate_*` 路径不得展开 `attachments_json` 为 base64 |
| 粘贴存储 | 占位符 + 磁盘 cache；context JSONL 内联 base64 | SQLite 存路径；发送时编码（体积更优） |
| ReadMedia / image_read | tool 消息内联图片 → 计入**下一次**主 loop usage | `image_read` 子调用独立请求 → **不计入**会话 `token_count`；tool 结果仅文本 |
| 压缩输入 | `prepare()` 仅保留 `TextPart` 送入摘要 LLM | `build_compact_input` 须改为文本专用视图，禁止 base64 |
| 压缩语义 | 被摘要区图片剥离、信息丢失；最近 2 条保留含图 | 同左；tail 保留 `attachments_json` |
| vision 门控 | `image_in`：UI 粘贴 + `check_message` + 工具注册 | `supports_vision`：UI toast + loop 发送前校验 + 工具过滤 |

## Decisions

### D1. 模型目录（`ModelCatalog`）

Rust 静态表 + `list_models` IPC，前端删除硬编码 `MODEL_OPTIONS` 重复。

| id | provider | api_model | vision | effort | max_context |
|----|----------|-----------|--------|--------|-------------|
| deepseek-v4-flash | deepseek | deepseek-v4-flash | false | yes | 1_000_000 |
| deepseek-v4-pro | deepseek | deepseek-v4-pro | false | yes | 1_000_000 |
| kimi-k2.6 | kimi | kimi-k2.6 | true | no | 256_000 |
| mimo-v2.5 | mimo | mimo-v2.5 | true | no | 1_000_000 |
| mimo-v2.5-pro | mimo | mimo-v2.5-pro | false | no | 1_000_000 |
| mimo-v2.5-pro-ultraspeed | mimo | mimo-v2.5-pro-ultraspeed | false | no | 1_000_000 |

Kimi base URL 保持 `https://api.moonshot.cn`。MiMo：`https://api.xiaomimimo.com`。

### D2. 多模态消息与持久化

```text
DB: messages.content (text) + messages.attachments_json (optional)
API 组装: user content → [{type:text},{type:image_url,...}]
```

- 附件存项目沙箱 `.uploads/<uuid>.<ext>`，DB 只存相对路径与 MIME
- 禁止在 SQLite 存 base64
- `messages_from_store` 发送前读取文件编码为 `data:{mime};base64,...`
- 单条消息附件上限默认 4 张、单张 ≤ 50MB（对齐 MiMo 文档）

### D3. `image_read`：vision 子调用

```text
image_read(path, prompt?)
  → 非 vision 模型：工具不注册
  → vision：独立 chat/completions（无 tools，单轮）
       user: [image_url, text]
  → tool result：纯文本 JSON（描述 + 可选 metadata）
```

理由：MiMo tool 角色不支持 `image_url`；子调用对所有 vision 模型行为一致。

子调用**不传**输出 token 上限（用厂商默认），除非实测需要再加保守上限。

### D4. 输出 token 参数策略

| 场景 | 是否传上限 | 字段（若传） |
|------|-----------|-------------|
| 主 Agent loop | **否** | — |
| 上下文压缩摘要 | 是（控制成本） | 按 Provider 映射 |
| 推荐问生成 | 是（短输出） | 按 Provider 映射 |
| `image_read` 子调用 | **否**（默认） | — |

映射函数（仅显式需要时）：

```rust
fn apply_output_token_limit(body: &mut Value, provider: ProviderKind, limit: u32) {
    match provider {
        DeepSeek => body["max_tokens"] = limit,
        Kimi | MiMo => body["max_completion_tokens"] = limit,
    }
}
```

**禁止**对 DeepSeek 仅发 `max_completion_tokens`（spike 证实被忽略）。**禁止**对 Kimi/MiMo 仅发已弃用的 `max_tokens`（新代码路径）。

现有 `ChatRequest.max_tokens: Option<u32>` 保留为内部「输出上限」语义；`None` 表示省略（主循环默认）。

### D5. MiMo thinking

复用 Kimi 分支形状，但**不加** `keep: all`：

```json
{ "thinking": { "type": "enabled" | "disabled" } }
```

历史 `reasoning_content` 继续完整回填（与 DeepSeek/Kimi 一致）。

### D6. 工具注册

`ToolRegistry::tools_for_model(model)` 过滤：无 vision 时不含 `image_read`。

### D7. UI：Drawer

侧栏保留项目/会话/摘要按钮；「模型与密钥」打开右侧 Drawer：

- 按 Provider 分组模型单选
- vision 模型旁 `Eye` 图标
- 思考开关 + DeepSeek 强度
- 三 Provider API Key（沿用 `ApiKeySection` 行组件）

非 vision 粘贴：toast「当前模型不支持图片输入，请选用 Kimi K2.6 或 MiMo v2.5」。

### D8. 上下文 token 与压缩（对齐 kimi-cli）

**Pending 与权威计数**

- `token_count`：每次主 Agent loop API 流式响应的 `usage.total_tokens`（已有）
- `pending_estimate`：自上次 usage 后追加消息的**文本专用**估算（`content` + `reasoning_content` + tool call 名/参数字符串），**MUST NOT** 读取附件文件或展开 base64
- 压缩触发：`token_count + pending_estimate`（已有 `should_auto_compact`）
- 接受 trade-off：大段图片 token 在下次 API 回报前 pending 偏低，压缩可能略晚触发（kimi-cli 同样行为）

**压缩输入**

- `build_compact_input` / `prepare_compaction` MUST 使用文本专用序列化（store 消息的 `content` 与 tool 文本），**不得**调用完整 `messages_from_store` 展开附件
- 被压缩区的 `attachments_json` 不进入摘要 prompt；摘要中可保留「用户曾发送 N 张图片」类文本线索（若 `content` 提及），但**不得**嵌入 base64
- 保留 tail（默认最近 2 条 user/assistant）的 `attachments_json` 原样持久化；重建 API 请求时再 base64 编码

**压缩后 token 基线**

- `after_tokens` = 摘要 LLM `usage.output_tokens` + 保留消息的文本 `estimate_*`（不含图片）；下次主 loop usage 校正

**`image_read` 与会话 token**

- 子调用为独立 `chat/completions`，其 `usage` **不写入** `sessions.last_token_count` / `pending_estimate`
- 主 loop 下一次请求的 `usage.input` 自然包含 tool 结果文本 token

**vision 发送前校验**

- loop / IPC 层：若消息含 `attachments_json` 且模型 `supports_vision=false`，MUST 返回明确错误（对齐 kimi `check_message` → `LLMNotSupported`）

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 图片增大上下文与费用 | 附件数/大小限制；压缩摘要不含 base64；被摘要区图片信息丢弃（与 kimi-cli 一致） |
| pending 低估含图消息、压缩触发偏晚 | 与 kimi-cli 一致接受；权威值靠 API usage；文档化 trade-off |
| `build_compact_input` 误展开 base64 撑爆摘要 | 文本专用序列化路径 + 单元测试 |
| `estimate_store_messages_tokens` 经 `messages_from_store` 误算 | 拆分 text-only estimate 与 API 完整组装 |
| `image_read` 额外 API 调用 | 仅 Agent 显式调用；子调用不传 tools 降延迟 |
| DeepSeek 误用 `max_completion_tokens` | 集中映射函数 + 单元测试 |
| Kimi 低 prompt_tokens 难判断读图 | 不依赖 token 数判读；`image_read` 以子调用文本为准 |
| 会话锁定后无法换 vision 模型 | 文档/ toast 提示新建会话 |

## Migration Plan

- 无 DB 破坏性迁移：`attachments_json` 可空列默认可空
- 新 Provider `mimo`：用户需在 Drawer 配置 MiMo API Key
- 现有会话与消息不受影响（纯文本路径不变）

## Open Questions

- （已关闭）MiMo 鉴权：Bearer ✅
- （已关闭）流式 usage：MiMo ✅
- 压缩摘要是否改为也不传 token 上限：当前保留显式上限以控成本，实现时可再评估
