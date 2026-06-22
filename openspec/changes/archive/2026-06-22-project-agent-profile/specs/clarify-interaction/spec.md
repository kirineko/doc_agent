## ADDED Requirements

### Requirement: Confirm agents markdown clarify kind

The system SHALL support a clarify question kind `confirm_agents_md` for approving a proposed `AGENTS.md` body before write.

#### Scenario: confirm_agents_md schema

- **WHEN** the agent calls `clarify_ask` with `kind` set to `confirm_agents_md`
- **THEN** the question SHALL include `preview_markdown` (non-empty string) and MAY include `changelog_summary` (short string)
- **AND** the question SHALL NOT require `brief` (unlike `confirm_brief`)

#### Scenario: confirm_agents_md answer records approval

- **WHEN** the user confirms a `confirm_agents_md` question
- **THEN** `clarify_answer` SHALL record affirmative approval for resume_turn
- **AND** the approved `preview_markdown` SHALL be available to the agent only via tool result context, not as a new user chat message

#### Scenario: confirm_agents_md rejects empty preview

- **WHEN** `clarify_ask` is called with `kind` `confirm_agents_md` and empty or missing `preview_markdown`
- **THEN** the tool SHALL return a validation error

## MODIFIED Requirements

### Requirement: Clarify question kinds

The system SHALL support clarify question kinds `single`, `multi`, `text`, `confirm_brief`, and `confirm_agents_md`.

#### Scenario: Kind-specific required fields

- **WHEN** `kind` is `confirm_brief`
- **THEN** `brief` SHALL be required
- **WHEN** `kind` is `confirm_agents_md`
- **THEN** `preview_markdown` SHALL be required and `brief` SHALL be optional
