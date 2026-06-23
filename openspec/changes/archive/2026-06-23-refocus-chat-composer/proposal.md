## Why

用户发送消息、切换会话或在侧栏/文件区用鼠标操作后，Chat 输入框经常失去焦点，无法像 Cursor 等产品一样「随时可打字」。需在回合结束、切换会话时主动 refocus，并对 composer 自身的 keydown 处理加入 IME 守卫。

## What Changes

- **回合结束总是 refocus**：composer 从 disabled 恢复可编辑时，**无条件** focus textarea（不再因焦点在侧栏而跳过）。
- **切换会话 refocus**：`activeSessionId` 变化且 composer 可编辑时，自动 focus textarea。
- **Overlay 抑制**：Settings/Credentials Drawer、图片预览、斜杠/mention 弹层、Model Flyout、更新遮罩、composer disabled / 无项目时，不自动 refocus。
- **composer keydown IME 守卫**：`handleChatInputKeyDown` 在 IME 组合中（`isComposing` / `keyCode === 229`）不拦截任何键，避免 Enter 确认候选词误触发发送。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `workspace-ui`：Chat 输入区焦点策略（回合结束、切换会话、Overlay 抑制）与 composer keydown 的 IME 守卫

## Impact

- **前端**：`composerFocusPolicy.ts`、`useComposerFocus.ts`、`chatInputKeyDown.ts`；`ChatPanel.tsx`、`App.tsx`、`Sidebar.tsx`（Model Flyout 状态上浮）；测试
- **后端 / IPC / 依赖**：无

## 纳入 / 排除

**纳入**

- turn 结束 / 取消 / 发送失败恢复 → refocus
- 侧栏切换会话、新建会话 → refocus
- Overlay 打开时不 refocus
- composer keydown 的 IME 守卫（Enter 确认候选词不误发）

**排除**

- busy 期间可编辑或消息排队
- type-to-focus（非输入区按键自动进入 composer）：在受控组件 + 真实 IME 环境下无法可靠工作，焦点切换会干扰 IME 对首键的组合，已放弃
- 全局 focus trap
