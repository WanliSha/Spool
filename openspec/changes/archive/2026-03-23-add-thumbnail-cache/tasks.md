# Add Thumbnail Cache — Tasks

- [x] Add Rust dependencies: `dirs`, `sha2`, `base64`
- [x] Implement cache directory creation and path resolution
- [x] Implement cache key generation (SHA256 of path + mtime)
- [x] Implement thumbnail generation (decode + resize to 320px + encode JPEG)
- [x] Implement `get_thumbnail` Tauri command with cache read/write
- [x] Implement LRU eviction (delete oldest when over 200MB)
- [x] Implement `get_cache_stats` and `clear_cache` commands
- [x] Update frontend: show thumbnails in file list
- [x] Verify build and test with real images
