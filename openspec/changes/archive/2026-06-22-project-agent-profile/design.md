## Context

- **会话隔离**：`project-session` 要求构造请求仅用当前会话历史。
- **Clarify**：`clarify_ask` 一次一问、`resume_turn` 续跑；答案仅 tool result，不进 user 消息。
- **斜杠**：22 条均为 `kind: template`，选中后填入 prompt，不触发特殊逻辑。
- **注入点**：`loop_support::build_working_messages` 在 skills 索引后拼 system。
- **Backlog**：BL-006；用户决策见下。

## Goals / Non-Goals

**Goals:**

- 项目根 `AGENTS.md` 作为项目记忆；手写生效，与 `/init` 解耦
- `/init` 为真 command → `send_message` → 占 turn → 当前 session 模型
- init 用 clarify（含新 `confirm_agents_md`），读项目 + 会话史
- user 消息显示 `/init` 原文；clarify pending 时拒绝 init
- 允许空项目/空会话 init

**Non-Goals:**

- 推荐问（`generate_suggestions`）驱动 init
- 独立 IPC init 状态机（不占 turn）
- init 自动继续上一轮文档任务

## Decisions

### D1：记忆载体与 init 解耦

| 能力 | 说明 |
|------|------|
| **AGENTS.md 注入** | 每 turn `Sandbox` 读 `<root>/AGENTS.md`；存在则追加到 system（见 D5） |
| **手写** | 用户用任意编辑器修改；下一 turn 自动生效，**无需** `/init` |
| **`/init`** | 可选：clarify 问卷 + 写文件；与注入独立实现 |

### D2：`/init` 为 session 内 turn

- 前端 command：`send_message`，content 为**用户输入原文**（含 `/init` 与可选尾部）
- 使用**当前 session 锁定模型**与 thinking 配置
- 占用 running turn；clarify 等待期间按现有 spec 不占并行名额
- `working_messages` 自然包含当前会话历史（init 前所有消息）

**Turn 内 system augment（可选）**：首步可由后端在 user 消息之外附加短 hint（实现细节），或仅靠 `skill_read profile` + 用户原文；不在 UI 改写 user 文本。

### D3：斜杠 command 分叉

```typescript
type SlashEntry =
  | { kind: "template"; id; category; prompt; ... }  // 现有
  | { kind: "command"; id: "init"; label; description; keywords; acceptsTail: true }
```

- **template**：选中填入 prompt，不发送
- **command init**：Enter → 校验（无 clarify pending、有 project/session 等）→ `send_message(原文)`

### D4：Clarify 流程与 `confirm_agents_md`

1. Agent `skill_read profile`
2. `fs_read` 已有 `AGENTS.md`（可选）
3. 工具读项目（`office_read` / `fs_read` / `fs_list`）+ 利用会话史
4. 多轮 `clarify_ask`（`single`/`multi`/`text`）
5. **`clarify_ask` `kind=confirm_agents_md`**：`preview_markdown` 为拟写入**全文**（卡片内可滚动，MVP）
6. 用户确认 → `fs_write` `AGENTS.md`（merge 策略见 D6）
7. Assistant **简短**摘要（如「已更新 PPT 节：…」），非贴全文

新 kind 字段：

```json
{
  "id": "confirm-agents-md",
  "kind": "confirm_agents_md",
  "prompt": "请确认以下项目配置将写入 AGENTS.md",
  "preview_markdown": "# 项目 Agent 配置\n...",
  "changelog_summary": "新增 PPT 风格节；更新命名约定"
}
```

前端 `ClarifyKind` 扩展；`ClarifyQuestionCard` 渲染 Markdown 预览区。

### D5：AGENTS.md 注入

- 路径：项目根 `AGENTS.md`（POSIX 相对路径字面量，沙箱校验）
- **每 turn 读盘**（MVP 不做 mtime 缓存）
- 追加在硬编码 system + `index_markdown()` 之后：

```text
## 项目配置（AGENTS.md）
{truncated_body}
```

- **注入上限**：`MAX_AGENTS_MD_INJECT_CHARS = 3000`（常量，可调）；超出按节优先级截断：PPT → Word → Excel → PDF/Typst → 概述 → 其余
- 文件不存在：跳过该段，行为与 today 一致

### D6：AGENTS.md 文件 schema 与 merge

**推荐结构**（profile skill 与 synthesis prompt 强制）：

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

- **文件硬上限**：`MAX_AGENTS_MD_FILE_CHARS = 8000`（写入时校验，超出要求 Agent 压缩）
- **init 更新**：读旧文件 → LLM merge（保留未涉及节）→ 整文件替换写入
- **手写**：无格式强制；注入时仍截断

### D7：写 `AGENTS.md` 的门禁

- **仅 init turn** 允许 Agent `fs_write` / `fs_patch` 目标为 `AGENTS.md`（`profile` skill 声明；后端 `Sandbox` 或 tool 层校验：非 init 标记的 turn 拒绝写该路径）
- **手写**不受限（用户 OS 级编辑）
- init turn 标记：`send_message` 检测 user 内容以 `/init` 开头（允许前导空白）设置 `turn_meta.profile_init = true`

### D8：门禁与空 init

| 条件 | 行为 |
|------|------|
| `clarify_pending` 存在 | 前后端拒绝 `/init` send，提示先完成澄清 |
| 无 project / 无 session | 与现有 send 门禁一致 |
| 空项目 + 空会话 | **允许**；clarify 问通用偏好，生成骨架 AGENTS.md |

### D9：profile skill

- 路径：`assets/skills/profile/SKILL.md`
- 注册于 `core/skills.rs`；`index_markdown()` 列出
- 内容：init 触发条件、与 clarify/doc 创作区分、问题库（按 Office 类型）、confirm_agents_md、写文件与摘要要求、**禁止非 init 写 AGENTS.md**

### D10：与 InitCapsule / clarify doc 创作

- **InitCapsule**：仍为 session starter 推荐问，与 `/init` 无关
- **clarify skill 文档创作**：`confirm_brief` 仍用于交付物简报；`confirm_agents_md` 仅用于 AGENTS.md

## Risks / Trade-offs

- **长会话 init token 成本高** → profile skill 要求优先读 AGENTS.md + 最近产物路径，避免重复读全文；可预扫描摘要（Phase 1.5）
- **模型差异** → profile skill 强调 Markdown 与 JSON clarify 参数；测试覆盖 DeepSeek + 一种 vision 模型
- **用户误用 `/init` 期待继续写 doc** → turn 结束摘要明确「项目配置已更新」；skill 写明不自动续作上一任务
- **手写格式混乱** → 注入原样截断；不解析失败

## Migration Plan

- 无 DB 迁移
- 既有项目无 AGENTS.md：零行为变化直至用户创建文件或 `/init`
- 归档后更新 `project-backlog` BL-006 为已完成

## Open Questions

（无阻塞项；MVP 决策已闭合。）
