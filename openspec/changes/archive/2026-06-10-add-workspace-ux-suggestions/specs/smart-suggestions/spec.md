# 智能推荐问能力

推荐问生成统一使用 **DeepSeek Flash（非思考模式、无工具）**，与会话所选模型无关。生成失败、超时或输出不可解析时 MUST 静默降级为「无推荐」，不得阻塞或报错打断用户。

## ADDED Requirements

### Requirement: DeepSeek Key 门控
系统 SHALL 仅在已配置 DeepSeek API Key 时启用推荐问功能；未配置（含仅配置了其他厂商 key）时，首次会话不进入初始化状态、不发起任何推荐生成调用。用户首次保存 DeepSeek key 后，若当前会话为空且尚无推荐，系统 SHALL 立即触发该会话的初始化流程。

#### Scenario: 无 key 时功能整体关闭
- **WHEN** 用户未配置 DeepSeek key（或仅配置了 Kimi key）并打开空会话
- **THEN** 不出现初始化状态，输入框直接可用，无推荐问展示，无 LLM 调用发生

#### Scenario: 首次配置 key 后补触发
- **WHEN** 用户在空会话界面完成 DeepSeek key 保存
- **THEN** 系统随即对当前空会话触发首次推荐问生成（进入初始化状态）

### Requirement: 首次会话推荐问生成
系统 SHALL 在空会话（无任何消息）打开时生成 3–4 条推荐问：后端扫描项目内文档文件（docx/xlsx/pptx/pdf/md/csv），按修改时间读取最近至多 3 个文档的文本摘要（每个截断），连同文件清单交给 DeepSeek Flash，要求生成围绕文档分析/生成、提及具体文件名、可直接执行的问题（每条不超过 80 个字符），以 JSON 数组返回。已有消息的会话 MUST NOT 触发首次推荐。

#### Scenario: 基于文档内容生成推荐
- **WHEN** 项目内存在课程相关 docx/xlsx 文件且用户打开新会话
- **THEN** 生成的推荐问引用了实际文件名（如「汇总 课程体系.xlsx 各 sheet 的关键数据」），数量为 3–4 条

#### Scenario: 空项目仍可推荐
- **WHEN** 项目目录内没有任何文档文件
- **THEN** 生成通用的文档创建类推荐问（如新建 Word/PPT），不报错

### Requirement: 后续推荐问生成
系统 SHALL 在每轮对话完成（turn_complete）后，基于当前会话最近消息与工具调用足迹异步生成 2–3 条 follow-up 推荐问（每条不超过 80 个字符）；生成过程 MUST NOT 阻塞输入框或下一轮对话。

#### Scenario: 对话结束后出现 follow-up
- **WHEN** 一轮含文档生成的对话完成
- **THEN** 输入框上方出现 2–3 条与产物相关的推荐问胶囊（如调整样式、导出 PDF）

### Requirement: 生成调用约束与降级
推荐生成调用 SHALL：固定 `deepseek-v4-flash` 且 `thinking.enabled = false`、不携带工具定义、设置超时（约 20s）、提示词要求仅输出 JSON 字符串数组；解析时容忍代码围栏包裹；解析失败或调用失败 MUST 返回空结果并由前端静默处理。

#### Scenario: 输出不可解析时静默降级
- **WHEN** 模型返回的内容无法解析为字符串数组
- **THEN** 后端返回空数组，前端不展示推荐区，无错误弹出

#### Scenario: 超时降级
- **WHEN** 推荐生成调用超过超时阈值
- **THEN** 调用被取消并返回空结果，初始化状态解除、输入框解锁
