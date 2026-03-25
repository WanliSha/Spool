## Why

Spool's metadata engine (read/write EXIF, XMP, IPTC) is currently only accessible through the Tauri GUI. Film photographers often need to batch-annotate scanned photos with dates (read from date stamps printed on film) and GPS coordinates. By exposing the metadata engine as a standalone CLI binary and pairing it with a Claude Code skill, users can leverage Claude's vision capabilities to read date imprints from photos and natural language understanding to resolve place names to coordinates — then write metadata directly via the CLI.

## What Changes

- Add a `spool` CLI binary that shares the existing Rust core (`metadata::read_all`, `metadata::write_all`) without Tauri dependencies
- CLI subcommands: `get <file> [field]`, `set <file> <field> <value>`, `list <dir>`
- Date values accept partial input (`2024-12-25`) and store in EXIF format (`2024:12:25 12:00:00`), with optional time precision to the minute
- Add a Claude Code skill that teaches Claude how to use the CLI, including the date-from-photo-imprint workflow and GPS annotation workflow

## Capabilities

### New Capabilities
- `cli`: Standalone CLI binary exposing Spool's metadata read/write engine
- `claude-skill`: Claude Code skill for AI-assisted metadata annotation (date recognition from film imprints, GPS from place names)

### Modified Capabilities
_(none — the CLI reuses existing core logic without changing its behavior)_

## Impact

- **Code**: Extract core metadata functions from `src-tauri/src/` into a shared library; add a new `src-tauri/src/bin/spool.rs` (or separate crate) for the CLI entry point
- **Dependencies**: May need `clap` for CLI argument parsing; no new metadata dependencies
- **Build**: Cargo workspace may need restructuring to support both the Tauri app and the standalone CLI binary
- **Distribution**: CLI binary can be built independently of Tauri; skill file lives in project repo or user's `~/.claude/` directory
