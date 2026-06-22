# Repository Guidelines

## Project Structure & Module Organization

This is a Tauri 2 desktop app. Frontend code lives in `src/`: components in `src/components/`, state in `src/hooks/`, utilities in `src/lib/`, and setup in `src/test/`. Rust backend code lives in `src-tauri/src/`: `core/` is store/sandbox/secrets, `agent/` is providers/loop orchestration, `tools/` is registered handlers, and `ipc/` is only Tauri commands/events. Assets are in `public/`; requirements are in `openspec/`.

## Build, Test, and Development Commands

- `npm ci`: install the pinned Node dependencies.
- `npm run tauri dev`: run the desktop app.
- `npm run dev`: run the Vite frontend.
- `npm run typecheck`, `npm test`, `npm run build`: required frontend gates.
- `npm run bundle:js`: bundle document-skill JS libraries.
- `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`: required Rust gates.
- `npm run release:check`: run the full release gate.

## Coding Style & Naming Conventions

Use TypeScript and React function components. Keep components PascalCase, hooks as `useName.ts`, utilities camelCase, and tests as `*.test.ts` or `*.test.tsx`. Move orchestration into hooks or `src/lib/`. Rust follows `rustfmt` and snake_case. Keep files focused: Rust source usually <=300 lines, React components <=150, tests <=400. Justify new dependencies in `design.md` or the PR.

## Testing Guidelines

New or modified TypeScript/TSX code must add or update Vitest coverage unless the change is style, comment, or formatting only. Cover `src/lib/**` with unit tests, hooks with `renderHook`, and key UI with Testing Library. Rust changes need automated tests for sandbox paths, persistence, provider loops, tool handlers, or IPC contracts. New tools need JSON schema, handler tests, and OOXML validation.

## OpenSpec Workflow

Feature or behavior changes must start with OpenSpec artifacts under `openspec/changes/<change>/`: `proposal.md`, `design.md`, spec deltas with Requirements and Scenarios, and `tasks.md`. Implement only the approved MVP scope. If code conflicts with `design.md`, update the artifact first. Mark completed tasks `[x]`, then archive specs into `openspec/specs/`.

## Commit & Pull Request Guidelines

Use Conventional Commits, for example `feat(governance): ...`, `fix(runtime): ...`, and `chore(release): ...`. Keep commits scoped and imperative. PRs should describe behavior, list verification, link OpenSpec changes/issues, and include UI screenshots.

## Release & Agent-Specific Instructions

Releases use CalVer tags `YYYY.M.D` with no leading zeros and no `v` prefix; tag, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` versions must match. PR CI runs on `pull_request` to `main`; `push main` does not package. Agent file operations stay inside the selected project root and respect file-governance locks.
