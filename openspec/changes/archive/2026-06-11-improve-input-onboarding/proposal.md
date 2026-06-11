## Why

输入框始终可见，但发送依赖「已选项目 + 已选/建会话 + API Key」等前置条件；当前缺失会话或项目时发送静默失败，用户无法感知原因。同时，空会话自动触发推荐问初始化打断高频「直接输入」路径。需改为：允许随时输入，仅在必要时引导；推荐问初始化改为显式 opt-in。

## What Changes

- 支持**草稿态**（已选项目、无 activeSession）：可输入，发送时懒创建会话，不触发 starter
- 中间空状态提供**初始化胶囊**；点击后建会话（若需）并触发 starter；直接发送则跳过 starter
- 移除空会话打开 / 首次保存 Key 时的**自动 starter** 触发
- 切换项目时选中该项目**最近会话**；无会话则进入空白草稿态；**不清空**输入框
- 侧栏 **API Key 与模型配置分离**；Key 全局可见，已配置时默认收起
- 未选项目或无 Key 时**尝试发送**才展示一次性 hint，不清空已输入内容
- 默认模型：**DeepSeek V4 Flash + thinking enabled + effort high**
- 会话一旦有 user/assistant 消息，**锁定模型**不可再切换
- 侧栏「新建」保留，建空会话但不自动 starter
- 推荐问生成始终使用 DeepSeek；未配置 DeepSeek Key 时不展示初始化胶囊及相关功能

## Capabilities

### New Capabilities

（无新增 capability 目录；行为增量写入既有 spec delta。）

### Modified Capabilities

- `workspace-ui`：草稿态输入、初始化胶囊、发送阻断 hint、项目切换行为、侧栏分区
- `smart-suggestions`：starter 改为显式触发；DeepSeek Key 门控与初始化胶囊联动
- `project-session`：懒创建会话、切换项目选中最近会话
- `model-config`：Key 全局入口、默认模型、会话模型锁定

## Impact

- 前端：`App.tsx`、`Sidebar.tsx`、`ChatPanel.tsx`；新增 `ensureSession` / `pendingSessionConfig` / send readiness 逻辑；可能新增小组件（InitCapsule、SendHintBanner）
- 后端：`create_session` IPC 接受 thinking 参数；`update_session` 拒绝已有消息的会话变更 model/thinking
- Spec：`workspace-ui`、`smart-suggestions`、`project-session`、`model-config` delta
- 测试：发送 readiness、项目切换、模型锁定、starter 触发条件
