## Context

当前项目沙箱内有三类隐藏目录，均由 `should_skip_name`（路径段以 `.` 开头）从 `@` 与文件浏览中排除：

| 现路径 | 用途 | 生命周期 |
|--------|------|----------|
| `.uploads/` | 用户粘贴图片 | 持久；`attachments_json` 引用 |
| `.skill-run/` | `skill_run` 脚本与错误现场 | turn 内临时；自动清理 |
| `.cache/pdf/` | PDF 页渲染 | 持久；源文件/参数变则失效 |

路径常量分散在 `skill_run_tmp.rs`、`pdf_cache.rs`、`ipc/mod.rs`、`openai_compat.rs` 等处。PDF vision 设计曾 defer `.skill-run/` 迁入 `.cache/`，现与 attachments 一并收口。

约束（来自讨论）：

- 子目录名用 `attachments`（非 `uploads`）
- 不迁移、不删除旧根目录（`.uploads/`、`.skill-run/`）
- 不做 cache 清理 UI；只定义边界
- 不必兼容历史路径；多模态/PDF 缓存尚未发版

## Goals / Non-Goals

**Goals:**

- 项目根仅一个点目录 `.cache/`，其下 `attachments/`、`skill-run/`、`pdf/`
- 所有读写上述目录的代码引用集中常量模块
- 在 spec 中明确三类子目录的产品边界与附件文件缺失时的降级行为
- 更新 Agent 可见的 tool 描述与 skills 中的路径示例

**Non-Goals:**

- 「清理 cache」按钮或自动 GC
- 删除/迁移用户磁盘上已有的 `.uploads/`、`.skill-run/`
- DB `attachments_json` 历史路径迁移
- pending 附件 chip 移除时删除磁盘文件
- 修改 `.gitignore`

## Decisions

### D1. 目录布局

```
.cache/
  attachments/          # 用户粘贴图片，UUID 文件名
  skill-run/
    script.js
    error.json
  pdf/<cache_key>/
    manifest.json
    page_NNN.png
```

项目根不再新建 `.uploads/` 或 `.skill-run/`。`.cache/pdf/` 保持现有 `cache_key` 与 manifest 语义不变。

**备选**：保留 `.uploads/` 仅迁 skill-run — 拒绝，无法达成「单一点目录」目标。

### D2. 集中常量模块 `core/cache_paths.rs`

```rust
pub const CACHE_ROOT: &str = ".cache";
pub const ATTACHMENTS_DIR: &str = ".cache/attachments";
pub const SKILL_RUN_DIR: &str = ".cache/skill-run";
pub const SKILL_RUN_SCRIPT: &str = ".cache/skill-run/script.js";
pub const SKILL_RUN_ERROR: &str = ".cache/skill-run/error.json";
pub const PDF_CACHE_ROOT: &str = ".cache/pdf";
```

`pdf_cache.rs` 的 `CACHE_ROOT` 改为 re-export 或引用该模块，避免双源。`skill_run_tmp.rs` 删除本地 `TMP_DIR` 等重复常量。

`is_upload_attachment_path` 校验前缀改为 `.cache/attachments/`（仍禁止 `..`）。

### D3. Cache 边界（产品语义，本变更仅文档化 + 实现路径一致）

| 子目录 | 性质 | 用户手动删除后 |
|--------|------|----------------|
| `pdf/` | 可重建派生缓存 | 下次 `pdf_read`/`pdf_render_pages` 重渲 |
| `skill-run/` | 可重建临时现场 | 失败修复丢失；成功 run 本会自动清 |
| `attachments/` | 会话附件（非 DB 内嵌） | UI 缩略图「无法加载」；Agent 重建上下文时静默跳过缺失文件（现有 `messages_from_store` 行为）；文字消息保留 |

将来若做「清 cache」UI，默认应只清 `pdf/` + `skill-run/`，`attachments/` 需单独确认 — **本变更不实现**。

### D4. 旧目录策略：不读、不写、不删

- 新代码只写 `.cache/…`
- 不扫描或迁移 `.uploads/`、`.skill-run/` 内容
- 不添加 fallback 读取旧路径
- 旧目录若存在，对用户透明（仍被点目录规则隐藏）

### D5. 隐藏与沙箱

沿用现有点目录隐藏规则；`.cache` 作为唯一根点目录，其子目录名不含前导 `.`，但不会出现在 `@` 列表（祖先 `.cache` 已被 `should_skip_entry` 跳过）。

沙箱 `resolve` / `resolve_for_write` 行为不变；仅相对路径字符串更新。

### D6. Agent 与文档同步

必须同步更新（否则 Agent 会写旧路径）：

- `skill_run` tool JSON 描述与 `tool_args` hint
- `fs_patch` 描述中的示例路径
- `assets/skills/docx|pptx|xlsx|SKILL.md` 中的 `.skill-run/script.js` → `.cache/skill-run/script.js`
- `html-report` skill 禁止路径段
- `pdf` skill 中若引用 uploads 则改为 attachments

## Risks / Trade-offs

- **[Risk] `.cache/attachments` 名称暗示可删** → spec 与 design 明确其为会话附件；未来清 cache UI 须分区处理
- **[Risk] Agent 仍输出旧路径** → tool 描述与 skill 全量更新；单测覆盖 `script_path` 新值
- **[Risk] 粘贴后未发送的孤儿文件** → 现有行为（chip 移除不删盘）；本变更不解决，可在后续 change 处理
- **[Trade-off] attachments 放在 cache 根下** → 换取单一点目录；产品文案将来需区分「系统 cache」与「聊天附件」

## Migration Plan

- 部署：纯路径替换，无 DB migration
- 发版前功能未对外发布，无用户迁移负担
- 回滚：还原常量与 spec 即可；新路径下已写文件可手动忽略

## Open Questions

- （已关闭）attachments 子目录名 → `attachments`
- （已关闭）旧目录 → 不迁移不删除
- （已关闭）清理 UI → 本变更不做
