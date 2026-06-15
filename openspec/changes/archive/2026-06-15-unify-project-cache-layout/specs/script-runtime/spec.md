## MODIFIED Requirements

### Requirement: skill_run 临时脚本恢复区

系统 SHALL 使用项目沙箱 `.cache/skill-run/` 作为 `skill_run` 的临时脚本恢复区。该目录 MUST only contain temporary recovery files for the latest run。清理时机分两级：成功且无交付物时立即清理；写出 Office 交付物或存在排版告警时在 turn 内保留供修复，turn 结束时只要不存在未修复的执行失败（`.cache/skill-run/error.json`）就 MUST 自动清理，不依赖 Agent 显式调用。

#### Scenario: inline 脚本执行前保存

- **WHEN** Agent 调用 `skill_run` 并传入 `code`
- **THEN** 系统 SHALL 在执行前将脚本内容写入 `.cache/skill-run/script.js`

#### Scenario: 纯计算脚本成功后立即清理

- **WHEN** `skill_run` 执行成功且未写出 `.docx/.pptx/.xlsx/.xlsm` 交付物、无排版告警
- **THEN** 系统 SHALL 删除 `.cache/skill-run/` 临时目录（若存在）

#### Scenario: 写出交付物后 turn 内保留脚本

- **WHEN** `skill_run` 执行成功且写出 Office 交付物（或返回 `style_warnings`）
- **THEN** 系统 SHALL 保留 `.cache/skill-run/script.js` 供本 turn 内 `fs_patch` 修复与 `path` 重跑
- **AND** 成功响应 SHALL include `script_path` 与修复指引
- **AND** 系统 SHALL 清除上一次失败遗留的 `.cache/skill-run/error.json`

#### Scenario: turn 结束自动清理

- **WHEN** Agent turn 结束（正常完成或达到最大工具步数）
- **AND** `.cache/skill-run/error.json` 不存在（无未修复的脚本失败）
- **THEN** 系统 SHALL 自动删除 `.cache/skill-run/`，无论 `style_warnings` 是否被处理

#### Scenario: 失败现场跨 turn 保留

- **WHEN** `skill_run.code` 执行失败
- **THEN** 系统 SHALL 保留 `.cache/skill-run/script.js` 并写入 `.cache/skill-run/error.json`
- **AND** 工具错误结果 MUST include `script_path` with value `.cache/skill-run/script.js`
- **AND** turn 结束时该目录 MUST NOT 被清理

### Requirement: fs_patch 局部文本修改

系统 SHALL 提供 `fs_patch` 工具，对项目内 UTF-8 文本文件执行精确子串替换，作为修复 `.cache/skill-run/script.js` 等大文件的首选方式（替代 `fs_write` 全量重写）。

#### Scenario: 唯一匹配替换

- **WHEN** Agent 调用 `fs_patch`，每条 edit 的 `old` 在文件中恰好出现一次
- **THEN** 系统 SHALL 应用全部替换并返回 `applied` 计数

#### Scenario: 原子性 — 任一 edit 未命中则全部不应用

- **WHEN** 任一 edit 的 `old` 未找到，或多处匹配且未设 `replace_all`
- **THEN** 系统 MUST NOT 写入任何修改
- **AND** 返回结构化错误，列出每条未命中 edit 的原因（`not found` / `multiple matches`）

#### Scenario: 拒绝无效 edit

- **WHEN** edit 的 `old` 为空字符串，或 `old` 与 `new` 相同
- **THEN** 工具 MUST 返回 invalid arguments 错误
