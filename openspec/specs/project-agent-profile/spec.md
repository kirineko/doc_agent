# project-agent-profile Specification

## Purpose

项目级 Agent 配置文件 `AGENTS.md` 的读盘注入、`/init` 初始化流程与写入门禁。

## Requirements

### Requirement: Project agent profile file location

The system SHALL treat `<project_root>/AGENTS.md` as the canonical project-level agent profile file.

#### Scenario: Profile file at project root

- **WHEN** a project is opened with root path `/path/to/project`
- **THEN** the profile file path SHALL be `/path/to/project/AGENTS.md` (POSIX relative path `AGENTS.md` within sandbox)

#### Scenario: Hand-edited profile takes effect without init

- **WHEN** the user creates or edits `AGENTS.md` outside the application
- **AND** a subsequent agent turn runs for any session in that project
- **THEN** the updated file content SHALL be reflected in system injection without requiring `/init`

### Requirement: AGENTS.md injection into working messages

On each agent turn, the system SHALL read `AGENTS.md` from disk when present and append a bounded excerpt to the system message in `build_working_messages`.

#### Scenario: Profile injected when file exists

- **WHEN** `AGENTS.md` exists and is non-empty
- **THEN** the system message SHALL include a section headed `## 项目配置（AGENTS.md）` followed by file content truncated to at most 3000 characters

#### Scenario: No profile file

- **WHEN** `AGENTS.md` does not exist
- **THEN** the system message SHALL NOT include the project configuration section
- **AND** agent behavior SHALL match pre-change behavior aside from other unrelated prompts

#### Scenario: Per-turn disk read

- **WHEN** multiple turns run in the same session
- **THEN** each turn SHALL read `AGENTS.md` from disk at turn start (no cross-turn in-memory cache in MVP)

### Requirement: Profile init decoupled from injection

AGENTS.md injection and the `/init` command SHALL be independent capabilities: injection depends only on file existence; `/init` is optional for generating or updating the file.

#### Scenario: Injection without ever running init

- **WHEN** the user hand-writes `AGENTS.md` and never invokes `/init`
- **THEN** every agent turn SHALL still inject the profile per the injection requirement

### Requirement: Init slash command consumes a turn

The `/init` slash command SHALL be a real command that sends a user message and starts a normal agent turn using the current session model and thinking settings.

#### Scenario: Init message shown verbatim

- **WHEN** the user submits `/init` or `/init 固化PPT风格`
- **THEN** the chat SHALL display the user message exactly as typed (including the `/init` prefix and optional tail)

#### Scenario: Init uses current session model

- **WHEN** `/init` starts a turn while the session model is locked to provider X
- **THEN** the init turn SHALL use provider X and the session thinking configuration

#### Scenario: Init turn reads session history

- **WHEN** `/init` runs after prior messages in the same session
- **THEN** `working_messages` for that turn SHALL include prior session messages so the agent can incorporate conversation context

### Requirement: Init turn workflow via clarify and profile skill

During an init turn, the agent SHALL follow the `profile` skill: read existing `AGENTS.md`, inspect project files, ask clarify questions, obtain `confirm_agents_md` approval, then write `AGENTS.md`.

#### Scenario: Init reads existing profile before questions

- **WHEN** an init turn starts and `AGENTS.md` already exists
- **THEN** the agent SHALL read the existing file before proposing updates

#### Scenario: Init ends with short changelog summary

- **WHEN** the init turn completes after writing `AGENTS.md`
- **THEN** the final assistant message SHALL summarize what changed in brief prose
- **AND** SHALL NOT paste the full `AGENTS.md` body into the chat message

#### Scenario: Empty project and empty session init allowed

- **WHEN** the user runs `/init` on a project with no office files and a session with no prior messages
- **THEN** the init turn SHALL proceed with general preference questions and MAY create a skeleton `AGENTS.md`

### Requirement: AGENTS.md write restricted to init turns

The agent SHALL NOT write or patch `AGENTS.md` via `fs_write` or `fs_patch` except during a turn marked as profile init.

#### Scenario: Non-init turn rejects AGENTS.md write

- **WHEN** a normal document-editing turn calls `fs_write` with path `AGENTS.md`
- **THEN** the tool SHALL fail with an error instructing the agent to use `/init` for profile updates

#### Scenario: Init turn allows AGENTS.md write

- **WHEN** a turn is marked profile init and `confirm_agents_md` has been answered affirmatively
- **THEN** `fs_write` or `fs_patch` targeting `AGENTS.md` SHALL succeed within sandbox limits

### Requirement: Init blocked while clarify pending

The system SHALL reject starting `/init` when any clarify question is pending for the session.

#### Scenario: Backend rejects init send with pending clarify

- **WHEN** `clarify_pending` is non-null for the session
- **AND** the user attempts to send a message whose trimmed content starts with `/init`
- **THEN** `send_message` SHALL fail with a user-visible error to complete or dismiss pending clarification first

#### Scenario: Frontend disables init command while clarify pending

- **WHEN** the UI shows a pending clarify card
- **THEN** submitting `/init` from the slash menu or composer SHALL be prevented with the same user-visible rationale

### Requirement: Profile skill discoverability

The system SHALL ship a built-in `profile` skill readable via `skill_read` that documents init workflow, question guidance, and AGENTS.md schema.

#### Scenario: Profile skill in index

- **WHEN** the agent receives the skills index in system prompt
- **THEN** the index SHALL list the `profile` skill alongside format skills
