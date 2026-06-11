## Why

`skill_run` is the main escape hatch for complex Office document generation, but long scripts are fragile: one unescaped quote, truncated tool argument, or JavaScript parse error can make the whole run fail and force the Agent to regenerate hundreds of lines. The current failure path often hides the real root cause, so the Agent guesses at fixes and may repeatedly rewrite otherwise valid scripts.

This change improves fault tolerance without constraining script flexibility. It keeps `skill_run` as free-form JavaScript, but adds precise diagnostics, temporary script persistence, `path`-based reruns, and cleanup so long scripts can be repaired locally instead of regenerated.

## What Changes

- Add precise diagnostics for failed `skill_run` calls:
  - invalid tool arguments JSON reports parser line/column and a nearby raw snippet instead of silently becoming empty args
  - JavaScript parse/runtime failures report useful source location and nearby script lines when available
  - quote-related failures distinguish ASCII `"` (`U+0022`) from smart quotes such as `“` (`U+201C`) and `”` (`U+201D`)
  - streamed tool calls that finish because of model output length are reported as truncation, not as generic argument or JavaScript errors
- Add a simple project-local temporary script directory named `.skill-run/`.
- Persist `skill_run.code` into `.skill-run/script.js` before execution so failed long scripts have a repair target.
- Add `skill_run.path` as an alternative to `code`, allowing the Agent to rerun a repaired script from the project sandbox.
- Clean up `.skill-run/` after any successful `skill_run` execution.
- Keep `.skill-run/` only when a run fails, so the Agent can inspect or edit `script.js` and retry.
- Do not introduce a structured document DSL or require content/data separation; free-form JavaScript remains the primary interface.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `script-runtime`: `skill_run` gains fault-tolerant diagnostics, temporary script persistence, `path` execution, and success cleanup behavior.

## Impact

- Affected backend modules:
  - `src-tauri/src/agent/provider/sse.rs` for finish reason detection during streamed tool calls
  - `src-tauri/src/agent/types.rs` and provider plumbing if `finish_reason` needs to be represented on assistant turns
  - `src-tauri/src/agent/loop_runner.rs` for tool argument parse diagnostics and avoiding silent `{}` fallback
  - `src-tauri/src/tools/skill.rs` for `skill_run.path`, temporary script save, and cleanup orchestration
  - `src-tauri/src/tools/runtime/*` for source-aware JavaScript diagnostics
- Affected OpenSpec capability:
  - `openspec/specs/script-runtime/spec.md`
- Affected skill documentation:
  - `src-tauri/assets/skills/*` may receive short guidance that long generated scripts can be repaired via `.skill-run/script.js` and rerun with `skill_run.path`.
- No external runtime dependency is added.
- No breaking change is intended for existing `skill_run.code` callers.
