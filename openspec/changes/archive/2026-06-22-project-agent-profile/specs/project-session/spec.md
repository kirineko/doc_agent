## ADDED Requirements

### Requirement: Shared project profile across sessions

Project-level `AGENTS.md` SHALL be shared across all sessions of a project while conversational message history remains session-scoped.

#### Scenario: New session inherits profile injection

- **WHEN** session B is created in a project where session A previously wrote or the user hand-edited `AGENTS.md`
- **THEN** session B agent turns SHALL inject the same `AGENTS.md` content
- **AND** session B SHALL NOT automatically include session A chat messages in `working_messages`

#### Scenario: Profile update visible to all sessions on next turn

- **WHEN** `AGENTS.md` is updated via `/init` in session A
- **THEN** the next agent turn in session B SHALL inject the updated profile after disk read
