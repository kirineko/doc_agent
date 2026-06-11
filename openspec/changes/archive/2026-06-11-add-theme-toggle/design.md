# 设计：浅色 / 深色主题切换

## Context

- 当前 UI 在 `index.css`、`App.tsx` 及 ~13 个 React 组件中硬编码深色值（如 `#0b1020`、`slate-800`、`prose-invert`、`github-dark.css`）。
- 技术栈：React 19 + Tailwind CSS v4（`@import "tailwindcss"`），无现有主题 hook 或 `localStorage` 用法。
- 顶栏 `header` 仅左对齐 Logo、标题、项目名，无右侧控件区。
- 用户决策：**两档**（深 / 浅）、浅色 **Notion 感**、**默认深色**、右上角 **toggle**、偏好 **localStorage** 持久化。

## Goals / Non-Goals

**Goals：**

- `dark` / `light` 两档可即时切换，全工作区（三栏、消息气泡、工具链、侧栏配置、@ 弹层、空状态）视觉一致。
- 默认 `dark`，与现网用户习惯一致；重启后恢复上次选择。
- 顶栏右上角 toggle，accessible（`aria-label`、键盘可操作）。
- 语义 token 体系，后续新组件默认引用 token 而非硬编码色值。

**Non-Goals：**

- `system` / `prefers-color-scheme` 自动跟随。
- Tauri `config.toml` 或 SQLite 存主题。
- 用户自定义配色、过渡动画、独立设置页。
- 改窗口原生 chrome（仅 WebView 内容区）。

## Decisions

### D1：`<html data-theme="dark|light">` + CSS 变量

在 `document.documentElement` 设置 `data-theme`；`index.css` 定义语义变量：

| Token | dark（现有） | light（Notion 感） |
|-------|-------------|-------------------|
| `--bg-app` | `#0b1020` | `#fbfbfa` |
| `--bg-panel` | `#0f172a` | `#ffffff` |
| `--bg-elevated` | `#111827` | `#ffffff` |
| `--bg-muted` | `slate-950/40` 等效 | `#f7f6f3` |
| `--border` | `#1f2937` | `#e9e9e7` |
| `--border-subtle` | `#334155` | `#ededec` |
| `--text-primary` | `#f9fafb` | `#37352f` |
| `--text-secondary` | `#94a3b8` | `#787774` |
| `--text-muted` | `#64748b` | `#9b9a97` |
| `--accent` | `#22d3ee` | `#0b6e99`（Notion 蓝，可读） |
| `--accent-muted` | indigo 系深色半透明 | `#e7f3f8` |
| `--bubble-user` | indigo-950/20 + 边框 | `#eef0fd` + 浅 indigo 边 |
| `--bubble-assistant` | slate-950/60 | `#ffffff` + 灰边 |
| `--code-bg` | `#111827` | `#f7f6f3` |

Tailwind v4 通过 `@theme` 将 `--color-*` 映射为 utility（如 `bg-app`、`text-primary`），组件逐步替换 `bg-[#0b1020]` → `bg-app`。

**备选**：全面改用 `dark:` 前缀 — 拒绝，因当前默认类全是深色，迁移 diff 更大且易漏。

### D2：`useTheme` hook + `localStorage`

```text
key: "doc-agent-theme"
values: "dark" | "light"
default: "dark"
```

流程：

```text
App mount
  ├─ read localStorage
  ├─ set data-theme on <html>
  └─ ThemeToggle reflects state

User clicks toggle
  ├─ flip dark ↔ light
  ├─ update DOM + React state
  └─ write localStorage
```

在 `main.tsx` 或 `useTheme` 首帧同步设置 `data-theme`，避免 FOUC（首屏闪白）：内联 script 可选，MVP 在 `main.tsx` 顶层同步读 storage 即可。

**备选**：Rust IPC 持久化 — 拒绝，UI 偏好无需跨进程，localStorage 足够。

### D3：顶栏 `ThemeToggle` 组件

```
┌──────────────────────────────────────────────────────────────┐
│ [logo] Doc Agent   项目名…                    [ ◯━━━ toggle ]│
│                                               ml-auto        │
└──────────────────────────────────────────────────────────────┘
```

- 位置：`App.tsx` `header` 内，`ml-auto` 推到最右。
- 形态：switch toggle（非下拉）；`dark` 时显示月亮侧高亮，`light` 时太阳侧高亮（或单一 thumb + 图标）。
- `aria-label`：`切换到浅色模式` / `切换到深色模式`。
- 不挤占左侧品牌区（现有 `workspace-ui` Logo 要求不变）。

### D4：组件迁移策略

分两轮，降低漏改风险：

1. **基础设施**：`index.css` 变量 + `@theme`；`body` 使用 `bg-app text-primary`；`.tool-card`、`.panel`、`.markdown-body` 改用变量。
2. **组件批量替换**：按目录扫 `slate-*`、`indigo-*`、`text-white`、`border-slate-*` 等，映射到语义 class；优先 `App`、`Sidebar`、`ChatPanel`、`MessageBubble`、`ToolChainPanel`、`ProjectFileExplorer`、`ApiKeySection`、`ModelConfigSection`、`WebSearchSection`、`FileMentionPopup`、`InitCapsule`、`SuggestionCards`。

保留 `indigo` / `amber` 等**语义色**用于消息/思考区，但背景与边框走 token 或 `dark:`/`[data-theme=light]:` 限定变体。

### D5：Markdown 与代码高亮

- 容器：`prose-invert`（dark）↔ `prose`（light），由 `data-theme` 父级或 hook 传入 class。
- highlight.js：两 stylesheet 均打包；`MarkdownView` 根据 theme 给 `markdown-body` 加 `hljs-theme-dark` / `hljs-theme-light` class，在 `index.css` 用 `@import` 或条件加载对应 pre 背景（与 `--code-bg` 对齐）。
- KaTeX：沿用默认，两主题均可读。

### D6：Logo 适配

`logo.svg` 文档填充 `#0b1020` 在浅色底对比不足。改为：

- 描边保持 cyan/`var(--accent)` 或 `#22d3ee`；
- 文档填充改为 `currentColor` 或 `#ffffff`（light）/ `#0b1020`（dark）— 实现时用 CSS `filter` 或双 path + `data-theme` 选择器，MVP 优先 `currentColor` + 父级 `text-primary` 不适用时单独 `--logo-fill` token。

### D7：测试

- `useTheme.test.ts`：toggle 翻转、localStorage 读写、非法值回退 `dark`。
- `ThemeToggle` 组件测试：点击后 `data-theme` 变化（jsdom）。
- 现有组件测试：若断言具体 `slate-*` class，更新为语义 class 或移除脆弱断言。
- 无 Rust 变更，CI 矩阵不变。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 漏改某组件仍为深色块 | tasks 含全量 grep 验收；右栏/弹层列入清单 |
| 浅色对比度不足 | Notion _palette 参考；消息区/禁用态人工扫一眼 |
| FOUC 首屏闪色 | `main.tsx` 同步设 `data-theme` |
| Tailwind v4 `@theme` 与任意值混用 | design 定映射表，禁止新硬编码 hex |
| Logo 在浅色下不协调 | 独立 `--logo-fill` token |

## Migration Plan

- 纯前端增量；无 DB / 配置迁移。
- 未存 `localStorage` 的用户默认 `dark`，行为与现版一致。
- 回滚：移除 hook/toggle，恢复硬编码深色（git revert）。

## Open Questions

- （已决）两档 only、Notion 浅色、默认 dark、toggle、localStorage
- （实现时）toggle 具体视觉：纯 CSS switch vs 图标+switch 组合 — 实现阶段按现有侧栏控件风格统一即可
