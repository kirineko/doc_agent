## Context

当前 `App.tsx` 在 `!activeSessionId` 时 `sendMessageContent` 静默 return；`useEffect` 在空会话选中时自动 `runStarter`；侧栏 API Key 与模型配置均包在 `activeSession` 条件下。`list_sessions` 已按 `updated_at DESC` 排序。`create_session` IPC 默认 `deepseek-v4-flash` + `thinking_enabled=true` + `effort=high`，但前端 Sidebar 新建未传 thinking 且 UI checkbox 对新建无效。

## Goals / Non-Goals

**Goals:**

- 草稿态可输入；发送懒建会话（共用 `pendingSessionConfig`）
- 初始化胶囊为 starter 唯一入口；直接发送跳过 starter
- 切换项目 → 最近会话或草稿态；输入框保留
- Key 全局、模型分区；有消息后锁定模型
- 发送阻断时一次性 hint，保留输入

**Non-Goals:**

- 自动创建项目 / 自动选目录
- 切换项目时校验 `@` 引用有效性
- 在 UI 说明 starter 使用 DeepSeek
- 修改 followup 推荐逻辑
- 修改 starter 后端模型或提示词

## Decisions

### 1. 草稿态与 `ensureSession()`

前端维护 `pendingSessionConfig`（默认 `deepseek-v4-flash` / thinking on / high）。`ensureSession()`：若已有 `activeSessionId` 则返回；否则 `create_session` 并选中。懒发送、新建、初始化胶囊共用。

**备选**：虚拟 session id — 拒绝，增加后端复杂度。

### 2. Starter 仅胶囊触发

删除 `App.tsx` 中空会话 auto-run `useEffect`；删除 `handleApiKeyStatusChange` 中补触发 starter。`runStarter()` 仅由初始化胶囊（及将来显式 re-trigger）调用。

未配置 DeepSeek Key → 不展示初始化胶囊（功能整体不可用，与现有 smart-suggestions 门控一致）。

### 3. 项目切换

`onSelectProject(id)`：加载 sessions 后 `setActiveSessionId(sessions[0]?.id)`（最近或 `undefined`）。必须清除跨项目 stale session。`input` state 不重置。

### 4. 侧栏布局

```
项目
API Key（DeepSeek / Kimi，与会话无关，已保存折叠）
会话 + [新建]
模型（见可见性规则）
```

Key 不再绑定 `activeProvider`。发送时按**当前 pending/会话 model** 对应 provider 检查 Key。

### 5. 模型区三态

| 状态 | UI |
|------|-----|
| 草稿态 / 空会话（0 chat 消息） | 可编辑 model + thinking |
| 有 chat 消息 | 只读 label |
| initializing | 禁用编辑 |

锁定条件：`countChatMessages > 0`。后端 `update_session` 若 session 已有 user/assistant 消息且 model/thinking 变更 → 返回错误。

### 6. 默认模型

懒创建、新建、未改模型时均使用 `deepseek-v4-flash` + thinking enabled + high。`create_session` IPC 改为接受 `thinking_enabled` / `thinking_effort` 入参（与 model 一并传入）。

### 7. Hint 策略

仅 **尝试发送** 且被阻断时展示 ephemeral banner（~3s 或可关闭）：

- 无项目 → 高亮项目区
- 无对应 provider Key → 展开 Key 区并 focus

草稿态、空会话正常输入：**不展示**常驻 hint。中间空状态仅弱文案「或直接输入开始对话」。

### 8. 初始化胶囊

组件 `InitCapsule` 置于 ChatPanel 空状态区。展示条件：

```
activeProjectId &&
apiKeyStatus.deepseek &&
chatMessageCount === 0 &&
!initializing && !busy &&
starterSuggestions.length === 0
```

点击 → `ensureSession()` → `runStarter()`。

## Risks / Trade-offs

- **[Risk] 切换项目后 `@` 路径失效** → MVP 不处理；用户自行修正
- **[Risk] 懒创建与会话列表不同步** → `ensureSession` 后更新 `sessions` 列表
- **[Risk] 模型锁定仅靠前端** → 后端 `update_session` 二次校验
- **[Risk] 删除 auto-starter 后用户不知有推荐功能** → 空状态胶囊 + 弱提示

## Migration Plan

纯前端 + 轻量 IPC 变更，无 DB migration。发布后旧会话行为：有消息则模型已锁定；空会话不再自动 initializing。

## Open Questions

（无 — 设计已定稿。）
