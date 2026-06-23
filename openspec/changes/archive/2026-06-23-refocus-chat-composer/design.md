## Context

`ChatPanel` 已有 `focusTextareaAt` 与 `composerDisabled` true→false 时的 refocus（带 composer 外 guard）。新策略对齐 Cursor：**鼠标点侧栏不影响点击功能，回合结束 / 切换会话后立即可继续输入**——通过积极 refocus 实现，而非保守 guard。

## Goals / Non-Goals

**Goals:**

- 回合结束、切换会话后无需点击即可继续输入
- Overlay 打开时不抢焦点
- composer 内 IME 组合输入不被 keydown 处理逻辑误触发（Enter 确认候选词不误发）

**Non-Goals:**

- busy 期间可编辑
- 修改后端

## Decisions

### 1. 统一 `shouldAllowComposerFocus`

集中判断：`projectSelected && !composerDisabled && !anyOverlayOpen`。

Overlay 含：Settings/Credentials Drawer、图片预览、slash flyout/popup、mention popup、Model Flyout、更新进度遮罩（phase !== idle）。

**不再**因 `activeElement` 在侧栏而跳过 refocus。

### 2. 回合结束：`composerDisabled` true → false

`useComposerFocus` 监听；允许则 `scheduleFocus(0)`。若变化原因为 `importing` 结束则跳过（导入已通过 `onFocusInput` 设好光标）。

### 3. 切换会话：监听 `sessionId` 变化

跳过首次 mount；`sessionId` 变化且允许 focus 时 refocus。覆盖 Sidebar 全部切换路径，无需改 `useWorkspace` 每处 `setActiveSessionId`。

### 4. composer keydown 的 IME 守卫

`handleChatInputKeyDown`（Enter 发送、Backspace 删占位符、斜杠/mention 弹层导航等）在函数顶部统一拦截 `event.nativeEvent.isComposing` 或 `keyCode === 229`，IME 组合中不拦截任何键——否则 Enter 确认候选词会误触发发送、Backspace 误删占位符等。

### 5. Model Flyout 状态

`Sidebar` 通过 `onModelFlyoutOpenChange` 上浮至 `App`，传入 `ChatPanel` blockers。

### 6. 提取 `useComposerFocus` hook

控制 `ChatPanel` 体量。自动 refocus 经 hook 内 `scheduleFocus`，受 `shouldAllowComposerFocus` 约束。

`focusTextareaAt` 仍留 `ChatPanel` 供 @/斜杠/导入等**用户已在 composer 流程内**的显式 focus；**不**走 overlay blocker 检查（例如 slash 弹层打开时用户选命令仍须 focus）。

`importing` true→false 时不触发回合结束 refocus，避免覆盖导入流程设置的光标。

hook **不**对外暴露 `scheduleFocus`（无外部消费者）。

## ADR: 放弃 type-to-focus

曾尝试实现 type-to-focus（焦点在非输入区按下可打印字符时，focus textarea 并将字符插入光标位置）。经多轮迭代（keydown 插入 → keyup/stroke 双重时序 → 仅 focus+setSelectionRange 不 preventDefault），在受控组件 `value={input}` + 真实 IME 环境下始终无法可靠工作：

- keydown 内同步 `focus()` 切换焦点会干扰 IME 对首键的组合——焦点切换边界下 `compositionstart` 时序不可靠
- React 受控组件重渲染会把 IME 中间态候选词回写为旧 state，进一步打乱组合
- 在本机任意输入法（中文/日文等）均能复现首字符被当英文字面量采集

根因是浏览器/IME 在焦点切换边界的行为，无法靠应用层事件捕获根治。**结论：删除 type-to-focus 设计**，仅保留回合结束 / 切换会话的主动 refocus。对应删除：`composerTypeToFocus.ts` 及其测试、`useComposerFocus` 的 keydown listener、相关 spec 场景。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| Drawer 打开时回合结束 refocus 抢焦点 | blockers 含 Drawer open |
| Enter 发送被 IME 确认候选词误触发 | `handleChatInputKeyDown` 顶部 `isComposing`/`keyCode===229` 守卫（详见 §4） |
| sessionId 与 composerDisabled 同帧双 focus | 共用 `scheduleFocus`，rAF 延迟 + policy 检查 |

## Migration Plan

纯前端；回滚删除 hook 与 listener 即可。
