## ADDED Requirements

### Requirement: Skill declares CLI availability
The skill file SHALL describe all `spool` CLI subcommands, their arguments, and expected output formats so that Claude can invoke them correctly via Bash.

#### Scenario: Claude discovers skill
- **WHEN** Claude starts a conversation in a project with the skill installed
- **THEN** Claude knows the `spool` CLI is available and what commands it supports

### Requirement: Date-from-imprint workflow
The skill SHALL instruct Claude to use its vision capabilities to read date stamps printed on film photos, confirm with the user, and write the dates via the CLI.

#### Scenario: Clear date stamp on single photo
- **WHEN** user asks Claude to read the date from a photo and the imprint is clearly visible
- **THEN** Claude reads the photo, identifies the date, confirms with the user, and runs `spool set <file> DateTaken <date>`

#### Scenario: Unclear date stamp
- **WHEN** Claude cannot confidently read a date stamp (partially obscured, faded)
- **THEN** Claude reports what it can see and asks the user to confirm or provide the correct date

#### Scenario: Batch date recognition
- **WHEN** user asks Claude to read dates from multiple photos
- **THEN** Claude reads all photos, presents a summary table of recognized dates with confidence indicators, and waits for user confirmation before writing

### Requirement: Default time for date-only values
The skill SHALL instruct Claude to use `12:00:00` as the default time when only a date is identified from a photo imprint or when the user provides only a date.

#### Scenario: Date recognized without time
- **WHEN** Claude reads a date stamp that shows only a date (e.g., `24.12.25`)
- **THEN** Claude uses `12:00` as the time component (the CLI normalizes to `12:00:00`)

### Requirement: GPS annotation workflow
The skill SHALL instruct Claude to resolve place names to GPS coordinates using its own knowledge and write them via the CLI.

#### Scenario: Known location
- **WHEN** user says "these photos were taken at Taipei 101"
- **THEN** Claude resolves the coordinates, confirms with the user, and runs `spool set <file> --json '{"GPSLatitude":"25.0340","GPSLongitude":"121.5645","GPSLatitudeRef":"N","GPSLongitudeRef":"E"}'`

#### Scenario: Ambiguous location
- **WHEN** user provides an ambiguous location name (e.g., "Springfield")
- **THEN** Claude asks the user to clarify which location they mean before proceeding
