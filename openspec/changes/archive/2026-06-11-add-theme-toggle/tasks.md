## 1. 主题基础设施

- [x] 1.1 在 `index.css` 定义 `data-theme` 下 dark/light 语义 CSS 变量（Notion 感浅色 palette）并通过 Tailwind v4 `@theme` 映射为 utility class
- [x] 1.2 将 `body`、`.tool-card`、`.panel`、`.markdown-body` 迁移为语义 token
- [x] 1.3 新增 `src/hooks/useTheme.ts`：`dark`/`light` 状态、toggle、`localStorage`（`doc-agent-theme`）读写与非法值回退
- [x] 1.4 在 `main.tsx` 首帧同步设置 `document.documentElement.dataset.theme`，减轻 FOUC

## 2. 顶栏 Toggle

- [x] 2.1 新增 `src/components/ThemeToggle.tsx`（switch toggle + `aria-label`）
- [x] 2.2 `App.tsx` 顶栏右侧挂载 `ThemeToggle`（`ml-auto`），品牌区布局不变

## 3. 组件主题迁移

- [x] 3.1 迁移 `App.tsx` 根容器与 `header` 至语义 class
- [x] 3.2 迁移 `Sidebar`、`ApiKeySection`、`ModelConfigSection`、`WebSearchSection`
- [x] 3.3 迁移 `ChatPanel`、`MessageBubble`、`MessageList`、`SuggestionCards`、`InitCapsule`
- [x] 3.4 迁移 `RightPanel`、`ToolChainPanel`、`ProjectFileExplorer`、`FileMentionPopup`

## 4. Markdown 与品牌资源

- [x] 4.1 `MarkdownView`：`prose`/`prose-invert` 与 highlight.js 主题（`github` / `github-dark`）随 `data-theme` 切换
- [x] 4.2 调整 `public/logo.svg`（或 CSS）使浅色背景下 Logo 清晰可辨

## 5. 测试与验证

- [x] 5.1 新增 `useTheme.test.ts`（toggle、持久化、非法值回退）
- [x] 5.2 更新受主题迁移影响的组件测试（若有脆弱 class 断言）
- [x] 5.3 `npm run typecheck && npm test && npm run build` 通过
