## Context

Spool's metadata engine lives in `src-tauri/src/` with core functions `metadata::read_all(path)` and `metadata::write_all(path, fields)` that are pure logic — they take a `Path` and `HashMap<String, String>`, no Tauri dependency. The Tauri commands in `metadata.rs`, `exif.rs` etc. are thin wrappers that add state management (snapshot/current for undo). The CLI needs only the pure functions.

Current Cargo setup: single crate `spool` with `lib` + `bin` targets, Tauri as a dependency at crate level.

## Goals / Non-Goals

**Goals:**
- Standalone CLI binary that reads/writes photo metadata using Spool's existing engine
- Claude Code skill that teaches Claude the CLI commands and workflows (date-from-imprint, GPS annotation)
- Date input accepts `YYYY-MM-DD` (defaults to `12:00:00`) or `YYYY-MM-DD HH:MM`

**Non-Goals:**
- MCP server protocol — skill + Bash is sufficient
- Geocode API integration — Claude uses its own knowledge for coordinates
- Undo/snapshot mechanism — CLI writes are final
- GUI changes — the Tauri app is unaffected

## Decisions

### 1. Crate structure: extract a `spool-core` library crate

**Choice:** Create a Cargo workspace with `spool-core` (pure library), `spool-tauri` (GUI app), and `spool-cli` (CLI binary).

**Why:** The current crate has `tauri` as a top-level dependency. A CLI binary in the same crate would still pull in Tauri. Extracting core logic into `spool-core` gives both consumers a clean dependency.

**Alternative considered:** Adding a `[[bin]]` target to the existing crate with `#[cfg(feature = "cli")]` — messier, still compiles Tauri unless feature-gated throughout.

### 2. CLI framework: `clap` derive

**Choice:** Use `clap` with derive macros for argument parsing.

**Why:** Standard Rust CLI library, minimal overhead, good error messages. The command surface is small (`get`, `set`, `list`) so anything heavier is unnecessary.

### 3. CLI command design

```
spool list <path>                        # list supported image files
spool get <file> [field]                 # read metadata (all or specific field)
spool set <file> <field> <value>         # write a single field
spool set <file> --json '{"k":"v",...}'  # write multiple fields at once
```

**Why `set` with `--json`:** Batch GPS requires setting 4 fields (lat, lng, lat-ref, lng-ref). One command is better than four.

### 4. Date normalization in CLI

The CLI normalizes date inputs before passing to the core engine:
- `2024-12-25` → `2024:12:25 12:00:00`
- `2024-12-25 14:30` → `2024:12:25 14:30:00`
- `2024:12:25 14:30:00` → passed through as-is

This lives in the CLI layer, not in `spool-core` — the core engine stores whatever string it receives.

### 5. Skill file location

**Choice:** Ship as a file in the Spool repo at `.claude/skills/spool-cli.md`, so it's available when working in the Spool project directory. Users can also copy it to `~/.claude/skills/` for global availability.

**Why:** No plugin infrastructure needed. A single markdown file is all Claude Code requires.

## Risks / Trade-offs

- **Workspace restructuring breaks existing build** → Mitigation: Keep all existing code paths working; `spool-tauri` replaces the current crate with same dependencies and entry point.
- **CLI binary size** → The `image` and `rawler` crates are large. If `spool-core` pulls them in for decode support, the CLI binary will be ~20MB+. Acceptable for a local tool.
- **Claude vision accuracy on date imprints** → Some film date stamps are faded or partially obscured. The skill instructs Claude to flag uncertain readings and ask the user. This is a UX concern, not a technical risk.
