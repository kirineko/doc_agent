## MODIFIED Requirements

### Requirement: DeepSeek Key 门控
系统 SHALL 仅在已配置 DeepSeek API Key 时启用全部推荐问功能（含 starter 与 follow-up）；未配置（含仅配置了其他厂商 key）时，MUST NOT 展示初始化胶囊、MUST NOT 发起任何推荐问生成调用（starter 或 follow-up）。保存 DeepSeek Key 后 MUST NOT 自动触发 starter；用户 MUST 通过点击初始化胶囊显式触发 starter。

#### Scenario: 无 key 时推荐问功能整体关闭
- **WHEN** 用户未配置 DeepSeek key（或仅配置了 Kimi key）
- **THEN** 不出现初始化胶囊，不发起 starter 或 follow-up LLM 调用，输入框可直接使用

#### Scenario: 无 key 时对话结束无 follow-up
- **WHEN** 用户未配置 DeepSeek key 且完成一轮对话（turn_complete）
- **THEN** 不展示 follow-up 推荐问胶囊，不发起 follow-up 生成调用

#### Scenario: 配置 key 后不自动 starter
- **WHEN** 用户在空会话或草稿态界面完成 DeepSeek key 保存
- **THEN** 不自动进入 initializing 状态；若满足其他条件则展示初始化胶囊供用户点击

### Requirement: 首次会话推荐问生成
系统 SHALL 在用户**点击初始化胶囊**且当前上下文无 user/assistant 消息时生成 3–4 条 starter 推荐问：后端扫描项目内文档文件（docx/xlsx/pptx/pdf/md/csv），按修改时间读取最近至多 3 个文档的文本摘要（每个截断），连同文件清单交给 DeepSeek Flash，要求生成围绕文档分析/生成、提及具体文件名、可直接执行的问题（每条不超过 80 个字符），以 JSON 数组返回。打开空会话、新建会话或直接发送首条消息 MUST NOT 触发 starter。已有消息的会话 MUST NOT 触发 starter。

#### Scenario: 基于文档内容生成推荐
- **WHEN** 项目内存在课程相关 docx/xlsx 文件且用户点击初始化胶囊
- **THEN** 生成的推荐问引用了实际文件名（如「汇总 课程体系.xlsx 各 sheet 的关键数据」），数量为 3–4 条

#### Scenario: 空项目仍可推荐
- **WHEN** 项目目录内没有任何文档文件且用户点击初始化胶囊
- **THEN** 生成通用的文档创建类推荐问（如新建 Word/PPT），不报错

#### Scenario: 直接发送不触发 starter
- **WHEN** 用户在草稿态或空会话中不点胶囊而直接发送首条消息
- **THEN** 不生成 starter 推荐问
