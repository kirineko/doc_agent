## ADDED Requirements

### Requirement: Slash command kind

The slash command registry SHALL support entries with `kind: "command"` distinct from `kind: "template"`.

#### Scenario: Template entry behavior unchanged

- **WHEN** the user selects a slash entry with `kind: "template"`
- **THEN** the composer SHALL be filled with the template prompt only
- **AND** SHALL NOT auto-send a message

#### Scenario: Command entry sends on submit

- **WHEN** the user selects a slash entry with `kind: "command"` and submits (Enter)
- **THEN** the application SHALL send the composer text as a normal user message via `send_message`
- **AND** SHALL NOT replace the message with a hidden template prompt

### Requirement: Init slash command registration

The registry SHALL include a command entry for project profile initialization.

#### Scenario: Init command metadata

- **WHEN** the slash menu is loaded
- **THEN** an entry with id `init`, `kind: "command"`, label describing project agent configuration, and keywords including `init` and `agents` SHALL be present

#### Scenario: Init accepts optional tail

- **WHEN** the user types `/init 固化PPT风格` and submits
- **THEN** the full string SHALL be sent as the user message content

## MODIFIED Requirements

### Requirement: Slash command registry

The application SHALL maintain a slash command registry for workspace chat input, supporting both `template` and `command` kinds.

#### Scenario: Registry exposes kind

- **WHEN** frontend code reads a slash entry
- **THEN** each entry SHALL declare `kind` as either `template` or `command`
