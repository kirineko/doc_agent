## Context

ChatPanel 已实现 `@` 文件引用（`mention.ts` + `FileMentionPopup` + `fuzzyMatch`）与推荐问胶囊（`SuggestionCards`）。斜杠命令是同一输入框上的**第二类触发器**，面向任务模板而非文件路径。

约束：OpenSpec 流程、前端文件体量软上限、不新增 npm 依赖、模板 20–100 字。

## Goals / Non-Goals

**Goals:**

- `/` 触发命令菜单，fzf 模糊搜索 id / label / keywords / category
- 22 条静态命令，分类默认顺序 general → word → ppt → excel → pdf → web
- 选中后替换 `/query` 为 prompt，聚焦输入框，不发送
- 与 `@` 弹层互斥（同一时刻只显示一种）

**Non-Goals:**

- 动态 LLM 生成命令、命令持久化/收藏
- 后端 registry 或 i18n
- 替换或删除现有推荐问

## Decisions

### 1. 数据：前端静态 TS registry

`src/lib/slashCommands.ts` 导出 `SLASH_COMMANDS: SlashCommand[]` 与 `CATEGORY_ORDER`。

```typescript
export interface SlashCommand {
  id: string;           // general 类无前缀，如 "read"；其余如 "word:create"（冒号分隔，不用 /）
  category: SlashCategory;
  label: string;
  description: string; // 弹层副标题，一行
  keywords: string[];  // 模糊搜索用，含中文别名
  prompt: string;      // 20–100 字
}

export type SlashCategory =
  | "general" | "word" | "ppt" | "excel" | "pdf" | "web";

export const CATEGORY_ORDER: SlashCategory[] = [
  "general", "word", "ppt", "excel", "pdf", "web",
];

export const CATEGORY_LABELS: Record<SlashCategory, string> = {
  general: "通用",
  word: "Word",
  ppt: "PPT",
  excel: "Excel",
  pdf: "PDF",
  web: "Web",
};
```

**理由**：零延迟、可版本化、与 agent 工具链解耦；模板变更走 PR 即可。

### 2. 触发检测：镜像 `@` mention

`src/lib/slash.ts`：

- 触发：`/` 位于**行首或空白字符后**，`/` 与光标之间无空白 → `query` 为 `/` 后文本
- `applySlash`：删除 `[start, end)` 的 `/query`，插入 `command.prompt`
- `deleteSlashBeforeCursor`：Backspace 可选（v1 可省略，与 mention 不对称 acceptable）

**与 `@` 互斥**：`ChatPanel` 中 `mention` 优先于 `slash`（同 `@` 实现）。

### 3. 搜索与排序

- 空 query：按 `CATEGORY_ORDER` 分组展示全部命令（组内按 registry 数组顺序）
- 非空 query：对每条命令拼接 `id label description keywords category` 做 `fuzzyMatch`；结果仍按 category 分组，组顺序不变；组内按 score 降序
- 每类最多展示 8 条总上限可沿用 `@` 的 slice(0, 8) **或** 提高到 12（设计倾向 **不截断分类**，弹层 `max-h` 滚动展示全部匹配项）

### 4. UI：`SlashCommandPopup`

- 位置：textarea 上方，`mention-popup` 样式复用
- 行：主标题 `label` + 副标题 `description`；命中高亮
- 分组标题：`CATEGORY_LABELS[category]`
- 键盘：`↑↓` 跨组线性索引、`Enter`/`Tab` 确认、`Esc` 关闭（不修改 input）
- `onMouseDown` + `preventDefault` 防止失焦（同 FileMentionPopup）

### 5. Placeholder

在现有 placeholder 末尾追加提示：`/` 选择任务模板（与 `@` 引用并列，澄清/busy 态不展示 slash 提示亦可）

## 命令清单（22 条）

默认 registry 顺序即组内顺序。

### general（5）— id **无** `general/` 前缀，仅 `category: "general"`

| id | label | prompt |
|----|-------|--------|
| `read` | 阅读分析 | 请阅读 @文件名，概括内容结构；若是表格则总结关键数据，并给改进建议。 |
| `clarify` | 先澄清需求 | 我想做一份文档但还没想清楚，请先帮我澄清需求和格式。 |
| `search` | 搜索项目 | 请在项目里搜索「___」，告诉我出现在哪些文件。 |
| `convert` | 旧格式转换 | 请把 @旧格式文件 转成新 Office 格式，并说明是否有格式损失。 |
| `web-search` | 联网查资料 | 请联网搜索「___」的最新信息，整理成文档要点。 |

### word（5）

| id | label | prompt |
|----|-------|--------|
| `word:create` | 新建 Word | 帮我新建一份 Word 文档，主题是___。请先简单澄清再开始制作。 |
| `word:edit` | 精准修改 Word | 请精准修改 @文件名.docx：___。先解包改 XML 再回包，保留原有排版。 |
| `word:comment` | 添加批注 | 请给 @文件名.docx 添加批注：___。需要的话锚定到对应段落。 |
| `word:clean-revisions` | 接受修订 | 请接受 @文件名.docx 的全部修订，另存一份干净版本。 |
| `word:extract-table` | 提取表格 | 请从 @文件名.docx 提取表格为 CSV，并简要说明数据内容。 |

### ppt（2）

| id | label | prompt |
|----|-------|--------|
| `ppt:create` | 新建 PPT | 帮我新建一份 PPT，主题是___，大约___页。请先确认风格再制作。 |
| `ppt:edit` | 编辑 PPT | 请修改 @文件名.pptx：___（如改文案、换图、增删页）。 |

### excel（4）

| id | label | prompt |
|----|-------|--------|
| `excel:create` | 新建 Excel | 帮我新建一份 Excel，用途是___，需要哪些字段和 sheet？ |
| `excel:clean` | 清洗表格 | @文件名.xlsx 表头比较乱，请规范化并说明改了什么。 |
| `excel:check-formula` | 检查公式 | 请检查 @文件名.xlsx 的公式是否有错误，列出问题位置。 |
| `excel:analyze` | 数据分析 | 请对 @文件名.xlsx 做数据分析：___（如汇总、对比、排名）。 |

### pdf（4）

| id | label | prompt |
|----|-------|--------|
| `pdf:create` | Typst 新建 PDF | 请基于 Typst 新建 PDF，主题是___。先选模板写 .typ 再编译。 |
| `pdf:edit-typst` | 修订 Typst PDF | 请修改 @文件名.typ：___，改完后重新编译为 PDF。 |
| `pdf:ops` | PDF 页面操作 | 请处理 @文件名.pdf：___（合并/拆分/旋转/删除指定页）。 |
| `pdf:forms` | PDF 表单 | 请处理 @文件名.pdf 的表单需求：___。 |

> **Typst 工作流**（Agent 侧，与 system prompt 一致）  
> - **新建**（`pdf:create`）：`typst_read_template syntax/typst-guide` → `typst_list_templates` → 选场景模板 → `fs_write` `.typ` → `typst_to_pdf`  
> - **修订**（`pdf:edit-typst`）：`fs_read`/`fs_patch` 改已有 `.typ` → `typst_to_pdf`（可跳过 list/场景模板）；编译失败时按 diagnostics 局部 patch，勿整篇重写

### web（2）

| id | label | prompt |
|----|-------|--------|
| `web:report` | HTML 报告 | 帮我生成一份 HTML 网页报告，主题是___，表格和文字要清晰。 |
| `web:save-pdf` | 导出 PDF | 请把 @路径（HTML 文件或报告目录）导出为 PDF。 |

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 模板过短，Agent 仍走错工具链 | 模板中保留关键路径词（解包/XML、CSV、澄清）；system prompt 仍强制 skill_read |
| `word:comment` 等进阶能力用户期望过高 | description 注明「需 OOXML 流程」；模板含锚定说明 |
| `@` 与 `/` 同时输入混淆 | 互斥弹层 + placeholder 说明 |
| registry 与真实工具能力漂移 | 命令清单在 design/proposal 文档化；大改工具时需同步 PR |

## Migration Plan

纯前端增量发布，无数据迁移。回滚：移除 ChatPanel 集成与 registry 文件即可。

## Open Questions

（无 — 命令清单与排序已在探索阶段与用户确认。）
