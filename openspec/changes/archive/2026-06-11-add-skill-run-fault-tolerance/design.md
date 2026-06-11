## Context

`skill_run` currently accepts a `code` string, normalizes common model-generated JavaScript patterns, and executes the script in the embedded Boa runtime. This works well for short scripts, but Office document generation often produces long scripts containing large Chinese text literals, tables, headings, headers, footers, and style declarations. In those scripts, a single ASCII double quote (`"`, `U+0022`) inside a double-quoted JavaScript string can make the entire script fail to parse.

The user-facing failure mode is poor today:

- streamed tool call arguments can be long, but the agent loop later parses the final `arguments` string with `serde_json::from_str`
- when that JSON parse fails, the loop silently falls back to `{}`, which loses the real error and usually produces `code required`
- JavaScript parse errors do not reliably include enough source context for the Agent to repair only the failing line
- long scripts are not persisted anywhere, so the Agent tends to regenerate the full script, increasing the chance of a new quoting or truncation error

The target audience is office users, not developers. The temporary repair directory should therefore be simple, short-lived, and mostly invisible. We will use a single project-local hidden directory named `.skill-run/`, and clean it up after successful execution.

## Goals / Non-Goals

**Goals:**

- Preserve free-form JavaScript as the primary `skill_run` interface.
- Persist the latest `skill_run.code` script to `.skill-run/script.js` before execution.
- Keep `.skill-run/` only when execution fails, so the Agent can inspect or repair `script.js`.
- Add `skill_run.path` to execute a script file inside the project sandbox.
- Clean `.skill-run/` after any successful `skill_run` call.
- Return precise diagnostics for invalid tool arguments JSON, JavaScript parse/runtime failures, quote-related syntax errors, and streamed output truncation.
- Keep implementation simple: one temporary directory, no history, no TTL, no cache outside the project.

**Non-Goals:**

- Do not introduce a structured document DSL.
- Do not force content/data separation for document generation.
- Do not preserve failed scripts for auditing or history.
- Do not modify `.gitignore`; the product is oriented toward office users and `.skill-run/` should normally be deleted automatically.
- Do not add external JavaScript, Node.js, or shell dependencies.

## Decisions

### D1: Use `.skill-run/` as a simple project-local temporary repair directory

`skill_run.code` will create or overwrite:

```text
<project-root>/
  .skill-run/
    script.js
    error.json    # only on failure
```

Successful runs delete the entire `.skill-run/` directory. Failed runs keep it, allowing the Agent to read or edit `.skill-run/script.js` and retry with `skill_run.path`.

Rationale:

- A single fixed directory is easier for the Agent and office users to understand than timestamped run history.
- Failed scripts are only recovery material, not durable artifacts.
- Cleaning on success keeps the directory invisible in normal use.

Alternative considered: application cache outside the project. Rejected because it adds path plumbing and a second sandbox boundary while providing little value if failed scripts are not long-term assets.

### D2: `skill_run` accepts exactly one script source: `code` or `path`

The tool schema will allow:

```json
{
  "code": "async function main() { return { ok: true }; }",
  "timeout_secs": 30
}
```

or:

```json
{
  "path": ".skill-run/script.js",
  "timeout_secs": 30
}
```

If both `code` and `path` are present, the tool returns an invalid-arguments error. If neither is present, it returns an invalid-arguments error. `path` MUST resolve through the existing project `Sandbox`, so scripts cannot be read outside the project.

For `code`:

1. create or replace `.skill-run/script.js`
2. execute the in-memory code or the saved file contents
3. on success, delete `.skill-run/`
4. on failure, write `.skill-run/error.json` and keep the directory

For `path`:

1. read the script through `Sandbox`
2. execute it
3. on success, delete `.skill-run/` if it exists
4. on failure:
   - if the path is inside `.skill-run/`, update `.skill-run/error.json`
   - if the path is a user-authored script elsewhere, do not copy or delete it

Example handler shape:

```rust
enum ScriptSource {
    Inline { code: String },
    Path { path: String, code: String },
}

fn resolve_script_source(ctx: &ToolContext, args: &Value) -> Result<ScriptSource, ToolError> {
    let code = args.get("code").and_then(|v| v.as_str());
    let path = args.get("path").and_then(|v| v.as_str());

    match (code, path) {
        (Some(_), Some(_)) => Err(ToolError::InvalidArgs(
            "skill_run accepts either code or path, not both".into(),
        )),
        (Some(code), None) => Ok(ScriptSource::Inline { code: code.to_string() }),
        (None, Some(path)) => {
            let resolved = ctx.sandbox.resolve(path)?;
            let code = std::fs::read_to_string(resolved)
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            Ok(ScriptSource::Path { path: path.to_string(), code })
        }
        (None, None) => Err(ToolError::InvalidArgs("code or path required".into())),
    }
}
```

### D3: Make tool argument JSON parse failures visible

The agent loop currently parses tool call arguments and silently substitutes `{}` on failure. This must change for all tools, but the primary motivation is long `skill_run` calls.

Instead of executing the tool with `{}`, the loop should:

1. persist a failed tool call result with status `error`
2. add a tool message containing a structured parse error
3. continue the loop so the model can correct the call

The diagnostic should include the parser error, line/column, and a short snippet around the failure offset.

Example structured error:

```json
{
  "error": "invalid tool arguments JSON",
  "detail": "expected `,` or `}` at line 1 column 1842",
  "line": 1,
  "column": 1842,
  "snippet": "...简称\"广软\"），是经教育部批准...",
  "hint": "If this is skill_run code, avoid embedding long double-quoted text in the JSON arguments; use single-quoted JavaScript strings or rerun from .skill-run/script.js after repair."
}
```

Example helper shape:

```rust
fn parse_tool_args(raw: &str, tool_name: &str) -> Result<Value, Value> {
    serde_json::from_str(raw).map_err(|err| {
        json!({
            "error": "invalid tool arguments JSON",
            "detail": err.to_string(),
            "line": err.line(),
            "column": err.column(),
            "snippet": snippet_around_line_column(raw, err.line(), err.column(), 120),
            "hint": argument_parse_hint(tool_name),
        })
    })
}
```

### D4: Add source-aware JavaScript diagnostics

`runtime::execute_script` should return errors that carry source context when possible. For parse or runtime errors, the diagnostic should include:

- error message
- source line and column when available
- one to three nearby source lines
- quote diagnostics for suspicious lines containing ASCII double quotes inside likely text-heavy calls such as `p("...")`, `cell("...")`, `bulletItem("...")`, or `new TextRun("...")`

Example user-visible diagnostic:

```json
{
  "error": "JavaScript parse error",
  "detail": "unexpected identifier",
  "line": 204,
  "column": 58,
  "source": "p(\"广州软件学院（Software Engineering Institute of Guangzhou，简称\"广软\"），是经教育部批准设立的全日制非营利性民办普通本科高等学校。\", { indent: 480 }),",
  "quote_diagnostics": [
    { "column": 3, "char": "\"", "code_point": "U+0022", "name": "QUOTATION MARK" },
    { "column": 58, "char": "\"", "code_point": "U+0022", "name": "QUOTATION MARK", "note": "This ASCII quote terminates the JavaScript string." },
    { "column": 61, "char": "\"", "code_point": "U+0022", "name": "QUOTATION MARK" }
  ],
  "script_path": ".skill-run/script.js",
  "hint": "The quotes around 广软 are ASCII U+0022, not smart quotes. Repair .skill-run/script.js locally and rerun with skill_run {\"path\":\".skill-run/script.js\"}."
}
```

Quote diagnostics must be careful not to claim a guaranteed fix. They should identify suspicious characters and explain likely causes.

Example quote helper:

```rust
fn quote_diagnostics(line: &str) -> Vec<Value> {
    line.char_indices()
        .filter_map(|(byte_idx, ch)| {
            let name = match ch {
                '"' => "QUOTATION MARK",
                '\'' => "APOSTROPHE",
                '“' => "LEFT DOUBLE QUOTATION MARK",
                '”' => "RIGHT DOUBLE QUOTATION MARK",
                '‘' => "LEFT SINGLE QUOTATION MARK",
                '’' => "RIGHT SINGLE QUOTATION MARK",
                _ => return None,
            };
            Some(json!({
                "column": line[..byte_idx].chars().count() + 1,
                "char": ch.to_string(),
                "code_point": format!("U+{:04X}", ch as u32),
                "name": name,
            }))
        })
        .collect()
}
```

### D5: Detect streamed tool-call truncation

The SSE consumer should observe `choices[0].finish_reason`. If the assistant turn ends with `finish_reason == "length"` while a tool call argument stream is in progress, the Agent should receive a clear truncation error instead of attempting to parse a partial JSON string as if it were complete.

Example behavior:

```json
{
  "error": "tool call truncated",
  "tool": "skill_run",
  "received_argument_chars": 12480,
  "hint": "The model output ended before the tool arguments were complete. Retry with a shorter script or use skill_run.path after writing the script file."
}
```

This requires representing finish reason on `AssistantTurn` or returning a provider parse error with enough context to the loop. The implementation should keep UI streaming progress intact.

### D6: Two-tier cleanup — immediate for pure scripts, turn-end for deliverables

(Revised after review.) `.skill-run/` cleanup happens at two levels:

**Immediate cleanup** — a successful `skill_run` that wrote no Office deliverable
(`.docx/.pptx/.xlsx/.xlsm`) and produced no `style_warnings` deletes `.skill-run/` right away.

**In-turn retention** — a successful run that wrote an Office deliverable (or returned
`style_warnings`) keeps `.skill-run/script.js` so the Agent can verify the document
(`office_read_to_markdown`, `style_warnings`) and repair it via `fs_patch` + `skill_run.path`
within the same turn. The success response includes `script_path`, `script_retain_reason`,
and a repair hint. A successful run also clears any stale `.skill-run/error.json`.

**Turn-end guaranteed cleanup** — when the agent turn ends (normal completion or max tool
steps), the loop runner removes `.skill-run/` unless `.skill-run/error.json` exists. The
error file marks an unrepaired failure whose script must survive into the next turn.
This makes cleanup deterministic: whether or not the Agent handles `style_warnings`, the
directory is removed as long as no script failure is pending. Cleanup never depends on the
model remembering to call a tool.

```rust
// loop_runner.rs, at both turn exits
fn cleanup_skill_run_tmp(sandbox: &Sandbox) {
    let ctx = ToolContext::new(sandbox);
    skill_run_tmp::cleanup_on_turn_end(&ctx); // removes dir unless error.json exists
}
```

### D7: fs_patch for local script repair

`fs_patch` applies exact substring replacements (`old` → `new`, optional `replace_all`) to a
UTF-8 file. It is the preferred repair path for `.skill-run/script.js`, avoiding full-file
`fs_write` rewrites that re-introduce JSON escaping risk for long Chinese scripts.

Semantics are atomic: every edit must match (exactly once unless `replace_all`), otherwise
no change is written and a structured error lists each missed edit with its reason. This
prevents the confusing half-applied state where a retry of the same edit list reports
already-applied edits as `not found`. Empty `old` and `old == new` are rejected as invalid
arguments.

## Risks / Trade-offs

- **Risk: `.skill-run/` remains visible after an unrepaired failure.** → This is intentional and short-lived; the next successful `skill_run` removes it.
- **Risk: Deleting `.skill-run/` after an unrelated successful run removes a still-useful failed script.** → Acceptable under the simplified design. Failed scripts are recovery material, not durable history.
- **Risk: Diagnostics overstate quote root cause.** → Keep wording as “suspicious” and report objective code points rather than auto-fixing or asserting certainty.
- **Risk: `skill_run.path` could execute unexpected project files.** → It only reads through the existing sandbox and remains an Agent tool action; no shell or network capability is introduced.
- **Risk: More structured tool errors affect all tools.** → Keep the loop behavior compatible by returning tool messages with status `error` rather than aborting the whole turn.

## Example Recovery Flow

1. Agent calls `skill_run.code` with a long docx script.
2. Tool writes `.skill-run/script.js`.
3. Boa reports a parse error near:

   ```javascript
   p("广州软件学院（Software Engineering Institute of Guangzhou，简称"广软"），是经教育部批准设立的全日制非营利性民办普通本科高等学校。", { indent: 480 }),
   ```

4. Tool keeps `.skill-run/` and returns:

   ```json
   {
     "error": "JavaScript parse error",
     "line": 204,
     "column": 58,
     "script_path": ".skill-run/script.js",
     "hint": "Repair the ASCII U+0022 quotes around 广软, then rerun with skill_run {\"path\":\".skill-run/script.js\"}."
   }
   ```

5. Agent edits only `.skill-run/script.js`, for example:

   ```javascript
   p('广州软件学院（Software Engineering Institute of Guangzhou，简称"广软"），是经教育部批准设立的全日制非营利性民办普通本科高等学校。', { indent: 480 }),
   ```

6. Agent calls:

   ```json
   { "path": ".skill-run/script.js", "timeout_secs": 30 }
   ```

7. Tool succeeds, writes the requested `.docx`, returns the result with `script_path`, and keeps
   `.skill-run/script.js` (clearing `error.json`) so the Agent can verify the document and patch
   the script again if needed.
8. The Agent verifies with `office_read_to_markdown`; when the turn ends without a pending
   failure, the loop runner deletes `.skill-run/` automatically.
