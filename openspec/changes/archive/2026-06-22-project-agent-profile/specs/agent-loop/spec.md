## ADDED Requirements

### Requirement: System prompt includes project AGENTS.md

`build_working_messages` SHALL append project `AGENTS.md` content to the system message when the file exists, after built-in system text and skills index.

#### Scenario: Injection ordering

- **WHEN** constructing messages for a turn
- **THEN** the system message order SHALL be: base system instructions, skills `index_markdown()`, then optional `## 项目配置（AGENTS.md）` section

#### Scenario: Injection respects character budget

- **WHEN** `AGENTS.md` exceeds the configured inject limit (3000 characters in MVP)
- **THEN** the injected excerpt SHALL be truncated deterministically without failing the turn
