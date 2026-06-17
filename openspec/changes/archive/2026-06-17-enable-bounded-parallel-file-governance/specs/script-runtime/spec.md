## MODIFIED Requirements

### Requirement: skill_run 临时脚本恢复区

系统 SHALL 使用项目沙箱 `.cache/skill-run/<session_key>/` 作为 `skill_run` 的临时脚本恢复区（`session_key = hash(session_id)`，同一会话内各 turn 共用同一路径；段名为 8 位 hex）。该目录 MUST only contain temporary recovery files for the owning session。清理时机分两级：成功且无交付物时可立即清理；写出 Office 交付物或存在排版告警时在 turn 内保留供修复，turn 结束时只要不存在未修复的执行失败（`error.json`）就 MUST 删除整个 session scratch 目录，不依赖 Agent 显式调用。

#### Scenario: inline 脚本执行前保存

- **WHEN** Agent 调用 `skill_run` 并传入 `code`
- **THEN** 系统 SHALL 在执行前将脚本内容写入 `.cache/skill-run/<session_key>/script.js`

#### Scenario: 两个会话脚本路径不同

- **WHEN** session A 与 session B 同时调用 `skill_run` inline code
- **THEN** A 与 B 的 `script_path` MUST 不同
- **AND** 任一 session 清理时 MUST NOT 删除另一 session 的 scratch 目录

#### Scenario: 同 session 跨 turn 共用 script_path

- **WHEN** session A 在 turn 1 的 `skill_run.code` 执行失败
- **AND** 用户在 turn 2 继续修复
- **THEN** turn 2 的 `script_path` MUST 与 turn 1 相同
- **AND** Agent 可用 `fs_patch` + `skill_run {"path":"<script_path>"}` 重跑

#### Scenario: path 重跑使用返回 script_path

- **WHEN** `skill_run.code` 执行失败
- **THEN** 工具错误结果 MUST include session-scoped `script_path`
- **AND** Agent 可用 `skill_run {"path":"<script_path>"}` 重跑该脚本

#### Scenario: 失败现场跨 turn 保留

- **WHEN** `skill_run.code` 执行失败
- **THEN** 系统 SHALL 保留该 session 的 `script.js` 并写入同目录 `error.json`
- **AND** turn 结束时该 session scratch 目录 MUST NOT 被清理

### Requirement: 运行时沙箱

运行时 MUST 禁用网络访问；文件读写 MUST 仅通过宿主注入的自定义 op 进行，且每次访问经现有 `Sandbox` 路径校验；单次执行 MUST 有超时上限（默认 30 秒），超时即终止并返回错误。写入类 op 在落盘前 MUST 通过文件锁系统申请目标路径 Write lock，冲突时 MUST 返回错误且不写入磁盘。

#### Scenario: 越界写被拒

- **WHEN** 脚本尝试写入项目根目录之外的路径
- **THEN** op 返回沙箱错误，文件未被创建

#### Scenario: 写锁冲突被拒

- **WHEN** 脚本尝试写入已被其他 session 占用的 `out.pptx`
- **THEN** op 返回 file busy 错误，`out.pptx` 不被覆盖

#### Scenario: 网络不可用

- **WHEN** 脚本尝试发起 fetch 请求
- **THEN** 运行时报错（无网络扩展），执行不挂起

### Requirement: 运行时目录与存在性 op

系统 SHALL 在 `skill_run` 嵌入式运行时提供沙箱内的目录 listing 与路径存在性检查，经 Native op 实现且每次访问 MUST 经过 `Sandbox` 路径校验。Agent SHOULD 使用工具返回的 OOXML `out_dir` 调用 `doc_list` / `fs.readdirSync`，MUST NOT 假设固定 `unpacked/` 存在。

#### Scenario: doc_list 列出返回的 OOXML 目录

- **WHEN** `ooxml_unpack` 返回 `.cache/ooxml/a782729d/4b2c025c`
- **AND** 脚本调用 `doc_list(".cache/ooxml/a782729d/4b2c025c/ppt/slides")`
- **THEN** 返回 slide 文件列表

#### Scenario: fs polyfill 映射

- **WHEN** 脚本调用 `fs.existsSync(path)` 或 `fs.readdirSync(path)`
- **THEN** 行为分别等价于 `doc_exists(path)` 与 `doc_list(path)` 的名称数组
