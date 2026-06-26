## Context

`src/lib/slashCommands.ts` 已有 7 个分类（command / general / word / ppt / excel / pdf / web）、23 条 template。后端 `markdown_to_html` 三件套与 `image_download` 已上线，但 registry 无对应入口。`web:report` 走 html-report + `fs_write`，与 markdown skill 的模板化路径易混淆。

约束：OpenSpec 流程、prompt 20–100 字、无新 npm 依赖、纯前端变更。

## Goals / Non-Goals

**Goals:**

- 新增 `markdown` 分类 Tab 与 4 条 template（slide / report / resume / convert）
- general 新增 `download-images`，prompt 仅含 `{{主题}}` 占位符，引导 Agent 搜索 + `image_download`
- 修订 `web:report` description，与 markdown 报告分工清晰
- 更新 `CATEGORY_ORDER`、测试断言与 spec delta

**Non-Goals:**

- `markdown:export-pdf`（沿用 `web:save-pdf`）
- 移动/删除 `web:report`
- 后端工具、skill、IPC 变更
- SlashMenuFlyout / SlashCommandPopup 组件逻辑改动（自动消费 registry）

## Decisions

### 1. 新增独立 `markdown` 分类，而非并入 `web`

**理由**：markdown skill 工作流（list_templates → read_template → fs_write → markdown_to_html）与 html-report（fs_write 自由 HTML）工具链不同；独立 Tab 降低误选率，与 word/ppt 等格式分组一致。

**顺序**：`command → general → markdown → word → ppt → excel → pdf → web`（markdown 作为轻量入口置于 general 之后）。

### 2. 四条 markdown 命令对应 profile + convert

| id | 引导重点 |
|----|---------|
| `markdown:slide` | profile=slide，选模板 → 写 .md → 转 HTML |
| `markdown:report` | profile=report，选模板 → 写 .md → 转 HTML |
| `markdown:resume` | profile=resume，frontmatter |
| `markdown:convert` | 已有 `.md`，指定 profile 转换 |

prompt 不重复 `skill_read`（system prompt 已强制）；用户可见措辞统一用「转 HTML」，不用工具名 `markdown_to_html`。

### 3. `download-images` 放 general，占位符为主题非 URL

**理由**：用户通常说「下载与 XX 相关的图片」，Agent 用 `web_search` 找 URL 再 `image_download`；比让用户粘贴 URL 更符合 Chat 交互。

**prompt**：`请找并下载与「{{主题}}」相关的图片到 images/，告诉我本地路径。`

### 4. `web:report` description 微调

由「生成项目内静态网页报告」改为「自由 HTML/CSS 静态报告（非 Markdown 模板）」——仅改 description，prompt 不动。

## Registry 清单（实现参考）

```typescript
// markdown 分类（插入 SLASH_TEMPLATE_SEEDS，位于 web 条目之前）
{ id: "markdown:slide",   category: "markdown", label: "Markdown 幻灯片", ... }
{ id: "markdown:report",  category: "markdown", label: "Markdown 报告", ... }
{ id: "markdown:resume",  category: "markdown", label: "Markdown 简历", ... }
{ id: "markdown:convert", category: "markdown", label: "转 HTML", ... }

// general 分类（追加）
{ id: "download-images", category: "general", label: "下载图片", ... }
```

`SlashCategory` 与 `CATEGORY_LABELS.markdown = "Markdown"` 同步更新。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| Tab 增至 8 个，横向略挤 | 已有 overflow-x-auto；可接受 |
| `download-images` 无 Tavily Key 时搜索失败 | description 可注明需联网；与 `web-search` 同类依赖 |
| markdown / web 报告仍可能混淆 | description 双写边界；用户可通过 `/markdown` 过滤 |

## Open Questions

（无 — 方案已在 explore 阶段确认。）
