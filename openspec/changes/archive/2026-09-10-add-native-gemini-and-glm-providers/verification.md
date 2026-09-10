# Verification: Google OpenAI compatibility redesign

## Scope and database decision (2026-09-10)

Gemini now uses the official OpenAI-compatible Chat Completions endpoint through the shared Rust client. Interactions request/step/schema conversion and wire/local ID mapping were removed. The user confirmed there are no existing Interactions sessions; no migration or fallback branch is implemented.

Keep the single nullable messages.provider_state_json column. It now contains only versioned message/tool extra_content metadata and signatures, not steps, text, reasoning, arguments or IDs. Removing all persistence for these values would break tool continuation and clarify resume (the protocol probe returned HTTP 400 when signatures were removed). No second column, tool table migration, model rewrite or user database cleanup was added. Tool reads use message seq then insertion rowid to preserve metadata order across identical timestamps.

## Automated coverage

- Actual OpenAiCompatClient HTTP request: Bearer authentication, /v1beta/openai/chat/completions, messages body, streamed Unicode response.
- Shared SSE byte framing: one-byte UTF-8 chunks, mixed LF/CRLF, comments, multiline data, DONE, truncated frames/JSON, EOF without finish, cancellation while stalled and while waiting for headers.
- No-index parallel calls keyed by id, argument fragments, normalized UI indices, missing/invalid/conflicting signatures, complete and partial arguments in a length response.
- Thought tags across chunks and combined non-stream response, ordinary literal tags retained, summary separated from answer, signature-only metadata, total-based output usage.
- Real Store close/reopen for two same-name tools, normalized IDs, ordered signatures, third tool and non-JSON error result; clarify pending/reload/result without new user message.
- Compaction retains the entire active chain and the preserved messages can be re-encoded; signatures do not enter summary text or pending estimates.
- One-column idempotent migration and old-provider NULL behavior; message/turn Debug and IPC JSON omit state.
- Complete tool registry definitions are encoded without stripping schema constraints; this is an offline equality check, not a remote acceptance claim for project schemas.
- Auxiliary title/summary/vision consumers require a complete nonempty text response and reject truncated or tool responses.

## Real API tests

Command (credentials read from environment, never printed):

```sh
DOC_AGENT_GOOGLE_SMOKE=1 cargo test --manifest-path src-tauri/Cargo.toml provider::gemini::live_tests --lib -- --nocapture
```

Both product-provider tests passed: parallel alpha/beta calls -> local UUID normalization -> real temporary SQLite close/reopen -> gamma -> tool error -> final answer -> next user question; high-effort thought-summary separation and an in-memory generated red PNG image. These tests call the product Rust provider rather than an independent Python implementation. Opted-in missing keys and failures fail the tests; without the explicit flag they do not contact model APIs.

Earlier independent probes (2026-09-09) additionally confirmed low/medium/high acceptance, json_object, two images, raw synthetic oneOf/not acceptance, missing-signature HTTP 400 and length finish. They do not establish arbitrary JSON Schema enforcement.

## Gates

- npm run typecheck: passed.
- npm test: 341 tests passed across 64 files.
- npm run build: passed (existing bundle-size advisory only).
- Initial full Rust regression: 587 passed, including local HTTP/cancellation/CONNECT tests.
- Final fmt/clippy/full-test results are recorded after the final auxiliary-response checks.

## Remaining acceptance boundaries

No project documents, real project tool schemas, user database or keys were uploaded as test data. Full project-schema remote acceptance remains unclaimed. The existing packaged desktop UI screenshots and macOS/Windows Clash/TUN GUI matrices (tasks 8.4, 9.3, 9.5) remain open; loop/store/API tests and local CONNECT do not replace those platform checks. These acceptance tasks remain incomplete; the user explicitly authorized archiving and release on 2026-09-10 after these boundaries were disclosed.

## Release preparation: 2026.9.10 (2026-09-10)

- Full `npm run release:check` passed at version 2026.9.10: vendor/JS bundles, Rust fmt/clippy/test, frontend typecheck, 345 frontend tests and production build. Only the existing frontend bundle-size advisory remains. Local log: `/tmp/doc-agent-release-check-2026.9.10.log`. OpenSpec strict validation and git diff --check also passed.
- package.json, package-lock.json (root and package entry), Cargo.toml, Cargo.lock and tauri.conf.json are synchronized to 2026.9.10; CHANGELOG has the dated release section.
- GitHub read-only inspection found no remote 2026.9.10 tag. Remote main was 1168e666432b061e77c27fa90e2bfc1df54ffe11; local HEAD 4a5f183 also contains the OpenSpec workflow update. Release changes remain uncommitted during preparation.
- The last Release run (28297063313, tag 2026.6.28) succeeded. Required OSS and updater signing secret names are present; secret values and validity were not inspected.
- A tag push automatically runs checks, builds macOS aarch64 and Windows x86_64 bundles, uploads OSS versioned assets, replaces latest.json and publishes a public GitHub Release. It is not a draft-only operation.
- README now requires committing release changes and a clean worktree before tagging, then atomically pushing main and the tag.
- Legacy DeepSeek Flash UI regression coverage added four tests; frontend total is now 345. The three previously sandbox-blocked local network tests passed when loopback listeners were allowed.
- No tag or release was created. Packaged GUI/Clash/TUN acceptance remains open as documented above; preparation does not certify those platforms or archive incomplete tasks.

## Archive decision (2026-09-10)

The user requested OpenSpec archive followed by release of 2026.9.10. Main specs were synchronized across all eight delta capabilities and checked against every changed requirement. Incomplete acceptance tasks retain unchecked boxes; archive does not represent successful packaged GUI, Windows/macOS Clash/TUN or extended proxy acceptance. The full release gate passed with 588 Rust tests and 345 frontend tests.

Archive validation: all 42 main specs pass standard validation; the change passes strict validation. Global strict validation still reports pre-existing Purpose placeholder/length warnings. A pre-existing missing cancellation scenario in workspace-ui was documented to match its existing requirement; no runtime behavior changed.
