## 1. Argument and Streaming Diagnostics

- [x] 1.1 Replace the silent `{}` fallback for malformed tool-call arguments in `loop_runner.rs` with a structured tool error that includes parser detail, line, column, snippet, and a tool-specific hint.
- [x] 1.2 Ensure malformed argument errors are persisted as failed tool calls and added as tool messages so the Agent can correct the same turn instead of aborting silently.
- [x] 1.3 Extend SSE provider handling to capture `finish_reason` from streamed responses.
- [x] 1.4 Detect `finish_reason == "length"` during or after tool-call streaming and surface a truncation diagnostic instead of generic JSON or missing-argument errors.
- [x] 1.5 Add unit tests for malformed arguments, including a `skill_run` payload with an unescaped ASCII quote inside a long code string.
- [x] 1.6 Add unit tests for streamed tool-call truncation metadata and the resulting diagnostic path.

## 2. Temporary Script Directory and Path Execution

- [x] 2.1 Update the `skill_run` JSON schema to accept exactly one of `code` or `path`, while preserving `timeout_secs`.
- [x] 2.2 Implement argument validation that rejects calls with both `code` and `path`, or with neither.
- [x] 2.3 Implement `.skill-run/script.js` creation for inline `skill_run.code` before execution.
- [x] 2.4 Implement sandboxed `skill_run.path` file loading for project-relative JavaScript files.
- [x] 2.5 Delete `.skill-run/` after any successful `skill_run` execution. (Revised by section 5: two-tier cleanup.)
- [x] 2.6 Preserve `.skill-run/script.js` after failed inline execution and include `script_path` in the tool error result.
- [x] 2.7 Write `.skill-run/error.json` on failed `skill_run` execution with the structured diagnostic returned to the Agent.
- [x] 2.8 Add tests covering inline success cleanup, inline failure preservation, path rerun success cleanup, and sandbox rejection for out-of-project paths.

## 3. Runtime Source Diagnostics

- [x] 3.1 Add source-context helpers that map JavaScript error line/column information to nearby script lines when available.
- [x] 3.2 Add quote diagnostics for suspicious source lines, reporting quote characters with code points such as `U+0022`, `U+201C`, and `U+201D`.
- [x] 3.3 Update `runtime::execute_script` error handling to return structured diagnostics for parse and runtime failures without silently rewriting script text.
- [x] 3.4 Include `.skill-run/script.js` in diagnostics when the failure came from an inline saved script or temporary path rerun.
- [x] 3.5 Add tests using the sample failure shape `p("...简称"广软"），...")` to verify the diagnostic points at ASCII `U+0022` quotes and includes source context.

## 4. Documentation and Integration

- [x] 4.1 Update `skill_run` tool description to mention long-script recovery via `.skill-run/script.js` and `skill_run.path`.
- [x] 4.2 Add short guidance to document skills that long JavaScript failures can be repaired by editing `.skill-run/script.js` and rerunning with `path`.
- [x] 4.3 Verify existing `skill_run.code` examples still work unchanged.
- [x] 4.4 Run Rust formatting, clippy, and tool/runtime tests relevant to `agent`, `tools::skill`, and `tools::runtime`.
- [x] 4.5 Manually exercise the recovery flow: run a failing long script, inspect `.skill-run/script.js`, fix one line, rerun with `path`, and confirm `.skill-run/` is removed on success.

## 5. Post-review: fs_patch and Turn-End Cleanup (D6 revised, D7)

- [x] 5.1 Add `fs_patch` tool with atomic exact-substring edits (`old`/`new`/`replace_all`), rejecting empty or identical edits, and structured errors listing missed edits without writing partial changes.
- [x] 5.2 Retain `.skill-run/script.js` after successful runs that wrote Office deliverables or returned `style_warnings`; include `script_path`, `script_retain_reason`, and repair hint in the success response.
- [x] 5.3 Clear stale `.skill-run/error.json` on any successful `skill_run`.
- [x] 5.4 Add turn-end cleanup in `loop_runner.rs` (both normal completion and max-steps exits): remove `.skill-run/` unless `error.json` marks a pending failure.
- [x] 5.5 Update skill docs (`docx/xlsx/pptx`) and tool descriptions: repair with `fs_patch` + `path` rerun; cleanup is automatic at turn end.
- [x] 5.6 Tests: fs_patch unique/replace_all/atomicity/validation, stale error.json clearing, turn-end cleanup with and without pending failure.
