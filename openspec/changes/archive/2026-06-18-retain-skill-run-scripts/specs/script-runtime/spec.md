## MODIFIED Requirements

### Requirement: skill_run 临时脚本恢复区

系统 SHALL 使用项目沙箱 `.cache/skill-run/<session_key>/` 作为 `skill_run` 的临时脚本恢复区（`session_key = hash(session_id)`，同一会话内各 turn 共用同一路径；段名为 8 位 hex）。该目录 MUST only contain recovery files for the owning session。`script.js` 在成功执行后 MUST 保留，供同 session 跨 turn 读取与 `fs_patch` 修复；新的 inline `code` 执行前 MUST 覆盖写入。`error.json` 仅在执行失败时写入；成功执行（含 path 重跑修复成功）后 MUST 删除。整个 session scratch 目录 MUST 仅在用户 cancel turn 时删除；turn 正常结束或达到 max tool steps MUST NOT 删除 `script.js`。

#### Scenario: inline 脚本执行前保存

- **WHEN** Agent 调用 `skill_run` 并传入 `code`
- **THEN** 系统 SHALL 在执行前将脚本内容写入 `.cache/skill-run/<session_key>/script.js`

#### Scenario: 两个会话脚本路径不同

- **WHEN** session A 与 session B 同时调用 `skill_run` inline code
- **THEN** A 与 B 的 `script_path` MUST 不同
- **AND** 任一 session 清理时 MUST NOT 删除另一 session 的 scratch 目录

#### Scenario: 同 session 跨 turn 共用 script_path

- **WHEN** session A 在 turn 1 成功执行 `skill_run` 写出交付物
- **AND** 用户在 turn 2 要求修改该交付物
- **THEN** turn 2 的 `script_path` MUST 与 turn 1 相同
- **AND** `script.js` MUST 仍存在于磁盘
- **AND** Agent 可用 `fs_patch` + `skill_run {"path":"<script_path>"}` 重跑

#### Scenario: path 重跑使用返回 script_path

- **WHEN** `skill_run.code` 执行失败
- **THEN** 工具错误结果 MUST include session-scoped `script_path`
- **AND** Agent 可用 `skill_run {"path":"<script_path>"}` 重跑该脚本

#### Scenario: 失败现场跨 turn 保留

- **WHEN** `skill_run.code` 执行失败
- **THEN** 系统 SHALL 保留该 session 的 `script.js` 并写入同目录 `error.json`
- **AND** turn 结束时 scratch 目录 MUST NOT 被清理

#### Scenario: 成功执行保留 script.js

- **WHEN** `skill_run` 成功执行（含纯计算脚本、无 Office 交付物）
- **THEN** `.cache/skill-run/<session_key>/script.js` MUST 保留
- **AND** 若存在 `error.json` MUST 被删除

#### Scenario: cancel turn 清理 scratch

- **WHEN** 用户 cancel 进行中的 turn
- **THEN** 系统 SHALL 删除该 session 的 `.cache/skill-run/<session_key>/` 目录（若存在）
