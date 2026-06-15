## 1. 路径常量模块

- [x] 1.1 新增 `src-tauri/src/core/cache_paths.rs`，定义 `CACHE_ROOT`、`ATTACHMENTS_DIR`、`SKILL_RUN_*`、`PDF_CACHE_ROOT` 及 helper（如 `attachment_rel_path(name)`）
- [x] 1.2 在 `core/mod.rs` 导出 `cache_paths`；`pdf_cache.rs` 改为引用该模块（移除重复 `CACHE_ROOT` 字面量）

## 2. 后端路径替换

- [x] 2.1 `skill_run_tmp.rs`：全部改为 `cache_paths` 常量
- [x] 2.2 `ipc/mod.rs`：`save_upload` 写入 `.cache/attachments/`；`read_attachment_preview` 校验前缀同步
- [x] 2.3 `openai_compat.rs`：`is_upload_attachment_path` 改为 `.cache/attachments/` 前缀；更新相关单测
- [x] 2.4 `loop_runner.rs` / `loop_support.rs`：注释与 cleanup 引用新路径（逻辑不变）
- [x] 2.5 `skill.rs`、`tool_args.rs`、`fs.rs`、`runtime/diagnostics.rs`：tool 描述与 hint 中的路径示例更新
- [x] 2.6 `image_read.rs`：tool 描述中 `.uploads/` → `.cache/attachments/`（若有）

## 3. Skills 与文档

- [x] 3.1 更新 `assets/skills/docx|pptx|xlsx/SKILL.md` 中 `.skill-run/script.js` → `.cache/skill-run/script.js`
- [x] 3.2 更新 `assets/skills/html-report/SKILL.md` 禁止路径段
- [x] 3.3 更新 `assets/skills/pdf/SKILL.md` 与 `reference.md`（若含 `.uploads/` 引用）— 无引用，跳过

## 4. 测试

- [x] 4.1 更新 `src-tauri/src/tools/tests.rs` 中所有 `.skill-run` 路径断言
- [x] 4.2 更新 `openai_compat` attachment 单测路径
- [x] 4.3 更新前端 `attachments.test.ts`、`messages.test.ts`、`MessageList.test.tsx` fixture 路径
- [x] 4.4 运行 `cargo test`、`npm test` 确认通过

## 5. 收尾

- [x] 5.1 更新 `CHANGELOG.md` `[Unreleased]`：BREAKING 路径变更摘要
- [x] 5.2 自检：项目根新写入仅产生 `.cache/` 树；旧 `.uploads/`/`.skill-run/` 不被读写
