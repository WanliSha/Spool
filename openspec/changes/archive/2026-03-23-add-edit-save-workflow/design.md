# Add Edit & Save Workflow — Design

## Architecture

```
Frontend (Svelte)                         Rust Backend
─────────────────                         ────────────
Select file          ──IPC──▶             read_exif(path)
                                            → little_exif read
                     ◀──IPC──             Return HashMap<String, String>

Edit field           ──IPC──▶             update_exif(path, field, value)
                                            → update in-memory state
                                            → push to undo stack
                     ◀──IPC──             Return updated state + modified flag

Undo                 ──IPC──▶             undo_exif(path)
                                            → pop undo stack
                     ◀──IPC──             Return previous state

Reset                ──IPC──▶             reset_exif(path)
                                            → restore import snapshot
                     ◀──IPC──             Return original state

Reset All            ──IPC──▶             reset_all_exif()

Save                 ──IPC──▶             save_exif(path)
                                            → little_exif write to disk
                                            → update snapshot

Save All             ──IPC──▶             save_all_exif()
```

## Rust State Management

```rust
struct ExifState {
    // Snapshot captured at import time
    snapshot: HashMap<String, String>,
    // Current edited values
    current: HashMap<String, String>,
    // Undo stack: each entry is (field_name, previous_value)
    undo_stack: Vec<(String, Option<String>)>,
    // Whether current differs from snapshot
    modified: bool,
}

// Global state: path -> ExifState
// Managed via tauri::State<Mutex<HashMap<String, ExifState>>>
```

## EXIF Crate
- `little_exif` — pure Rust, read+write, field-level access, supports JPEG/TIFF/PNG/WebP
- 163 ExifTag variants covering camera, lens, dates, GPS, etc.

## Tauri Commands
- `read_exif(path) -> ExifData` — read EXIF, store snapshot, return fields
- `update_exif(path, field, value) -> ExifData` — edit in memory, push undo
- `undo_exif(path) -> ExifData` — pop undo stack
- `reset_exif(path) -> ExifData` — restore to import snapshot
- `reset_all_exif() -> Vec<String>` — reset all, return affected paths
- `save_exif(path)` — write current state to disk
- `save_all_exif() -> Vec<String>` — save all modified, return saved paths
- `get_modified_files() -> Vec<String>` — list files with unsaved changes

## Frontend
- Click file in list → load EXIF fields in editor panel
- Edit fields → call update_exif
- Show modified indicator (dot) on files with unsaved changes
- Toolbar buttons: Save, Save All, Undo, Reset, Reset All
- Keyboard shortcut: Cmd+Z for undo, Cmd+S for save
