# clarify-skill Specification

## Purpose
TBD - created by archiving change add-clarify-skill. Update Purpose after archive.
## Requirements
### Requirement: clarify skill 资产

系统 SHALL 内置 `clarify` skill（`assets/skills/clarify/SKILL.md`），经编译期 `include_str!` 内置，不依赖运行时外部路径。该 skill 通过现有 `skill_read` 工具按名称 `clarify` 获取。

#### Scenario: skill_read 可获取 clarify

- **WHEN** Agent 调用 `skill_read {"skill": "clarify"}`
- **THEN** 返回 clarify skill 的 SKILL.md 全文，且内容包含触发条件、问题库、深度控制规则与创作简报格式说明

#### Scenario: 未知 skill 错误信息包含 clarify

- **WHEN** Agent 调用 `skill_read {"skill": "unknown"}`
- **THEN** 错误信息中的可用 skill 列表包含 `clarify`

---

### Requirement: clarify skill 流程定义

clarify skill 的 SKILL.md SHALL 定义完整的需求澄清流程。澄清过程中 **MUST** 通过 `clarify_ask` 工具发起每一道结构化问题；**禁止**仅用 assistant 纯文本列出选项或问卷。创作简报汇总 **MUST** 使用 `clarify_ask` 且 `kind=confirm_brief`，并等待用户确认后再进入 `skill_read` + 生成。

流程步骤更新为：

1. 识别文档类型与场景
2. 评估已有信息，选择深度路径
3. **逐问澄清** — 每问调用一次 `clarify_ask`（一次一问）
4. **汇总确认** — `clarify_ask` + `confirm_brief`，等待用户确认
5. **转入生成** — `skill_read` 对应格式 skill

#### Scenario: 标准路径使用 clarify_ask

- **WHEN** 用户发送模糊 PPT 创作请求且 Agent 进入 clarify 流程
- **THEN** Agent 通过 `clarify_ask` 逐题澄清，而非在 assistant 消息中列出 A/B/C 文本选项

#### Scenario: 创作简报经 confirm_brief 确认

- **WHEN** 澄清轮次完成
- **THEN** Agent 调用 `clarify_ask`（`kind=confirm_brief`）展示创作简报，用户确认后才 `skill_read` 对应格式 skill

#### Scenario: 澄清完成输出创作简报

- **WHEN** 用户在前端确认创作简报
- **THEN** 对话历史含可读的已答澄清卡片，且 Agent 收到的 tool result 含结构化 brief 字段

### Requirement: clarify skill 问题库——Word 文档

clarify skill SHALL 包含 Word 文档专属问题库，覆盖内容、结构、排版/样式三个维度：

**内容维度**：
- 文档主题与核心目的
- 目标受众（内部/外部，职级，专业背景）
- 关键论点或必须覆盖的内容要点

**结构维度**：
- 预期章节与层级（是否需要目录、附录、摘要）
- 篇幅要求（页数或字数范围）
- 是否需要封面页、页眉页脚、页码

**排版/样式维度**：
- 整体风格（公文正式 / 商务报告 / 学术 / 现代简洁）
- 正文字体偏好（宋体 / 微软雅黑 / 等线 / 无要求）
- 行间距与段间距偏好（宽松阅读 / 紧凑商务）
- 标题层级标识方式（编号体系 1. 1.1 1.1.1 / 纯样式区分）
- 是否有企业模板或品牌色要求

#### Scenario: Word 排版问题被问及

- **WHEN** 用户请求创建 Word 文档且未提供风格信息
- **THEN** Agent 在澄清过程中至少包含一个关于排版/样式的问题（字体、风格或间距其中之一）

---

### Requirement: clarify skill 问题库——PPT 演示

clarify skill SHALL 包含 PPT 演示专属问题库，覆盖内容、结构、排版/样式三个维度：

**内容维度**：
- 演讲主题与核心结论
- 受众（内部团队 / 客户 / 管理层 / 公众）与演讲场景
- 必须包含的数据、案例或关键信息

**结构维度**：
- 幻灯片数量（或演讲时长）
- 叙事结构（问题-方案-行动 / 时间轴 / 对比分析 / 自定义）
- 是否需要封面、目录、致谢/联系页

**排版/样式维度**：
- 视觉风格偏向（简约留白 / 商务深色 / 图文并茂 / 数据驱动）
- 主色调偏好或品牌色/logo 要求
- 文字密度（每页结论句 / 可接受较多文字）
- 是否需要统一动画效果

#### Scenario: PPT 视觉风格问题被问及

- **WHEN** 用户请求创建 PPT 且未提供视觉风格信息
- **THEN** Agent 在澄清过程中包含关于视觉风格或配色的问题

---

### Requirement: clarify skill 问题库——报告/HTML Report

clarify skill SHALL 包含报告（含 HTML Report）专属问题库，覆盖内容、结构、排版/样式三个维度：

**内容维度**：
- 报告类型（数据分析报告 / 项目总结 / 研究报告 / 调研报告）
- 数据来源与核心指标
- 报告结论导向（描述现状 / 诊断问题 / 提出建议）

**结构维度**：
- 必须包含的章节（摘要 / 背景 / 分析 / 结论 / 建议）
- 图表需求（类型、数量、数据是否已有）
- 篇幅与交付格式（打印/屏幕阅读/在线浏览）

**排版/样式维度**：
- 配色方案（企业品牌色 / 中性灰蓝 / 自定义）
- 是否有 logo 或品牌规范要植入
- 图表样式（简约线条 / 填充色块 / 渐变）
- 是否需要打印友好布局

#### Scenario: 报告样式问题被问及

- **WHEN** 用户请求创建报告且未提供配色/样式信息
- **THEN** Agent 在澄清过程中包含关于配色方案或品牌规范的问题

---

### Requirement: clarify skill 交付格式覆盖 Markdown 网页

clarify skill 的 SKILL.md SHALL 在交付格式选项中包含「Markdown 网页（幻灯片 slide / 报告 report / 简历 resume）」，作为与 Word / PPT / Excel / HTML 报告 / Typst PDF 并列的可选交付格式。当用户选择该交付格式或意图为生成 Markdown 网页时，clarify 流程 SHALL 至少澄清 profile（slide / report / resume）与风格/模板倾向（除非用户已明确或 AGENTS.md 已规定）。

#### Scenario: 交付格式包含 Markdown 网页

- **WHEN** Agent 读取 clarify skill 全文并在交付格式不明时发起交付格式选择
- **THEN** 候选交付格式包含「Markdown 网页（slide / report / resume）」

#### Scenario: Markdown 网页澄清 profile 与风格

- **WHEN** 用户选择 Markdown 网页交付且未说明用途与风格
- **THEN** Agent 在澄清过程中至少询问 profile（slide / report / resume）与样式/模板倾向

