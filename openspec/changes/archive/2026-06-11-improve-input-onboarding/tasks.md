## 1. 基础逻辑与 IPC

- [x] 1.1 新增 `DEFAULT_SESSION_CONFIG` 与 `pendingSessionConfig` 状态（App 或 hook）
- [x] 1.2 实现 `ensureSession()`：无 session 时 create + 更新列表与 activeSessionId
- [x] 1.3 扩展 `create_session` IPC：接受 `thinking_enabled` / `thinking_effort` 并传入 store
- [x] 1.4 `update_session` 后端：已有 user/assistant 消息时拒绝 model/thinking 变更 + 单测

## 2. 发送与项目切换

- [x] 2.1 `sendMessageContent`：先 `ensureSession`；无项目或无 provider Key 时展示 SendHint 并 return（保留 input）
- [x] 2.2 实现 `getSendBlockers()` 纯函数 + 单测
- [x] 2.3 项目切换：加载 sessions 后选中 `[0]` 或 `undefined`；修复跨项目 stale session
- [x] 2.4 切换项目时不清空 `input` state

## 3. Starter 与初始化胶囊

- [x] 3.1 移除空会话 auto-run starter 的 `useEffect` 与 key 保存补触发逻辑
- [x] 3.2 新增 `InitCapsule` 组件与空状态弱引导文案
- [x] 3.3 胶囊点击：`ensureSession` → `runStarter`；直接发送路径不调用 starter
- [x] 3.4 更新 `shouldRunStarter` / 相关测试以反映显式触发

## 4. 侧栏重组

- [x] 4.1 API Key 区移至全局位置（DeepSeek + Kimi），已保存折叠、未配置展开
- [x] 4.2 模型区独立：草稿态/空会话可编辑；有消息只读
- [x] 4.3 「新建」保留，去掉任何 starter 副作用；传入 pendingSessionConfig
- [x] 4.4 删除原 `activeSession` 内嵌 Key 区块

## 5. ChatPanel 与 Hint

- [x] 5.1 新增 ephemeral `SendHintBanner`（发送阻断时展示）
- [x] 5.2 空状态集成 InitCapsule；无 DeepSeek key 时不渲染胶囊
- [x] 5.3 发送按钮：有内容即可点（busy/initializing 除外），阻断逻辑在 send handler

## 6. 验证

- [x] 6.1 前端单测：send blockers、项目切换 session 选择、模型锁定 UI 条件
- [x] 6.2 后端单测：update_session 锁定
- [x] 6.3 本地自检：`npm test`、`cargo test`、手动走通草稿发送 / 胶囊 / 切换项目保留输入
