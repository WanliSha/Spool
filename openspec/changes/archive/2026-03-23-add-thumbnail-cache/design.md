# Add Thumbnail Cache — Design

## Architecture

```
Frontend                              Rust Backend
────────                              ────────────
Request thumbnail     ──IPC──▶        get_thumbnail(path)
for file path                            │
                                         ├─ Check cache (hash of path + mtime)
                                         │   ├─ Hit → read cached JPEG
                                         │   └─ Miss → decode image → resize to 320px
                                         │              → save to cache → return
                                         │
Display <img>         ◀──IPC──        Return base64 JPEG string
```

## Rust Implementation

### Cache Directory
Use `dirs` crate for OS-standard paths:
- macOS: `~/Library/Caches/Spool/thumbnails/`
- Linux: `~/.cache/spool/thumbnails/`
- Windows: `%LOCALAPPDATA%\Spool\Cache\thumbnails\`

### Cache Key
`SHA256(file_path + modification_time_secs)` → used as filename: `{hash}.jpg`

### Thumbnail Generation
1. Open image with `image` crate
2. Resize to fit 320px long edge (maintain aspect ratio)
3. Encode as JPEG quality 80
4. Save to cache directory
5. Return as base64 string

### LRU Eviction
- On startup or when adding new entry, check total cache size
- If over 200MB limit, delete oldest files (by file mtime) until under limit

### Tauri Commands
- `get_thumbnail(path: String) -> String` — returns base64 JPEG
- `get_cache_stats() -> CacheStats` — returns size, count (for future settings page)
- `clear_cache()` — deletes all cached thumbnails

## Frontend
- File list items show thumbnail instead of just filename
- Thumbnails loaded lazily as items scroll into view
- Base64 string displayed via `<img src="data:image/jpeg;base64,{data}">`

## Dependencies
- `dirs` crate — OS cache directory
- `sha2` crate — cache key hashing
- `base64` crate — encoding thumbnails for IPC
