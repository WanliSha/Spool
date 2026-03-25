## ADDED Requirements

### Requirement: List supported image files
The CLI SHALL provide a `list` subcommand that scans a directory and outputs paths of supported image files (JPEG, TIFF, PNG, WebP, BMP, and RAW formats: CR2, CR3, NEF, ARW, RAF, DNG, ORF, RW2).

#### Scenario: List files in a directory
- **WHEN** user runs `spool list <directory>`
- **THEN** the CLI outputs one file path per line for each supported image file in the directory

#### Scenario: List files recursively
- **WHEN** user runs `spool list <directory> --recursive`
- **THEN** the CLI outputs supported image files from the directory and all subdirectories

#### Scenario: No supported files found
- **WHEN** user runs `spool list <directory>` and the directory contains no supported files
- **THEN** the CLI outputs nothing and exits with code 0

### Requirement: Read metadata from a file
The CLI SHALL provide a `get` subcommand that reads unified metadata (EXIF + XMP + IPTC) from a photo file and outputs it as key-value pairs.

#### Scenario: Read all metadata
- **WHEN** user runs `spool get <file>`
- **THEN** the CLI outputs all metadata fields as `Key: Value` lines, one per field

#### Scenario: Read a specific field
- **WHEN** user runs `spool get <file> <field>`
- **THEN** the CLI outputs only the value of the requested field

#### Scenario: Read with JSON output
- **WHEN** user runs `spool get <file> --json`
- **THEN** the CLI outputs all metadata as a JSON object

#### Scenario: File not found
- **WHEN** user runs `spool get <nonexistent-file>`
- **THEN** the CLI prints an error to stderr and exits with a non-zero code

### Requirement: Write metadata to a file
The CLI SHALL provide a `set` subcommand that writes a metadata field to a photo file, persisting across EXIF, XMP, and IPTC formats as applicable.

#### Scenario: Set a single field
- **WHEN** user runs `spool set <file> <field> <value>`
- **THEN** the field is written to the file and the CLI exits with code 0

#### Scenario: Set multiple fields via JSON
- **WHEN** user runs `spool set <file> --json '{"Field1":"val1","Field2":"val2"}'`
- **THEN** all specified fields are written to the file

#### Scenario: Unsupported file format
- **WHEN** user runs `spool set <file> <field> <value>` on a non-image file
- **THEN** the CLI prints an error to stderr and exits with a non-zero code

### Requirement: Date input normalization
The CLI SHALL normalize date inputs for date fields (DateTaken, DateCreated, DateModified) into EXIF-compatible format.

#### Scenario: Date only input
- **WHEN** user runs `spool set <file> DateTaken "2024-12-25"`
- **THEN** the value is stored as `2024:12:25 12:00:00`

#### Scenario: Date with time to minute precision
- **WHEN** user runs `spool set <file> DateTaken "2024-12-25 14:30"`
- **THEN** the value is stored as `2024:12:25 14:30:00`

#### Scenario: Full datetime input
- **WHEN** user runs `spool set <file> DateTaken "2024-12-25 14:30:00"`
- **THEN** the value is stored as `2024:12:25 14:30:00`

#### Scenario: EXIF-format input passthrough
- **WHEN** user runs `spool set <file> DateTaken "2024:12:25 14:30:00"`
- **THEN** the value is stored as-is
