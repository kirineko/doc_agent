## Why

用户难以一次性写出能触发 Agent 正确工具链（skill_read、OOXML、data_query 等）的提示词；现有 LLM「推荐问」动态且短，无法稳定覆盖 Word/PPT/Excel/PDF/Web 等主流程。需要在输入框提供**静态、分类、可搜索**的斜杠命令菜单，选中后填入简短模板供用户修改再发送。

## What Changes

- 输入框支持 `/` 触发的斜杠命令选择器：fzf 式模糊搜索、键盘导航、分组展示
- 内置 **22 条**静态命令（模板 20–100 字），覆盖系统主要文档能力；**阅读类仅保留 `read`**；**PDF 排版类基于 Typst**（`pdf:create` 新建、`pdf:edit-typst` 修订）
- 默认分类排序：**general → word → ppt → excel → pdf → web**
- 选中命令后将 `/query` 替换为预置 `prompt` 填入输入框，**不自动发送**；用户可编辑后 Enter 发送
- 与现有 `@` 文件引用、推荐问胶囊、澄清卡片互斥/共存规则对齐
- 纯前端实现（无新 IPC、无新 npm 依赖）

## Capabilities

### New Capabilities

- `slash-commands`：静态命令 registry（id、分类、label、keywords、prompt）、搜索与排序规则

### Modified Capabilities

- `workspace-ui`：ChatPanel 斜杠弹层 UI、键盘交互、placeholder 提示、与 `@` 互斥

## Impact

- 前端：`src/lib/slashCommands.ts`（registry）、`src/lib/slash.ts`（检测/替换）、`src/lib/slashFuzzy.ts` 或扩展 `fuzzy.ts`、`src/components/SlashCommandPopup.tsx`、`ChatPanel.tsx`
- 测试：`slash.ts` / `slashFuzzy` 单测；`SlashCommandPopup` 或 ChatPanel 集成测（可选）
- 无 Rust / IPC / 依赖变更

## 纳入 / 排除

**纳入**

- 22 条命令（见 design.md 清单）
- 模糊搜索、分类分组、默认排序
- 选中填 prompt、可编辑、不直发

**排除**

- 各格式单独的 read 命令（`word:read`、`ppt:read`、`pdf:read`、`excel:read`）
- Typst 作为独立斜杠分类（归入 pdf 子命令，不单独设 typst 类）
- 命令使用统计、最近使用、服务端下发模板
- 斜杠命令自动发送或绕过 clarify 的后端逻辑
