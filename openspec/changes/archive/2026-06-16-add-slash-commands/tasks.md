## 1. Registry 与检测逻辑

- [x] 1.1 新增 `src/lib/slashCommands.ts`：22 条命令、`CATEGORY_ORDER`、`CATEGORY_LABELS`（与 design.md 一致）
- [x] 1.2 新增 `src/lib/slash.ts`：`detectSlash`、`applySlash`（行首或空白后 `/`，query 无空白）
- [x] 1.3 新增 `src/lib/slashFuzzy.ts`（或扩展 `fuzzy.ts`）：对命令多字段 fuzzy 匹配 + 按 category 分组排序
- [x] 1.4 为 `slash.ts` 与搜索排序添加 Vitest 单测（触发边界、apply 替换、分类顺序、read 唯一性）

## 2. UI 组件

- [x] 2.1 新增 `SlashCommandPopup.tsx`：分组标题、双行 item、命中高亮、空状态
- [x] 2.2 复用/延伸 `mention-popup` 样式；必要时在 `index.css` 补充 slash 专用 class（保持文件精简）

## 3. ChatPanel 集成

- [x] 3.1 `ChatPanel.tsx`：slash 状态（index、cursor）、与 mention 互斥（mention 优先）
- [x] 3.2 键盘：`↑↓`/`Enter`/`Tab`/`Esc` 与 `@` 行为对齐；选中后 `pickSlash` 填 prompt 不发送
- [x] 3.3 澄清/busy/initializing 时不展示 slash 弹层
- [x] 3.4 更新 textarea placeholder：补充 `/` 选择任务模板提示

## 4. 验证

- [x] 4.1 `npm run typecheck && npm test && npm run build` 通过
- [x] 4.2 手动冒烟：空 query 分组顺序 general→web；`/word` 过滤；选 `/read` 填 prompt；与 `@` 互斥；澄清期无弹层
