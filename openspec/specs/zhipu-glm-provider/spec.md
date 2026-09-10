# zhipu-glm-provider Specification

## Purpose

定义智谱 GLM-5.3-Flash 在文档助手中的模型调用、鉴权、思考和流式工具交互行为，确保使用标准 Chat Completions API 并与既有本地工具和视觉输入流程一致。

## Requirements

### Requirement: GLM 标准 Chat Completions 接入

系统 SHALL 使用 provider `zhipu`、模型 `glm-5.3-flash`，通过 `https://open.bigmodel.cn/api/paas/v4/chat/completions` 和 Bearer 鉴权调用。系统 MUST NOT 将路径拼接为含额外 `/v1` 的地址，不使用 Coding Plan 或 Responses endpoint，不自动切换协议。

#### Scenario: 配置智谱 Key 后聊天
- **WHEN** 用户保存 zhipu Key 并选择 GLM-5.3-Flash 发送文本
- **THEN** 请求到标准 endpoint，model 正确，回复通过现有聊天区展示

#### Scenario: 额度或权限错误
- **WHEN** 智谱返回 HTTP 错误及额度/权限信息
- **THEN** 向用户呈现可理解错误，不记录密钥、不改用其他地址或模型

### Requirement: GLM 思考参数和历史回传

GLM 请求 SHALL 使用 `thinking.type:enabled`、`thinking.clear_thinking:false` 和合法 `reasoning_effort`。已有 assistant 的 reasoning_content 和工具调用信息 SHALL 在多轮请求中保留；空推理文本 MUST 被接受。

#### Scenario: 低强度工具调用没有推理文本
- **WHEN** GLM 在 low 档返回工具调用但无 reasoning_content
- **THEN** 系统正常处理该调用，并保留可用的其余消息内容

#### Scenario: 非空推理内容续答
- **WHEN** GLM 返回推理文本及工具调用后收到本地结果
- **THEN** 后续请求携带原推理文本、调用信息和匹配的工具结果

### Requirement: GLM 流式与视觉能力

系统 SHALL 支持 GLM 的流式文本、函数调用和 usage，工具流式请求携带 `tool_stream:true`。图片 SHALL 使用 Chat Completions image_url 内容块。函数参数完整后才交给本地执行。

#### Scenario: 流式工具调用闭环
- **WHEN** GLM 流式生成一个函数调用并在本地获得结果
- **THEN** 累积参数与调用 ID 正确，续答可输出结果，末尾 usage 可采集

#### Scenario: 图片输入
- **WHEN** 用户发送合法 PNG 附件
- **THEN** 请求包含 image_url Data URL，模型回答显示在普通聊天区

#### Scenario: 显式输出预算
- **WHEN** 辅助请求明确指定输出预算
- **THEN** GLM 请求使用 max_tokens 而非 max_completion_tokens
