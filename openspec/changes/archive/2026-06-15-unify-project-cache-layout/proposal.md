## Why

项目沙箱内同时存在 `.skill-run/`、`.uploads/`、`.cache/pdf/` 三个点目录，命名与层级不一致，路径常量分散在多处。多模态与 PDF 缓存尚未发版，现在统一为单一 `.cache/` 根目录成本最低，且便于定义各子目录的产品边界（可重建 cache vs 会话附件）。

## What Changes

- 引入统一项目缓存根目录 `.cache/`，项目根下仅保留这一个点目录（对用户文件浏览隐藏）
- **BREAKING**：用户粘贴图片写入 `.cache/attachments/`（原 `.uploads/`）
- **BREAKING**：`skill_run` 临时脚本恢复区迁至 `.cache/skill-run/`（原 `.skill-run/`）
- `.cache/pdf/<cache_key>/` 路径保持不变
- 新增 `core/cache_paths`（或等效模块）集中定义上述相对路径常量
- 定义 cache 边界语义：哪些子目录可安全重建、附件缺失时的降级行为；**本变更不做**「清理 cache」UI 或自动 GC
- **不**迁移或删除旧目录（`.uploads/`、`.skill-run/`）；**不**兼容历史路径与历史缓存

## Capabilities

### New Capabilities

- `project-cache-layout`：统一 `.cache/` 目录树、子目录职责、隐藏规则、附件缺失降级、与可重建 cache 的边界定义

### Modified Capabilities

- `multimodal-input`：附件落盘路径由 `.uploads/` 改为 `.cache/attachments/`
- `script-runtime`：`skill_run` 临时恢复区路径改为 `.cache/skill-run/`
- `workspace-ui`：历史附件展示场景中的路径示例更新
- `html-report`：禁止写入临时目录的路径段由 `.skill-run` 改为 `.cache/skill-run`
- `agent-loop`：turn 结束清理所引用的临时目录路径更新
- `image-read-tool`：示例路径与「附件路径限制」表述对齐新布局（vision 可读 `.cache/` 下页图与附件）

## Impact

- Rust：`core/cache_paths.rs`（新）、`tools/skill_run_tmp.rs`、`tools/pdf_cache.rs`、`ipc/mod.rs`（`save_upload`）、`agent/provider/openai_compat.rs`（`is_upload_attachment_path`）、`agent/loop_runner.rs`、`tools/skill.rs`、`agent/tool_args.rs`、`tools/fs.rs`、`tools/runtime/diagnostics.rs`
- 前端：测试 fixture 中的路径字符串；运行时逻辑无硬编码路径（依赖后端返回）
- OpenSpec / Skills：`assets/skills/{docx,pptx,xlsx,pdf,html-report}/SKILL.md` 与 tool 描述中的路径示例
- 测试：`src-tauri/src/tools/tests.rs` 及 attachment 相关单测
- 排除：清理 cache UI、DB 路径迁移、旧目录删除、pending chip 移除时删盘、`.gitignore` 变更
