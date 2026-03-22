# Add File Import — Design

## Architecture

```
Frontend (Svelte)                    Backend (Rust)
─────────────────                    ──────────────
Drag & drop zone          ──IPC──▶   scan_files(paths, recursive)
File picker button         ──IPC──▶   open_file_dialog() / open_folder_dialog()
                                      │
                                      ▼
                                    Filter by supported extensions
                                    Return Vec<FileEntry> { path, filename, size }
                                      │
                          ◀──IPC──    │
Display file list
```

## Rust Backend

### Tauri Commands
- `open_files_dialog()` — opens native file picker, returns selected file paths
- `open_folder_dialog()` — opens native folder picker, returns folder path
- `scan_paths(paths: Vec<String>, recursive: bool)` — scans paths (files or folders), filters by supported extensions, returns file entries

### Supported Extensions
```rust
const SUPPORTED_EXTENSIONS: &[&str] = &[
    // Standard
    "jpg", "jpeg", "tif", "tiff", "png", "webp", "bmp",
    // RAW
    "cr2", "cr3", "nef", "arw", "raf", "dng", "orf", "rw2",
];
```

### FileEntry Struct
```rust
#[derive(Serialize)]
struct FileEntry {
    path: String,
    filename: String,
    size: u64,
}
```

## Svelte Frontend

### Drop Zone
- Full window drop zone (dragover/drop events)
- Visual feedback on drag hover
- Extract file paths from drop event, send to `scan_paths`

### File Picker
- "Open Files" button → `open_files_dialog()`
- "Open Folder" button → `open_folder_dialog()`

### File List Display
- Simple list showing filename and path
- No thumbnails yet (separate change)

## Tauri Plugins
- `tauri-plugin-dialog` — native file/folder picker dialogs

## Permissions
- `dialog:default` — for file/folder picker
- `core:default` — already included
- Drag and drop works via standard web APIs, needs `tauri.conf.json` drag-drop config
