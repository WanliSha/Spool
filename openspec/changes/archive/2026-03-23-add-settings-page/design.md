# Add Settings Page — Design

## Architecture

```
Rust Backend                        Frontend (Svelte)
────────────                        ─────────────────
settings.rs                         Settings.svelte
  load_settings() ──IPC──▶          On mount: load settings
  save_settings() ◀──IPC──          On change: save settings
  clear_cache()   ◀──IPC──          Clear cache button
  get_cache_stats() ──IPC──▶        Show cache size

Config file location (via dirs crate):
  macOS:   ~/Library/Application Support/Spool/settings.json
  Linux:   ~/.config/spool/settings.json
  Windows: %APPDATA%\Spool\settings.json
```

## Settings Model

```rust
struct AppSettings {
    recursive_folder_loading: bool,  // default: false
    cache_size_limit_mb: u32,        // default: 200
}
```

## UI

```
┌─────────────────────────────────────┐
│  ⚙ Settings                   [✕]  │
│─────────────────────────────────────│
│                                     │
│  File Import                        │
│  ☐ Load subfolders when opening     │
│    a folder                         │
│                                     │
│  Thumbnail Cache                    │
│  Cache size limit: [200] MB         │
│  Current usage: 12.5 MB (250 files) │
│  [Clear Cache]                      │
│                                     │
└─────────────────────────────────────┘
```

## Integration
- Settings gear icon button in the toolbar
- Opens as a modal overlay
- Settings auto-save on change (no save button needed)
- `scan_paths` reads `recursive_folder_loading` from settings
