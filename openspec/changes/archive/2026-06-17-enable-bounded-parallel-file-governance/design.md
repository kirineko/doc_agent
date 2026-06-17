## Context

当前实现已经有这些基础：

- `TurnRegistry` 记录 active turn，并提供 cancel signal、reserved resume、session running 查询
- 前端 `sessionRunState.ts` 已按 session 维护 `idle | running | stopping`，可展示后台会话状态
- `loop_tool_batch.rs` 对连续 `pdf_read` 做批内并发，说明工具执行层已经能处理有限并发
- `pdf_cache.rs` 已有按 cache key 的 render lock，证明 cache 写入需要互斥

当前缺口也很明确：

- `TurnRegistry` 在 `check_turn_can_start` 中拒绝同 project 其他 active turn，导致同 project 不同文件任务无法并行
- `skill_run` 恢复区是 project 级固定路径，多个 turn 并发会覆盖脚本、误清理错误现场
- `ooxml_unpack` 的 `out_dir` 由模型提供，示例固定 `unpacked/`，且存在时先 `remove_dir_all`
- 文件写冲突没有执行前准入；`changed_paths` 是成功后的 UI 刷新信号，不能用于并发保护
- `skill_run` 动态写入路径只有 runtime 执行时才知道，静态 IO plan 只能覆盖一部分

## Goals / Non-Goals

### Goals

- 全局最多 3 个 running turn，跨 project 与同 project 共用限额
- 同一 session 同时仍最多 1 个 active/reserved turn
- 同 project 不同文件任务可并行
- 同 project 中任何两个 running turns 只要会写同一文件、同一目录子树或同一系统工作区，就拒绝后者
- 系统临时工作区必须由后端主动隔离，避免模型生成的固定目录导致冲突
- 所有文件写入路径在执行前尽量静态申请锁；`skill_run` runtime 动态写入必须在写入前兜底申请锁
- cancel、clarify awaiting、tool error、provider error、max steps、turn_complete 都必须释放全局 slot 与文件 locks

### Non-Goals

- 不提供排队；拒绝后者即可
- 不持久化 running/lock 状态到 DB；应用重启后锁自然消失
- 不做跨进程锁；用户用 Finder/Office 打开同一文件不在本变更范围
- 不自动重命名用户指定输出；同名输出是产品层冲突，应明确拒绝

## Decisions

### D1: 分离全局并发限制与文件锁

新增两个概念：

- `RunLimiter`：只负责全局 running turn 数量，最多 3
- `FileLockRegistry`：只负责 project 内文件资源冲突

`TurnRegistry` 保留 turn 生命周期、cancel、reserved resume 与同 session 互斥，不再承担同 project 串行策略。

推荐文件：

```text
src-tauri/src/agent/turn_control.rs      # 保留 TurnRegistry，移除 project_busy 检查
src-tauri/src/core/file_locks.rs         # 新增 FileLockRegistry
src-tauri/src/state.rs                   # AppState 增加 file_locks 与 run_limiter
```

参考代码：

```rust
pub const MAX_GLOBAL_RUNNING_TURNS: usize = 3;

#[derive(Clone, Default)]
pub struct RunLimiter {
    inner: Arc<Mutex<HashMap<String, ActiveRunSlot>>>, // session_id -> slot
}

#[derive(Clone, Debug)]
pub struct ActiveRunSlot {
    pub session_id: String,
    pub turn_id: String,
    pub project_id: String,
}

pub struct RunSlotGuard {
    limiter: RunLimiter,
    session_id: String,
}

impl RunLimiter {
    pub fn acquire(
        &self,
        session_id: String,
        turn_id: String,
        project_id: String,
    ) -> Result<RunSlotGuard, String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        if guard.contains_key(&session_id) {
            return Err("当前会话正在执行任务，请等待完成或先停止。".into());
        }
        if guard.len() >= MAX_GLOBAL_RUNNING_TURNS {
            return Err("当前已有 3 个任务正在执行，请稍后重试。".into());
        }
        guard.insert(session_id.clone(), ActiveRunSlot { session_id: session_id.clone(), turn_id, project_id });
        Ok(RunSlotGuard { limiter: self.clone(), session_id })
    }
}
```

`RunSlotGuard` 必须 RAII drop 释放；显式 terminal path 也可调用 `release`，但 drop 是兜底。

**与 `ActiveTurnGuard` 同寿**：`RunSlotGuard` 在 `register_active_turn` 成功后与 `ActiveTurnGuard` 一并创建，在 agent loop 完全退出（`turn_complete` / `turn_cancelled` / `turn_awaiting_user` / terminal error）时随 guard drop 释放。`cancel_turn` IPC 仅设置 `CancelSignal`，**不得**提前 release slot。

**stopping 占 slot**：`stopping` 是前端过渡态（`markSessionStopping` → 等待 `turn_cancelled`）。后端在该 session 的 loop 未退出前，slot 与 `TurnRegistry` active 条目持续有效。这与 `turn_awaiting_user` 不同——后者 loop 已退出，guard 已 drop，**立即** release slot。

| 阶段 | 前端 | RunLimiter | TurnRegistry |
|------|------|------------|--------------|
| turn 运行中 | `running` | 占 slot | active |
| 用户点停止 | `stopping` | **仍占 slot** | **仍 active** |
| terminal event | `idle` | release | unregister |

前端 `runningSessionCount` 可派生 `running + stopping` 做发送预判；权威计数以 `RunLimiter::occupied_count()` 为准。前端 stopping 超时（`forceSessionIdle`）只清 UI，**不得**假设后端已 release slot。

### D2: FileLockRegistry 使用 project-relative 规范化资源

资源 key 必须是 `(project_id, resource_path, scope)`，路径是 POSIX 风格、无 `.` / `..`、经 `Sandbox` 解析后确认在项目内。

锁模式：

| 模式 | 语义 | 并发规则 |
|------|------|----------|
| `Read` | 读取文件或目录 | 可与 Read 共存；与 Write/SubtreeWrite 冲突 |
| `Write` | 写入单个文件 | 独占该文件；与同路径 Read/Write 冲突 |
| `SubtreeWrite` | 删除/重建/批量写目录 | 独占该目录及 descendants；与 ancestor/descendant Read/Write/SubtreeWrite 冲突 |
| `Workspace` | 系统 scratch 工作区（skill-run 按 session、ooxml 按 work_key） | 默认不冲突，因路径含 session/work 段；若显式共享则按 SubtreeWrite |

冲突检查必须考虑祖先/后代：

- 写 `docs/report.docx` 与读/写 `docs/report.docx` 冲突
- `SubtreeWrite("unpacked")` 与 `Write("unpacked/word/document.xml")` 冲突
- `SubtreeWrite(".cache/ooxml/a782729d/4b2c025c")` 与 `.cache/ooxml/b913840e/c5d3e136` 不冲突
- project 不同则不冲突

参考代码：

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockMode {
    Read,
    Write,
    SubtreeWrite,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileResource {
    pub project_id: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct LockRequest {
    pub resource: FileResource,
    pub mode: LockMode,
}

#[derive(Clone, Debug)]
struct HeldLock {
    request: LockRequest,
    session_id: String,
    turn_id: String,
    session_title: String,
}

fn conflicts(a: &HeldLock, b: &LockRequest) -> bool {
    if a.request.resource.project_id != b.resource.project_id {
        return false;
    }
    let ap = a.request.resource.path.as_str();
    let bp = b.resource.path.as_str();
    let same = ap == bp;
    let ancestor = is_ancestor_or_same(ap, bp) || is_ancestor_or_same(bp, ap);
    match (a.request.mode, b.mode) {
        (LockMode::Read, LockMode::Read) => false,
        (LockMode::Read, LockMode::Write) | (LockMode::Write, LockMode::Read) => same,
        (LockMode::Write, LockMode::Write) => same,
        (LockMode::SubtreeWrite, _) | (_, LockMode::SubtreeWrite) => ancestor,
    }
}

fn is_ancestor_or_same(parent: &str, child: &str) -> bool {
    parent == child || child.strip_prefix(parent).is_some_and(|rest| rest.starts_with('/'))
}
```

错误文案：

```text
当前 {path} 已被会话「{session_title}」占用，请稍后重试。
```

如果没有标题，用 session id 截断展示：

```text
当前 {path} 已被会话 {session_id} 占用，请稍后重试。
```

### D3: ToolIoPlan 是执行前准入，不替代 changed_paths

新增：

```text
src-tauri/src/tools/io_plan.rs
```

职责：

- 根据 tool name + args 推导执行前 read/write/subtree resources
- 不执行工具、不访问外网、不读取大文件
- 使用 `Sandbox` 校验路径并转成 project-relative normalized path
- 返回 `ToolIoPlan { locks, dynamic_writes }`

`changed_paths` 继续只做 UI 文件列表刷新，不能参与准入。

参考结构：

```rust
pub struct ToolIoPlan {
    pub locks: Vec<LockRequest>,
    pub dynamic_writes: bool,
}

pub fn plan_tool_io(
    project_id: &str,
    sandbox: &Sandbox,
    tool_name: &str,
    args: &Value,
) -> Result<ToolIoPlan, ToolError> {
    let mut plan = ToolIoPlan { locks: Vec::new(), dynamic_writes: false };
    match tool_name {
        "fs_read" | "office_read_to_markdown" | "excel_read" | "pdf_read" | "pdf_render_pages" | "image_read" => {
            add_read_paths(project_id, sandbox, args, &mut plan)?;
        }
        "fs_write" | "fs_patch" | "excel_write" | "xlsx_recalc" => {
            add_write_arg(project_id, sandbox, args, "path", &mut plan)?;
        }
        "ooxml_unpack" => {
            add_read_arg(project_id, sandbox, args, "path", &mut plan)?;
            add_subtree_write_arg_or_generated(project_id, sandbox, args, "out_dir", &mut plan)?;
        }
        "ooxml_pack" => {
            add_read_arg(project_id, sandbox, args, "dir", &mut plan)?;
            add_write_arg(project_id, sandbox, args, "out_path", &mut plan)?;
            add_optional_read_arg(project_id, sandbox, args, "original", &mut plan)?;
        }
        "docx_comment" => {
            add_subtree_write_arg(project_id, sandbox, args, "dir", &mut plan)?;
        }
        "docx_accept_changes" => {
            add_read_arg(project_id, sandbox, args, "path", &mut plan)?;
            add_write_arg_or_path(project_id, sandbox, args, "out_path", "path", &mut plan)?;
        }
        "skill_run" => {
            plan.dynamic_writes = true;
            add_skill_run_script_locks(project_id, sandbox, args, &mut plan)?;
        }
        _ => {}
    }
    Ok(plan)
}
```

每个工具的策略见任务 3。

### D4: 工具执行时持有短生命周期 locks，turn 持有 workspace locks

工具级锁申请和释放应围绕单次工具调用：

1. LLM 返回 tool call
2. parse args
3. build `ToolIoPlan`
4. acquire locks
5. execute tool
6. persist tool result
7. release locks

例外：

- `skill_run` inline script recovery dir：按 session 隔离（同会话跨 turn 共用路径）；失败现场可跨 turn 保留；workspace lock 为 `SubtreeWrite(.cache/skill-run/<session_key>/)`
- `ooxml_unpack` 自动生成的 `.cache/ooxml/<session_key>/<work_key>/` 不与其他 session/turn 冲突，后续同 turn 的 `fs_patch` / `ooxml_pack` 会按返回路径正常加锁

`execute_one` 参考代码：

```rust
let io_plan = plan_tool_io(project_id, ctx.sandbox, &plan.call.function.name, &plan.args)?;
let lock_guard = state.file_locks.acquire_many(
    session_id,
    turn_id,
    session_title,
    io_plan.locks,
)?;
let outcome = state.tools.execute(ctx.with_dynamic_locks(io_plan.dynamic_writes), ...).await;
drop(lock_guard);
```

锁失败时返回 tool error，不应 crash loop：

```json
{
  "error": "file_busy",
  "message": "当前 docs/report.docx 已被会话「合同审阅」占用，请稍后重试。",
  "path": "docs/report.docx",
  "blocking_session_id": "..."
}
```

Agent 得到该 tool result 后可解释给用户；对于 `send_message` 启动前的全局 3 满额，仍由 IPC 直接拒绝，不写 user message。

### D5: `skill_run` 恢复区按 session 隔离

现状：

```text
.cache/skill-run/script.js
.cache/skill-run/error.json
```

目标：

```text
.cache/skill-run/<session_key>/script.js
.cache/skill-run/<session_key>/error.json
```

其中 `session_key = cache_key([session_id])`；同一会话内各 turn 共用同一路径。

```rust
pub fn skill_run_dir(session_id: &str) -> String {
    format!("{SKILL_RUN_DIR}/{}", cache_key(&[session_id]))
}

pub fn skill_run_script(session_id: &str) -> String {
    format!("{}/script.js", skill_run_dir(session_id))
}

pub fn skill_run_error(session_id: &str) -> String {
    format!("{}/error.json", skill_run_dir(session_id))
}
```

`ToolContext` 必须带 `session_id` 与 `turn_id`，否则 `skill_run` 不得使用 inline code 写恢复区。测试里的 `ToolContext::new` 可提供 `with_test_turn("test-session", "test-turn")` helper。

兼容策略：

- 新 inline code 错误返回 `script_path` 为 session-scoped path
- `skill_run {"path": "<returned script_path>"}` 可重跑
- 旧固定 `.cache/skill-run/script.js` 不再由系统生成；无需兼容读取
- turn 结束无 `error.json` 时删除整个 `<session_key>/` scratch 目录；有失败现场则保留至修复

### D6: `skill_run` runtime 动态写入要兜底锁

静态 IO plan 无法知道 JS 内部 `doc_write("out.docx", ...)` 或 `fs.writeFileSync(xmlPath, ...)` 的所有路径。必须在 runtime op 写入前申请锁。

推荐新增 runtime write delegate：

```rust
pub trait RuntimeWriteGate: Send + Sync {
    fn before_write(&self, path: &str) -> Result<RuntimeWritePermit, String>;
}

pub struct RuntimeWritePermit {
    _guard: FileLockGuard,
}
```

`ops::register_write` 从 capture 中拿 `Arc<dyn RuntimeWriteGate>`：

```rust
let permit = write_gate.before_write(&path).map_err(|e| {
    boa_engine::error::JsNativeError::typ().with_message(e)
})?;
std::fs::write(&resolved, bytes)?;
drop(permit);
```

为了避免同一个脚本连续多次写同一文件重复申请/释放，可以在 `execute_script` 生命周期里保留 `RuntimeLockSet`：

- 第一次写 `out.docx` 申请 Write lock
- 同脚本后续写同路径复用
- 脚本结束后统一释放

动态写冲突也必须返回结构化 `skill_run` 错误，保留脚本现场，便于 Agent 和用户理解。

### D7: `ooxml_unpack` 默认走 `.cache/ooxml`

现状模型会固定 `unpacked/`，且工具会删除已有目录。目标是让后端主动生成隔离目录。

接口变化：

```json
{
  "path": "template.docx",
  "out_dir": ".cache/ooxml/<session_key>/<work_key>"
}
```

`out_dir` 变为可选：

- 未传：后端生成 `.cache/ooxml/<session_key>/<work_key>/`（`work_key = cache_key([session_id, turn_id, source_path])`；路径由 session+turn+source 确定，**不**嵌入文件名 stem，便于 io_plan 与 handler 取得一致锁路径）
  - 该生成目录已存在时（同一轮对同一文档重复解包）必须**拒绝**而非静默 `remove_dir_all`，引导复用已返回的 `out_dir`，避免删除本轮已编辑的 XML
- 已传：按当前语义使用，但必须申请 `SubtreeWrite(out_dir)`；锁经 `TurnFileLockStore` **跨同 turn 内多次工具调用持有**，直到 turn 结束（`turn_complete` / `turn_cancelled` / `turn_awaiting_user` / terminal error）才释放；`turn_awaiting_user`（clarify 暂停）期间不持有文件锁
- `unpack::unpack` 不应在未持有锁时删除目录；删除目录前必须已经拿到 subtree write lock

返回值必须包含相对路径，不再只返回绝对 display：

```json
{
  "out_dir": ".cache/ooxml/a782729d/4b2c025c",
  "parts": 42
}
```

docx/pptx editing skill 必须改成：

1. 调用 `ooxml_unpack {"path": "template.docx"}`
2. 从返回 `out_dir` 拼接 `word/document.xml` 或 `ppt/slides/...`
3. 后续 `fs_read` / `fs_patch` / `skill_run` 使用返回路径
4. `ooxml_pack {"dir": "<returned out_dir>", "out_path": "output.docx", "original": "template.docx"}`

### D8: 用户可见文件与系统工作区的边界

`.cache/ooxml` 是系统工作区，应隐藏于文件浏览和 `@` 候选。当前 `.cache/` 已隐藏，满足 UI 层。

用户可见 OOXML 解包目录（显式 `out_dir: "contract_unpacked"`）仍允许，但：

- 视为用户可见产物
- 需要 `SubtreeWrite("contract_unpacked")`
- 不自动清理
- 仍被 flat `@` 清单按 existing rule 忽略 `_unpacked`，但文件浏览单层可见

### D9: 全局 3 并行启动时机

`send_message` 当前会先 ensure/create session，再 invoke `send_message`。后端必须在写 user message 前完成：

1. session busy 检查
2. clarify pending 检查
3. global run slot 检查

只有通过后才写入 user message。否则用户点击发送被拒时，不会产生“已发送但未执行”的孤儿消息。

`resume_turn` / clarify submit:

- submit answer 已进入恢复流程前，也要申请 global slot
- 若全局已满，submit 应失败并保留 pending clarify，不写答案
- 若文件锁冲突发生在后续工具执行，作为 tool result 返回；clarify answer 本身已经是用户输入，应保留

**stopping 与 global slot**（见 D1）：第 4 个 `send_message` 的拒绝条件为 `RunLimiter` 已有 3 个未 release 的 slot，其中**包含**用户已点停止但尚未 emit `turn_cancelled` 的 session。实现时勿在 `cancel_turn` 中 release slot。

### D10: PDF 源文件与渲染缓存的分工

`pdf_read` / `pdf_render_pages` 涉及两类资源，由**两层机制**各管一层，不叠加 `FileLockRegistry` 锁 cache 目录：

| 资源 | 机制 | io_plan |
|------|------|---------|
| 用户 PDF 源文件（如 `docs/a.pdf`） | `FileLockRegistry` | `Read` on `path` |
| `.cache/pdf/<cache_key>/` 渲染产物 | 现有 `pdf_cache::with_render_lock` | **不纳入** io_plan |

**为何 cache 不加 file lock**

- `with_render_lock(cache_key)` 已在 miss 渲染临界区做 double-check（锁外/锁内各 `try_cache_hit`），专门防止并行 miss 损坏同一 cache 目录。
- `cache_key` 由 `(rel_path, size, mtime, dpi, pages_spec)` 推导，同 project 多 session 读同一 PDF 应**共享缓存**；render_lock 串行化 miss 即可，无需 turn-scoped 隔离。
- 在 io_plan 阶段为 cache 申请 `SubtreeWrite` 需提前读 metadata 算 key，与 render_lock 职责重复，且可能错误阻塞 cache hit 路径。

**并发行为**

- 两 session 同参数读同一 PDF、均 cache miss → render_lock 串行渲染；源文件双 `Read`，允许。
- 一 miss 渲染、另一 cache hit → hit 路径不进 render_lock，可与 miss 并行。
- 一读 PDF、另一写同一 PDF → 源 `Read` vs `Write` → file_busy。
- 不同 PDF → 各自源 `Read`；cache key 不同，render 可并行。

`io_plan` 示例：

```rust
"pdf_read" | "pdf_render_pages" => {
    add_read_arg(project_id, sandbox, args, "path", &mut plan)?;
    // .cache/pdf/** 不加锁；render_pages_cached 内部仍调用 with_render_lock
}
```

**非 MVP 优化**：若实测跨 project 因相同 `cache_key` 字符串误串行 render_lock，可将 lock key 改为 `{project_id}:{cache_key}`；当前全局 key 已安全，最多多余等待。

### D11: 前端只做提示，不做安全来源

前端可显示 global running count 和文件占用错误，但所有安全规则必须在 Rust 后端强制。

前端需要处理：

- 全局满额 IPC error：`当前已有 3 个任务正在执行，请稍后重试。`
- 文件占用 tool result / error event：显示 toast 或消息内错误
- 后台 session terminal event：即使不是 active session，也要刷新其 session list ordering/title 和 project file list

## Tool IO Plan Matrix

| Tool | Read locks | Write/Subtree locks | Notes |
|------|------------|---------------------|-------|
| `fs_list` | none | none | 仅枚举目录条目；不加锁，避免根级 Read 与任意 SubtreeWrite 冲突而阻塞并行 |
| `fs_read` | path Read | none | text read |
| `fs_search` | project root Read? | none | no write; can skip lock or use project read token |
| `fs_write` | none | path Write | create parent allowed |
| `fs_patch` | path Write | path Write | write lock covers read-modify-write |
| `office_read_to_markdown` | path Read | none | includes PDF text read |
| `office_convert` | path Read | out_path Write | default `*-converted.ext` must be planned before execute |
| `excel_read` | path Read | none | |
| `excel_write` | path Write | path Write | read-modify-write existing workbook |
| `xlsx_recalc` | path Write | path Write | in-place write |
| `ooxml_unpack` | path Read | out_dir SubtreeWrite | generated out_dir if omitted |
| `ooxml_pack` | dir SubtreeWrite, original Read | out_path Write | dir SubtreeWrite 防止打包期间并发写子文件；dir 多为 `.cache/ooxml/...` |
| `docx_comment` | dir SubtreeWrite | dir SubtreeWrite | modifies unpacked dir |
| `docx_accept_changes` | path Read | out_path/path Write | in-place if no out_path |
| `docx_extract_table` | path Read | out_dir SubtreeWrite | writes CSVs |
| `excel_describe` | path Read | none | |
| `excel_normalize` | path Read | out_path Write | |
| `data_query` | sources Read | out_path Write if provided/default | default `query_result.csv` must lock |
| `pdf_merge` | inputs Read | out_path Write | |
| `pdf_split` | path Read | out_path Write or out_dir SubtreeWrite | burst uses out_dir |
| `pdf_rotate` | path Read | out_path Write | |
| `pdf_delete_pages` | path Read | out_path Write | |
| `pdf_render_pages` | path Read | none | `.cache/pdf/<key>` 仅 `with_render_lock`；见 D10 |
| `pdf_read` | path Read | none | 同上；vision 渲染走 `render_pages_cached` 内部锁 |
| `html_to_pdf` | path Read | out_path Write | WebView async handler |
| `typst_to_pdf` | path/dir Read | out_path Write + staging temp Write | staging name must be unique; final out lock needed |
| `skill_run` | script path Read if path mode | dynamic writes via runtime gate; script recovery workspace SubtreeWrite | inline script writes recovery file |
| `image_read` | image paths Read | none | cache/attachments read |
| `web_search` / `web_extract` | none | none | no project file locks |
| `clarify_ask` | none | none | no FS |

## File-Level Implementation Plan

### Backend

`src-tauri/src/state.rs`

- Add:

```rust
pub file_locks: Arc<FileLockRegistry>,
pub run_limiter: Arc<RunLimiter>,
```

`src-tauri/src/core/mod.rs`

- Export `file_locks`

`src-tauri/src/core/file_locks.rs` (new)

- Define `LockMode`, `FileResource`, `LockRequest`, `FileLockRegistry`, `FileLockGuard`
- Provide `acquire_many(owner, requests)` with atomic all-or-none behavior
- Sort/dedupe requests to avoid self-conflict
- Conflict error includes blocked path and owner session

`src-tauri/src/agent/turn_control.rs`

- Keep `TurnRegistry` for same-session active/reserved and cancel
- Remove checks that reject another active turn with same `project_id`
- Add `RunLimiter` here or in new `agent/run_limiter.rs`
- Update tests: same project second session is no longer rejected by registry; global fourth run is rejected by limiter

`src-tauri/src/agent/loop_runner.rs`

- At `run_turn` before persisting user message:
  - check clarify pending
  - acquire `RunSlotGuard`（与随后 `ActiveTurnGuard` 同寿；`cancel_turn` 不 release）
  - register `TurnRegistry`
  - write user message
- Pass `session_id`, `turn_id`, `project_id`, `session_title` into `ToolContext`
- Ensure guards release on all returns
- `turn_awaiting_user`: release run slot and unregister; pending clarify does not count running

`src-tauri/src/tools/mod.rs`

- Extend `ToolContext`:

```rust
pub struct ToolContext<'a> {
    pub sandbox: &'a Sandbox,
    pub secrets: Option<&'a Secrets>,
    pub project_id: &'a str,
    pub session_id: &'a str,
    pub turn_id: &'a str,
    pub session_title: &'a str,
    pub file_locks: Option<Arc<FileLockRegistry>>,
}
```

- Provide `ToolContext::for_turn(...)`; keep `new()` only for tests with inert lock context

`src-tauri/src/tools/io_plan.rs` (new)

- Implement static plans per matrix
- Unit test every registered filesystem-writing tool
- Assert all `ToolRegistry::default_tools()` are either in matrix or explicitly `NoFs`

`src-tauri/src/agent/loop_tool_batch.rs`

- Before `state.tools.execute`, call `plan_tool_io`
- Acquire lock guard
- For `skill_run`, pass dynamic write gate to runtime
- On lock conflict, return `ExecOutcome { ok: false, summary: structured_error, changed_paths: [] }`

`src-tauri/src/core/cache_paths.rs`

- Add:

```rust
pub const OOXML_WORK_DIR: &str = ".cache/ooxml";
pub const TURN_TMP_DIR: &str = ".cache/tmp"; // reserved; no caller yet
pub fn cache_key(parts: &[&str]) -> String;
pub fn skill_run_script(session_id: &str) -> String;
pub fn skill_run_error(session_id: &str) -> String;
pub fn turn_tmp_dir(session_id: &str, turn_id: &str) -> String; // reserved
pub fn ooxml_work_dir(session_id: &str, turn_id: &str, source_path: &str) -> String;
```

`ooxml_work_dir` 返回 `.cache/ooxml/{session_key}/{work_key}`，不含文件名 stem。

`src-tauri/src/tools/skill_run_tmp.rs`

- Replace fixed constants with functions requiring context
- `cleanup_on_turn_end` deletes `.cache/skill-run/<session_key>/` when no `error.json`
- Error retention keeps session scratch until fixed or turn ends clean

`src-tauri/src/tools/runtime/mod.rs` and `ops.rs`

- Add write gate capture to `execute_script`
- `__doc_write` obtains dynamic lock before write
- Track `written_paths` unchanged

`src-tauri/src/tools/ooxml/mod.rs`

- Make `out_dir` optional in schema
- Generate default via `cache_paths::ooxml_work_dir`
- Return relative `out_dir`
- Ensure `changed_paths` uses returned/generated `out_dir`, not missing arg

`src-tauri/src/tools/changed_paths.rs`

- For `ooxml_unpack`, prefer result `out_dir` when arg missing
- Keep UI refresh behavior for generated `.cache/ooxml` harmless because `.cache` hidden

`src-tauri/src/core/skills.rs` and `src-tauri/assets/skills/{docx,pptx,xlsx,runtime}/*.md`

- Update instructions:
  - Do not hard-code `unpacked/`
  - Use returned `out_dir`
  - Do not hard-code `.cache/skill-run/script.js`; use `script_path` from error/success response

### Frontend

`src/lib/sessionRunState.ts`

- Add selector:

```ts
export function runningSessionCount(state: SessionRunsState): number
```

- Keep per-session event model

`src/hooks/useWorkspace.ts`

- On `turn_complete` / `turn_cancelled` for non-active session:
  - refresh session list for that project
  - call project files event handler as today
  - do not apply messages unless active, but mark stale session so switching later reloads
- Show global full error from `send_message` catch without clearing input
- Show file lock conflict structured error as toast or inline error

`src/lib/sendReadiness.ts`

- Optionally add local blocker if `runningCount >= 3`; backend remains authoritative

`src/components/ChatPanel.tsx`

- Disable send with tooltip/banner when local running count is 3

## Testing Strategy

### Rust unit tests

- `file_locks.rs`
  - read/read same file allowed
  - read/write same file conflicts
  - write/write same file conflicts
  - subtree write conflicts with descendant write/read
  - same path in different project allowed
  - acquire_many all-or-none
  - conflict message includes session title
- `io_plan.rs`
  - every write tool produces expected locks
  - `ooxml_unpack` without out_dir produces generated subtree lock
  - `skill_run` marks dynamic_writes and locks recovery dir
  - `pdf_read` / `pdf_render_pages` only lock source `path` Read; no `.cache/pdf/**` locks
  - web tools produce no FS locks
- `cache_paths.rs`
  - session/turn paths are POSIX and cannot contain `..`

### Rust integration tests

- Three sessions across projects can run concurrently; fourth is rejected before user message persists
- Three sessions in one project can run if they write different files
- Same project two sessions writing `out.docx`: second tool result is file_busy
- Same project one session unpacking `a.docx` and another unpacking `b.docx` without out_dir: both succeed with different `.cache/ooxml/...`
- Two sessions both explicit `out_dir: "unpacked"`: while session A's turn still holds `TurnFileLockStore` (e.g. after `ooxml_unpack`, before `ooxml_pack`), session B gets file_busy on the same path; after A's turn ends (including `turn_awaiting_user`), locks release — prefer auto-generated `ooxml_work_dir` over cross-session shared explicit paths
- `ooxml_pack` holds `SubtreeWrite(dir)`：打包期间另一会话对 `dir` 子文件的 `fs_write` 返回 file_busy
- Two `skill_run` inline scripts in different sessions retain separate `script_path`
- `skill_run` dynamic write to locked `out.xlsx` returns file_busy before overwriting
- cancel during tool batch releases locks so a later turn can write same file
- clarify awaiting releases global slot (`RunLimiter::occupied_count()` returns to prior level)
- clarify submit rejected when global limit full; pending clarify preserved
- session in stopping state (cancel signalled, loop not exited) still blocks 4th global slot until `turn_cancelled`
- two sessions same PDF cache miss: render_lock serializes; both succeed; source Read locks coexist
- read PDF while another session writes same PDF: writer or reader tool gets file_busy per lock mode

### Frontend tests

- `sessionRunState` running count selector
- local full-capacity blocker preserves input
- non-active `turn_complete` updates session status without replacing active messages
- file busy error displays the blocked path

## Risks / Mitigations

- **Model still writes fixed paths in scripts**: runtime dynamic write lock prevents damage; skills update reduces frequency.
- **Over-conservative locks reduce parallelism**: start with write-focused locks; broad project read locks are avoided except for recursive operations.
- **Deadlock**: no waiting and acquire_many all-or-none means no deadlock. Conflict rejects immediately.
- **Lock leak**: `TurnFileLockStore` + `FileLockGuard` RAII; terminal-path tests cover cancel/error/complete/awaiting_user.
- **Generated `.cache/ooxml` accumulates**: accepted for MVP. Future cleanup can remove successful turn work dirs; this change does not add GC UI.
- **`skill_run` tests need context**: provide test helper context with fake session/turn and in-memory lock registry.

## Rollout Order

1. Implement path helpers and session/work scratch dirs while project-level mutex still exists.
2. Add FileLockRegistry and ToolIoPlan behind the existing mutex; test in isolation.
3. Wire tool execution locks and runtime dynamic lock.
4. Update skills to use returned paths.
5. Replace project mutex with global 3 limiter.
6. Update frontend full-capacity and background terminal handling.
7. Run full verification.
