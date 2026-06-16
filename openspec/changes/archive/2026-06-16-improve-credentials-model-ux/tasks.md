## 1. 密钥 Drawer 与 Header 入口

- [x] 1.1 新增 `CredentialsButton.tsx`（与 `SettingsButton` 同尺寸；无 LLM Key 时 amber dot）
- [x] 1.2 新增 `CredentialsDrawer.tsx`：标题「密钥与服务」；嵌入 `ApiKeySection` + Tavily Key 分区
- [x] 1.3 从 `WebSearchSection.tsx` 提取 `TavilyKeyPanel` 供 Drawer 复用
- [x] 1.4 新增 `WebSearchStatus.tsx`：侧栏仅状态 + 引导打开密钥 Drawer
- [x] 1.5 在 `App.tsx` Header 挂载密钥按钮与 Drawer；`useWorkspace` 增加 `credentialsOpen` / `highlightApiKeyProvider` wiring

## 2. 模型 Flyout

- [x] 2.1 新增 `useAnchorPosition`（或等价 hook）：基于 trigger ref 计算 fixed 定位，支持向上/向下 fallback
- [x] 2.2 新增 `ModelFlyout.tsx`：摘要条、Provider segmented、scroll 模型列表、sticky 思考区；保留 Tab 预选首模型
- [x] 2.3 更新 `Sidebar.tsx`：模型 trigger 仅 `activeProjectId` 时显示；移除 Key 相关文案
- [x] 2.4 删除 `ModelSettingsDrawer.tsx` 并清理引用；`modelSettingsOpen` 重命名为 `modelFlyoutOpen`

## 3. 弱提醒与发送拦截

- [x] 3.1 新增 `CredentialsHintBanner.tsx`：`!anyLlmKey` 时 Header 弱提醒（无 localStorage dismiss；可关闭当次）
- [x] 3.2 更新 `showSendBlocker`：缺 Key 时打开 `credentialsOpen` 而非 model flyout
- [x] 3.3 更新 `SendHintBanner` 文案/行为与 spec 一致

## 4. 清理与验证

- [x] 4.1 移除侧栏 `WebSearchSection` Key 表单；Sidebar 底部改用 `WebSearchStatus`
- [x] 4.2 删除未使用组件/import（如 `ProviderApiKeyPanel` 在 Flyout 路径下的引用）
- [x] 4.3 跑 `npm run typecheck && npm test && npm run build`；手动验证：启动无 Key 弱提醒、密钥 Drawer、Flyout 锚点、发送拦截跳转
