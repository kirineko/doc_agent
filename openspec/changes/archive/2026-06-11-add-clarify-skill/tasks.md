## 1. 新增 clarify skill 资产

- [x] 1.1 创建 `src-tauri/assets/skills/clarify/SKILL.md`，内容包含：触发条件、流程定义（逐问对话 + 深度控制规则）、Word 问题库（内容/结构/排版样式）、PPT 问题库（内容/结构/排版样式）、报告问题库（内容/结构/排版样式）、创作简报输出格式

## 2. 注册 clarify skill

- [x] 2.1 在 `src-tauri/src/core/skills.rs` 中添加 `CLARIFY_DOCS` 常量（`include_str!` 引入 SKILL.md）并在 `SKILLS` 数组追加 clarify 条目（name: `"clarify"`，description: 中文描述）
- [x] 2.2 更新 `src-tauri/src/tools/skill.rs` 中 `skill_read` 工具的 description，将 `clarify` 加入 skill 枚举说明

## 3. 更新系统提示词

- [x] 3.1 在 `src-tauri/src/agent/loop_runner.rs` 的 `build_working_messages` 函数中，追加 clarify 触发指示：收到全新文档创作且需求不完整（缺少主题/受众/结构/风格中 ≥ 2 项）时，MUST 先 `skill_read clarify`

## 4. 验证

- [x] 4.1 执行 `cargo test`，确认现有 skill 相关测试通过，clarify 出现在 `available_names()` 返回列表中
- [x] 4.2 本地运行 app，发送模糊文档创作请求，确认 agent 触发 `skill_read clarify` 并开始逐问澄清
- [x] 4.3 发送已包含充足信息的创作请求，确认 agent 进入极简路径或直接跳过澄清
