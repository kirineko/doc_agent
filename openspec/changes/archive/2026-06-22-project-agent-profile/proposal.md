## Why

办公 Agent 需要在**项目级**共享规则（PPT 风格、公文格式、模板路径等），同时保持**会话上下文隔离**。Cursor 生态已用项目根 `AGENTS.md` 表达此类「项目记忆」；Doc Agent 目前仅有硬编码 system prompt，无持久化、可编辑的项目配置，且斜杠菜单全是 prompt 模板、无真命令。

本 change 引入 **AGENTS.md 注入**（手写与 init 解耦），并以 **`/init` 真斜杠命令 + clarify 多轮澄清** 作为可选的生成/更新入口，占用正常 Agent turn、使用当前会话模型，并读取项目文件与会话历史。

## What Changes

- **新增** 项目根 `AGENTS.md` 作为项目记忆载体：存在即注入 `build_working_messages` 的 system 段（每 turn 读盘，长度截断）；用户可**手写/外部编辑**，无需 `/init`
- **新增** 真斜杠 **command** 类型；注册 `/init`（显示原文 user 消息，可选尾部补充说明）
- **新增** `/init` turn 流程：Agent 读已有 AGENTS.md、扫描项目、结合当前会话历史，经 `clarify_ask` 逐题澄清，经 **`confirm_agents_md`** 确认预览后 `fs_write` 更新 `AGENTS.md`，turn 结束输出简短变更摘要
- **新增** `profile` skill（`skill_read profile`）定义 init 流程与问题库
- **约束**：clarify pending 时禁止 `/init`；空项目/空会话允许 init；Agent 仅通过 init/profile 流程写入 `AGENTS.md`（非 init 路径不写该文件）
- **修改** `project-session`：会话隔离的显式例外——共享 AGENTS.md 注入，不共享会话消息
- **修改** `clarify-interaction`：新增 `confirm_agents_md` 题型

## Capabilities

### New Capabilities

- `project-agent-profile`：AGENTS.md 路径/schema/注入/init turn/与 init 解耦

### Modified Capabilities

- `clarify-interaction`：新增 `confirm_agents_md` kind 与校验
- `slash-commands`：支持 `kind: command`；注册 `init`
- `project-session`：项目级 AGENTS.md 注入与会话隔离关系
- `agent-loop`：system 构造含 AGENTS.md
- `workspace-ui`：clarify pending 时禁止 `/init` 发送

## Impact

- **后端**：`agent/loop_support.rs`（注入）、`tools/clarify.rs`（新 kind）、`core/skills.rs`（profile skill）、`ipc` send 门禁、`tools` fs_write 路径策略（AGENTS.md）
- **前端**：`slashCommands.ts`（command 类型）、`slash.ts` Enter 分叉、`ClarifyQuestionCard`（confirm_agents_md 预览）、`types.ts` ClarifyKind
- **文档**：`assets/skills/profile/SKILL.md`
- **Spec**：归档合并至 `openspec/specs/project-agent-profile/spec.md` 等
- **依赖**：无新 crate

## 非目标

- 跨会话 LLM 摘要记忆（对话内容自动串会话）
- `/profile sync` 等 turn 外自动提炼（可后续 change）
- AGENTS.md 版本历史 / diff UI（MVP 仅 confirm 卡片全文预览 + 结束摘要）
