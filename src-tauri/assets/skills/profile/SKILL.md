# 项目 Agent 配置（profile）

用于 `/init` 命令：通过多轮澄清生成或更新项目根 `AGENTS.md`。与文档创作 clarify 不同，本 skill **不**产出 docx/pptx/xlsx。

## 何时使用

- 用户发送 `/init` 或 `/init <补充说明>`（如「固化 PPT 风格」）
- **不要**在普通文档任务中调用；**不要**在非 init turn 写 `AGENTS.md`

## 流程（MUST 按序）

1. `skill_read profile`（本文件）
2. `fs_read` `AGENTS.md`（不存在时返回 `exists:false`，属正常，非错误）
3. 扫描项目：`fs_list`、抽样 `office_read` / `fs_read` 关键模板与近期产物
4. 结合**当前会话历史**理解用户意图（init 消息尾部说明、此前多轮讨论）
5. 多轮 `clarify_ask`（`single` / `multi` / `text`）澄清各 Office 类型偏好 — **每轮 assistant 仅一个 clarify_ask**，禁止同轮并行多个
6. 合成完整 `AGENTS.md` 草案，调用 `clarify_ask`：
   - `kind`: `confirm_agents_md`
   - `preview_markdown`: 拟写入**全文**（Markdown）
   - `changelog_summary`: 可选，一句话变更摘要
7. 用户确认后 `fs_write` 路径 `AGENTS.md`（merge 已有内容，保留未涉及章节）
8. 最终 assistant 消息：**简短**说明改了什么，**禁止**粘贴全文

## AGENTS.md 推荐结构

```markdown
# 项目 Agent 配置

## 概述
## Word
## Excel
## PPT
## PDF / Typst
## 命名与路径
## 禁止事项
## 参考文件
```

- 文件总长 ≤ 8000 字符；注入时系统仅取前 3000 字符
- 用户可手写 `AGENTS.md`，下一 turn 自动注入，无需 `/init`

## 澄清问题库（按项目情况选用）

**通用**
- 项目主要交付物类型（Word / PPT / Excel / PDF）
- 语言与语气（公文 / 口语 / 中英混排）
- 命名与目录约定

**PPT**
- 视觉风格（简约 / 商务深色 / 品牌色）
- 默认页数与结构（封面-目录-正文-总结）
- 图表与图片偏好

**Word**
- 公文/报告体例、标题层级
- 页眉页脚、字体字号习惯

**Excel**
- 表头规范、数字格式
- 是否保留公式或仅值

**PDF / Typst**
- 优先 Typst 还是 Word 转 PDF
- 公式与版式要求

## 禁止

- 非 init turn 使用 `fs_write` / `fs_patch` 写 `AGENTS.md`
- init 结束后自动继续上一文档任务
- 用 `confirm_brief` 代替 `confirm_agents_md`
- 在聊天中贴出完整 `AGENTS.md` 正文

## 与 InitCapsule 区别

侧栏推荐问（InitCapsule）仅填充首条消息；`/init` 是显式命令，占用独立 turn 更新项目配置。
