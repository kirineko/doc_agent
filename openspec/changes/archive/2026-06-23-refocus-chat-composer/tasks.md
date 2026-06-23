## 1. 焦点策略模块

- [x] 1.1 新建 `composerFocusPolicy.ts`：`shouldAllowComposerFocus` + blockers 类型
- [x] 1.2 新建 `useComposerFocus.ts`：回合结束 refocus、sessionId refocus

## 2. UI 接入

- [x] 2.1 `ChatPanel.tsx`：接入 hook；移除旧 composer 外 guard；保留 `focusTextareaAt` 供 @/斜杠
- [x] 2.2 `App.tsx`：传入 Settings/Credentials/ModelFlyout blockers
- [x] 2.3 `Sidebar.tsx`：`onModelFlyoutOpenChange` 上浮 Flyout 状态

## 3. 测试

- [x] 3.1 `composerFocusPolicy.test.ts`
- [x] 3.2 更新 `ChatPanel.test.tsx`（总是 refocus、Drawer 抑制、session 切换）
- [x] 3.3 `npm run typecheck && npm test` 通过
- [x] 3.4 review 清理：`focusTextareaAt` 去 guard、移除未用 `scheduleFocus` 导出
- [x] 3.5 `chatInputKeyDown.ts` 顶部加 isComposing/keyCode 229 守卫；新增 `chatInputKeyDown.test.ts`

## 4. 废弃 type-to-focus

- [x] 4.1 删除 `composerTypeToFocus.ts` 及其测试
- [x] 4.2 删除 `useComposerFocus` 的 type-to-focus keydown listener 及相关参数/测试
- [x] 4.3 删除 `ChatPanel.test.tsx` 中 type-to-focus 测试
- [x] 4.4 更新 proposal/design/spec，记录废弃 ADR
- [x] 4.5 `npm run typecheck && npm test` 通过

## 5. 验收

- [x] 5.1 手测：回合结束、切会话 refocus
- [x] 5.2 手测：Drawer 打开时不抢焦点
- [x] 5.3 手测：中文输入法按 Enter 确认候选词不误发消息
