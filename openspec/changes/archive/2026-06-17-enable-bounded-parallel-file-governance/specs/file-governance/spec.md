## ADDED Requirements

### Requirement: 全局最多 3 个 running turns

系统 SHALL 在应用进程内限制全局 running turn 数量最多为 3。该限制跨所有 project 与 session 生效。当已有 3 个 turn 处于 running 或 stopping 状态时，新的 `send_message` 或 `resume_turn` MUST 被拒绝，并返回可读错误「当前已有 3 个任务正在执行，请稍后重试。」拒绝 MUST 发生在持久化新 user message 或提交 clarify answer 之前。

#### Scenario: 第四个 turn 被拒绝

- **WHEN** 应用内已有 3 个 session 正在 running
- **AND** 用户在第 4 个 session 发送消息
- **THEN** `send_message` 返回全局并行已满错误
- **AND** 第 4 个 session 不新增 user message

#### Scenario: 跨 project 计入同一上限

- **WHEN** project A 有 2 个 running turns，project B 有 1 个 running turn
- **THEN** 任意 project 的新 turn 都 MUST 被拒绝，直到其中一个 turn 结束

#### Scenario: clarify 等待不占 running 名额

- **WHEN** session A emit `turn_awaiting_user` 并等待用户回答
- **THEN** A 不计入 3 个 running turns

#### Scenario: stopping 期间仍占 running 名额

- **WHEN** session A 正在 running
- **AND** 用户对其发起 stop，前端进入 stopping
- **AND** 后端尚未 emit `turn_cancelled`
- **THEN** A 仍计入 3 个 running turns
- **AND** 若已有 3 个此类 slot，第 4 个 `send_message` MUST 被拒绝

### Requirement: PDF 渲染缓存与文件锁分工

系统 SHALL 对 `pdf_read` 与 `pdf_render_pages` 的源 PDF 路径通过 `FileLockRegistry` 申请 `Read` lock。`.cache/pdf/<cache_key>/` 目录的创建与更新 MUST 继续由 `pdf_cache::with_render_lock` 保护，MUST NOT 纳入 `FileLockRegistry` 或 `ToolIoPlan` 的 write/subtree 锁。

#### Scenario: 同源 PDF 并行读与 cache miss

- **WHEN** 同一 project 中 session A 与 B 以相同参数对同一 PDF 发起需渲染的 `pdf_read`
- **AND** 两者均为 cache miss
- **THEN** 两者均可持有源文件 Read lock
- **AND** miss 渲染由 `with_render_lock` 串行化，cache 目录不被损坏

#### Scenario: 读 PDF 与写同一 PDF 冲突

- **WHEN** session A 正在读取 `report.pdf`
- **AND** session B 尝试写入 `report.pdf`
- **THEN** B 的工具调用 MUST 因文件锁冲突失败

### Requirement: 同 session 单 active turn

系统 SHALL 保持同一 `session_id` 同时最多一个 active 或 reserved turn。即使全局 running 未满，同一 session 的重复 `send_message` 或 `resume_turn` MUST 被拒绝，且不得写入新 user message。

#### Scenario: 同 session 重复发送被拒

- **WHEN** session A 正在 running
- **AND** 用户再次向 session A 发送消息
- **THEN** 后端拒绝该请求并提示当前会话正在执行任务

### Requirement: Project 文件锁

系统 SHALL 在每个 project 内维护内存文件锁。文件锁 MUST 以 sandbox 校验后的 project-relative POSIX 路径为 key，并支持：

- `Read`：读取文件或目录，可与其他 `Read` 共存
- `Write`：写入单个文件，独占同一路径
- `SubtreeWrite`：删除、重建或批量写目录，独占该目录及所有 descendants

不同 project 的同名路径 MUST NOT 互相冲突。同 project 内同一路径或 ancestor/descendant 关系上的写冲突 MUST 拒绝后者。

#### Scenario: 不同文件可并行写

- **WHEN** 同一 project 中 session A 写 `a.docx`
- **AND** session B 写 `b.docx`
- **THEN** 两个工具调用可并行执行

#### Scenario: 同一文件写冲突拒绝后者

- **WHEN** session A 已持有 `report.docx` 的 Write lock
- **AND** session B 也尝试写 `report.docx`
- **THEN** session B 的工具调用 MUST 失败
- **AND** 错误包含「当前 report.docx 已被会话」

#### Scenario: 目录写与子文件写冲突

- **WHEN** session A 正在 `SubtreeWrite` `unpacked/`
- **AND** session B 尝试写 `unpacked/word/document.xml`
- **THEN** session B MUST 被拒绝

#### Scenario: 跨 project 同名文件不冲突

- **WHEN** project A 的 session 写 `report.docx`
- **AND** project B 的 session 写 `report.docx`
- **THEN** 两者不因路径同名冲突

### Requirement: 工具执行前 IO 规划

系统 SHALL 在每个工具 handler 执行前，根据 tool name 与 arguments 生成 `ToolIoPlan`，并在申请所需文件锁成功后才执行工具。`tool_result.changed_paths` MUST NOT 作为并发准入依据，因为它只在工具成功后产生。

#### Scenario: 写工具执行前加锁

- **WHEN** Agent 调用 `fs_write {"path":"notes.md"}`
- **THEN** 系统在写入前申请 `notes.md` 的 Write lock

#### Scenario: 冲突作为工具失败回填

- **WHEN** 工具调用因文件锁冲突被拒绝
- **THEN** Agent loop MUST 持久化失败 tool result
- **AND** 继续保持消息序列合法

### Requirement: 动态写入兜底锁

系统 SHALL 对无法完全静态推导写路径的工具提供动态写入锁兜底。`skill_run` runtime 中的 `doc_write`、`doc_write_bytes`、`fs.writeFileSync`、ExcelJS/PptxGenJS 写文件 shim 在实际写磁盘前 MUST 申请目标路径 Write lock；申请失败 MUST 阻止写入并返回结构化错误。

#### Scenario: skill_run 动态写同一文件被拒

- **WHEN** session A 的脚本正在写 `out.xlsx`
- **AND** session B 的脚本也尝试写 `out.xlsx`
- **THEN** session B 的写入 MUST 在落盘前失败

### Requirement: 锁生命周期

系统 SHALL 在工具调用结束后释放该工具调用持有的文件锁；turn 结束、取消、错误、达到最大步数或 clarify awaiting 时 MUST 释放该 turn 的全局 running slot 与所有残留 locks。锁释放 MUST 幂等。

#### Scenario: cancel 后释放文件锁

- **WHEN** session A 写文件过程中被 stop 并最终 emit `turn_cancelled`
- **THEN** 后续 session B 可写同一文件，不再被 A 的锁阻塞
