## 1. Workspace Restructuring

- [x] 1.1 Create Cargo workspace with three members: `spool-core`, `spool-tauri`, `spool-cli`
- [x] 1.2 Move core metadata logic (`metadata.rs`, `exif.rs`, `xmp.rs`, `iptc.rs`, and supporting modules `decode.rs`, `lib.rs` helpers) into `spool-core` crate, removing Tauri dependencies
- [x] 1.3 Update `spool-tauri` to depend on `spool-core` and verify the GUI builds and runs correctly

## 2. CLI Binary

- [x] 2.1 Create `spool-cli` crate with `clap` derive, depending on `spool-core`
- [x] 2.2 Implement `list` subcommand (directory scan, `--recursive` flag)
- [x] 2.3 Implement `get` subcommand (all fields, single field, `--json` flag)
- [x] 2.4 Implement `set` subcommand (single field, `--json` for multiple fields)
- [x] 2.5 Implement date normalization logic (`YYYY-MM-DD` → `YYYY:MM:DD 12:00:00`, `YYYY-MM-DD HH:MM` → `YYYY:MM:DD HH:MM:00`)

## 3. Claude Code Skill

- [x] 3.1 Write skill file at `.claude/skills/spool-cli.md` with CLI command reference, date-from-imprint workflow, GPS annotation workflow, and default time rule

## 4. Testing

- [x] 4.1 Add integration tests for CLI: `list`, `get`, `set` on sample image files
- [x] 4.2 Add unit tests for date normalization
- [x] 4.3 Manual test: use Claude Code with the skill to annotate a photo with date and GPS
