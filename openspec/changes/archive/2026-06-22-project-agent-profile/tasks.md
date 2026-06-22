## 1. 能力 A — AGENTS.md 读盘注入（优先）

- [x] 1.1 `loop_support.rs`：实现 `read_agents_md_for_inject`（沙箱读 `AGENTS.md`，3000 字符截断，按 design D5 节优先级）
- [x] 1.2 `build_working_messages`：在 skills 索引后追加 `## 项目配置（AGENTS.md）` 段
- [x] 1.3 Rust 单测：无文件跳过、有文件注入、超长截断
- [x] 1.4 手工验证：项目根放置 `AGENTS.md`，任意 session turn 的 system 含该段（日志或 debug 断言）

## 2. profile skill

- [x] 2.1 新增 `assets/skills/profile/SKILL.md`（init 流程、问题库、schema、禁止非 init 写 AGENTS.md）
- [x] 2.2 `core/skills.rs` 注册 `profile`；`index_markdown()` 含 profile
- [x] 2.3 单测：`skill_read {"skill":"profile"}` 返回非空

## 3. clarify `confirm_agents_md`

- [x] 3.1 Rust：`ClarifyKind` / `clarify_ask` 校验支持 `confirm_agents_md` + `preview_markdown`（`brief` 非必填）
- [x] 3.2 前端 `types.ts`：`ClarifyKind` 扩展；`ClarifyQuestionCard` 全文 Markdown 可滚动预览 + `changelog_summary`
- [x] 3.3 测试：`clarify_ask` 接受/拒绝空 preview；前端卡片渲染 snapshot 或 RTL

## 4. AGENTS.md 写入门禁

- [x] 4.1 turn 元数据：`send_message` 检测 `/init` 前缀设置 `profile_init`
- [x] 4.2 `fs_write` / `fs_patch`：非 `profile_init` turn 拒绝路径 `AGENTS.md`；init turn 允许；8000 字符上限
- [x] 4.3 单测：非 init 写失败、init 写成功

## 5. 能力 B — `/init` 斜杠 command

- [x] 5.1 `slashCommands.ts`：`SlashEntry` 联合类型；注册 `{ kind: "command", id: "init", acceptsTail: true }`
- [x] 5.2 `slash.ts`（或等价）：command Enter → `send_message(原文)`；template 行为不变
- [x] 5.3 前端：clarify pending 时禁止 init（与 workspace-ui spec 一致）
- [x] 5.4 后端 `send_message`：clarify pending + `/init` 前缀 → 错误返回
- [x] 5.5 测试：slash 分叉；pending 时 send 被拒

## 6. init turn 端到端

- [x] 6.1 system 或 profile skill：init turn 指引（读 AGENTS.md、项目扫描、clarify、confirm_agents_md、fs_write、简短摘要）
- [x] 6.2 集成/手动：空项目 `/init` → clarify → confirm → 文件落盘 → 下一 turn 注入生效
- [x] 6.3 集成/手动：有会话史 `/init 固化PPT风格` → 摘要含 PPT 相关变更

## 7. 文档与 backlog

- [x] 7.1 归档前核对 `openspec/changes/project-agent-profile/specs/**` 与实现一致
- [x] 7.2 更新 `openspec/specs/project-backlog/spec.md` BL-006 状态与决策引用
- [x] 7.3 `npm test` + `cargo test` 全绿
