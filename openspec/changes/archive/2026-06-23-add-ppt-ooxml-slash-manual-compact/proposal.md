## Why

PPT 斜杠命令 `ppt:edit` 的 prompt 过于笼统，Agent 常默认走 `skill_run` / PptxGenJS 脚本路径，难以对已有 `.pptx` 做 OOXML 级精准修改（改文案、保留版式）。Word 已有对称的 `word:edit`（解包改 XML），PPT 缺少等价入口。

上下文自动压缩已在接近模型上限时生效，但用户无法在对话仍较长、尚未触发 85% 阈值时主动释放空间；也无法在感知变慢时自行压缩，只能等待自动触发。

## What Changes

- **新增** 斜杠模板 `ppt:edit-ooxml`（「精准修改 PPT」）：prompt 明确 `ooxml_unpack` → 编辑 `slide{N}.xml` → `ooxml_pack`，禁止 JS 脚本路径
- **修改** `ppt:edit` 的 label/description/prompt，明确为「脚本编辑 PPT」（`skill_read pptx/pptxgenjs.md` → `skill_run`），与 OOXML 命令分工
- **新增** 斜杠 command `compact`（与 `init` 同组）：用户手动触发上下文压缩
- **新增** IPC `compact_session`：跳过自动压缩阈值，调用与自动压缩共享的核心管线；**不**写入 `/compact` 用户消息；归档 + 摘要消息与自动压缩一致
- **修改** `context_compacted` 事件增加 `trigger: "auto" | "manual"`，前端按来源展示不同轻提示文案
- **修改** 注册表模板条目数 22 → **23**（新增 `ppt:edit-ooxml`）；command 组含 `init` 与 `compact`

## Capabilities

### New Capabilities

（无独立新 capability；行为并入现有 spec delta。）

### Modified Capabilities

- `slash-commands`：`ppt:edit-ooxml` 模板；`ppt:edit` prompt 调整；`compact` command 注册与发送分叉
- `context-compaction`：手动压缩入口、`compact_session` IPC、与自动压缩共享 core、无操作（历史过短）与阻断条件
- `workspace-ui`：手动压缩轻提示、`/compact` 发送拦截与阻断（clarify / turn running）、`context_compacted.trigger` 契约

## Impact

- **后端**：`agent/compaction.rs`（抽取 `compact_session_core`、新增 `force_compact_session`）、`ipc/mod.rs`（`compact_session` command）、`agent/types.rs`（`ContextCompacted` 增加 `trigger`）
- **前端**：`slashCommands.ts`、`useWorkspace.ts`（`/compact` 拦截 invoke）、`profileInit.ts` 旁新增 `isCompactMessage`、`agentEvents.ts` / `types.ts`（事件 payload）、`ChatPanel.tsx`（compact command 阻断 clarify）
- **测试**：`slash.test.ts`、`compaction_tests.rs`、IPC 契约测试、`agentEvents.test.ts`
- **依赖**：无新 crate

## 非目标

- 调整自动压缩阈值（85% / reserved）或 `MAX_PRESERVED_MESSAGES`
- 手动压缩支持尾部说明（`/compact 原因`）或自定义保留轮数
- 压缩摘要 UI  diff / 归档消息浏览
- 将 `ppt:edit` 改为仅 OOXML（保留脚本路径为 `ppt:edit`）
