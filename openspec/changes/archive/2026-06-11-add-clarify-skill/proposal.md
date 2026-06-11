## Why

用户在请求创建文档时往往描述不够完整（缺少受众、结构、风格等关键信息），导致 agent 生成的文档与预期偏差大，需要多次返工。当前 agent 没有内置的需求澄清机制，面对模糊请求会直接猜测生成。

## What Changes

- **新增** `clarify` Skill（编译进 assets/skills/clarify/SKILL.md）：一套针对文档创作的对话式需求澄清流程，指导 agent 在生成前逐步澄清关键信息
- **更新** agent 系统提示词：收到模糊文档创作请求时，MUST 先 `skill_read clarify` 并按流程执行
- **更新** `core/skills.rs`：注册 `clarify` skill，使其可通过 `skill_read` 工具获取
- **更新** `tools/skill.rs` 的工具描述：将 `clarify` 加入 skill 枚举说明

## Capabilities

### New Capabilities

- `clarify-skill`：doc-agent 内置需求澄清能力，包含触发条件、逐问对话流程、按文档类型分组的问题库（内容、结构、排版/样式）、深度控制规则，以及最终输出结构化「创作简报」驱动后续生成

### Modified Capabilities

- `document-skills`：skill_read 可用 skill 列表新增 `clarify`，描述与触发条件更新

## Impact

- `src-tauri/assets/skills/clarify/SKILL.md`：新增文件（编译资产）
- `src-tauri/src/core/skills.rs`：新增 clarify skill 注册
- `src-tauri/src/tools/skill.rs`：更新工具描述枚举
- `src-tauri/src/agent/loop_runner.rs`：系统提示词追加 clarify 触发说明
- 无新外部依赖，无 breaking change
