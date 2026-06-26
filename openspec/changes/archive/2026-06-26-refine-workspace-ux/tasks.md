## 1. 布局与 Inspector 基础

- [x] 1.1 新建 `InspectorTabs.tsx`：三 Tab（项目文件 / 工具调用链 / 构建产物）+ 徽标
- [x] 1.2 重构 `RightPanel.tsx`：移除 vertical `Group`，接入 `InspectorTabs`
- [x] 1.3 调整 `ToolchainTabArea` / 各 panel：去除重复 header，适配 Inspector 容器
- [x] 1.4 更新 `workspaceLayout.ts`：移除右侧 vertical layout 常量/存储；新增 Inspector Tab localStorage key
- [x] 1.5 默认 Tab 为「项目文件」；设置抽屉「重置布局」同步清理 Inspector Tab 缓存

## 2. Inspector 智能 Tab 切换

- [x] 2.1 实现 per-turn `userPinnedTab` 状态（挂接 `useWorkspace` / stream events）
- [x] 2.2 首个 tool running 时 auto-switch 至工具链（尊重 pin）
- [x] 2.3 新 turn（用户 send）清除 pin；产物累积仅更新 badge
- [x] 2.4 单测：`inspectorTabPolicy` 或等价纯函数覆盖 auto-switch / pin / 新 turn

## 3. 侧栏项目折叠组

- [x] 3.1 新建 `ProjectSessionTree.tsx`：手风琴（仅展开 active 项目）、会话缩进列表
- [x] 3.2 侧栏顶部动作区：新建会话（⌘N 提示）、搜索入口
- [x] 3.3 ghost「＋ 添加项目目录」替换全宽 primary 按钮
- [x] 3.4 项目行 `[+]`：非 active 时先 `selectProject` 再 `createSession`
- [x] 3.5 项目行 `···` 菜单：在文件夹中打开（`open_project_root`）、从列表移除
- [x] 3.6 重构 `Sidebar.tsx` 接入树组件；**移除**侧栏模型区块；保留 Web 搜索底部区
- [x] 3.7 组件测试：展开/折叠、项目内新建、菜单动作；**断言侧栏无模型 trigger**

## 4. Composer 上下文条与顶栏

- [x] 4.1 新建 `ComposerContextBar.tsx`：项目、**模型 trigger**、AGENTS.md、上下文 %
- [x] 4.2 将 `ModelFlyout` 锚定迁至上下文条（`#composer-model-trigger`）；删除侧栏 `#sidebar-model-trigger`
- [x] 4.3 从 `ChatPanel` 标题行迁出 `ContextUsageIndicator`、`AgentsMdIndicator`；`App.tsx` 的 `modelFlyoutOpen` 改由 ContextBar 上报
- [x] 4.4 `App.tsx` 顶栏移除项目名副标题
- [x] 4.5 项目名 popover 切换（或跳转侧栏）与 send blocker 高亮联动
- [x] 4.6 测试：侧栏无模型入口、上下文条 Flyout 切换模型、有消息会话 Flyout 只读

## 5. 空态居中 Composer

- [x] 5.1 `ChatPanel` 增加 `layoutMode: empty | chat` 分支布局
- [x] 5.2 空态：居中 max-width composer + 问候/弱引导 + Init 胶囊
- [x] 5.3 首条消息后过渡至底部 dock（CSS transition，保留 input 策略）
- [x] 5.4 测试：空态渲染、切换会话回空态、发送后 layout 切换

## 6. 命令面板

- [x] 6.1 新建 `CommandPalette.tsx` + `lib/commandPaletteSearch.ts`（fuzzy 分组）
- [x] 6.2 注册 ⌘K / Ctrl+K 全局快捷键；侧栏搜索按钮打开同一面板
- [x] 6.3 接入：切换项目/会话、添加项目、新建会话、斜杠命令 insert prompt
- [x] 6.4 单测：搜索排序、Enter 行为、Esc 关闭

## 7. Notion 视觉 token

- [x] 7.1 更新 `index.css`：侧栏项 hover/active、composer 圆角 shadow
- [x] 7.2 `WorkspaceLayout` 去 `main` 外 padding；栏间 flat 分隔
- [x] 7.3 侧栏/会话项移除重 border card 样式
- [x] 7.4 dark 主题同步 flat 调整（不改 hue 体系）

## 8. 验证

- [x] 8.1 更新/新增 Vitest：`Sidebar`/`ProjectSessionTree`、`InspectorTabs`、`CommandPalette` 相关测试
- [x] 8.2 `npm run typecheck && npm test && npm run build`
- [x] 8.3 `cd src-tauri && cargo test`（若无 Rust 变更则 smoke）
- [x] 8.4 手动走查：添加项目、项目菜单打开 Finder、空态居中、Agent 运行 auto Tab、⌘K 切换会话
