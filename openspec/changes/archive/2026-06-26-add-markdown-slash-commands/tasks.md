## 1. Registry 类型与分类

- [x] 1.1 在 `slashCommands.ts` 的 `SlashCategory` 增加 `"markdown"`，更新 `CATEGORY_ORDER` 与 `CATEGORY_LABELS`
- [x] 1.2 确认 `SlashMenuFlyout` / `slashFuzzy` 无需额外改动（自动消费 registry）

## 2. 新增命令条目

- [x] 2.1 在 `SLASH_TEMPLATE_SEEDS` 追加 4 条 markdown 命令（slide / report / resume / convert），prompt 20–100 字
- [x] 2.2 在 general 追加 `download-images`，prompt 仅含 `{{主题}}` 占位符
- [x] 2.3 更新 `web:report` 的 `description` 为「自由 HTML/CSS 静态报告（非 Markdown 模板）」

## 3. 测试

- [x] 3.1 更新 `slash.test.ts`：template 数量 23→28、`CATEGORY_ORDER` 含 markdown、新命令 prompt 长度与 id 规则
- [x] 3.2 补充 fuzzy 搜索用例：`/markdown`、`/下载` 命中预期命令
- [x] 3.3 运行 `npm run typecheck && npm test` 通过

## 4. 验收

- [x] 4.1 本地打开斜杠菜单，确认 8 个分类 Tab 顺序与 4 条 markdown + download-images 展示正确
- [x] 4.2 选中各新命令，确认 prompt 填入内容与占位符符合 spec
