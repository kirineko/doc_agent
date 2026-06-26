## Why

后端已具备 `markdown_to_html` / `markdown_list_templates` 等 Markdown 网页导出能力，以及 `image_download` 图片下载工具，但斜杠命令 UI 尚无对应入口。用户难以区分「Markdown 模板化网页」与「自由 HTML 报告（web:report）」两条路径，也无法通过 `/` 快速触发按主题下载图片的常见前置步骤。

## What Changes

- 新增斜杠分类 **`markdown`**，含 4 条 template 命令：`markdown:slide`、`markdown:report`、`markdown:resume`、`markdown:convert`
- 在 **`general`** 分类新增 `download-images` 命令：用户填**主题**（非 URL），引导 Agent 搜索并 `image_download` 落地
- 更新 **`web:report`** 的 `description`，明确为「自由 HTML/CSS，非 Markdown 模板」
- 更新 registry 分类顺序：`command → general → markdown → word → ppt → excel → pdf → web`
- template 总数由 23 增至 **28**；`SlashCategory` 类型与相关测试同步更新

## Capabilities

### New Capabilities

（无 — 本变更仅扩展现有 slash-commands 注册表，不引入新后端能力。）

### Modified Capabilities

- `slash-commands`：新增 `markdown` 分类与 5 条 template；更新分类枚举、默认顺序、模板数量、Agent 能力映射表；修订 `web:report` description

## Impact

- 前端：`src/lib/slashCommands.ts`、`src/lib/slash.test.ts`（及依赖 `CATEGORY_ORDER` / 模板数量的测试）
- OpenSpec：`openspec/specs/slash-commands/spec.md`（归档时合并 delta）
- 无 Rust / IPC / 新 npm 依赖变更

## 纳入 / 排除

**纳入**

- 4 条 markdown template + 1 条 general `download-images`
- `web:report` description 微调
- registry 与 spec 数量/顺序/映射表更新

**排除**

- 新增 `markdown:export-pdf`（复用现有 `web:save-pdf`）
- 移动或删除 `web:report`
- 后端工具或 skill 变更
- i18n、动态命令、使用统计
