## ADDED Requirements

### Requirement: Init command blocked during pending clarify

The workspace UI SHALL prevent submitting `/init` while a clarify question is pending for the active session.

#### Scenario: Slash init disabled with pending card

- **WHEN** `clarify_pending` is set for the active session
- **AND** the user attempts to run the `init` slash command
- **THEN** the UI SHALL show an error or disabled state explaining that clarification must be completed first
- **AND** SHALL NOT call `send_message`

#### Scenario: Composer init prefix guarded

- **WHEN** the user manually types a message starting with `/init` while clarify is pending
- **THEN** send SHALL be blocked client-side with the same message
- **AND** if bypassed, the backend error from `send_message` SHALL be surfaced

### Requirement: Confirm agents markdown clarify UI

The clarify card SHALL render `confirm_agents_md` questions with a scrollable full-text Markdown preview of `preview_markdown`.

#### Scenario: Preview displays full proposed body

- **WHEN** a pending clarify question has `kind` `confirm_agents_md`
- **THEN** the card SHALL render `preview_markdown` as Markdown inside a scrollable region
- **AND** SHALL provide confirm and reject actions consistent with other confirm-style clarify kinds

#### Scenario: Optional changelog hint

- **WHEN** `changelog_summary` is present on a `confirm_agents_md` question
- **THEN** the card SHALL display it as supplementary text above or below the preview
