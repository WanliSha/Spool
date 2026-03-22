# Thumbnail Cache Spec

## Overview
Cache generated thumbnails to disk so repeated access to the same files is instant.

## Cache Location
- Use OS-standard cache directory:
  - macOS: `~/Library/Caches/Spool/thumbnails/`
  - Linux: `~/.cache/spool/thumbnails/`
  - Windows: `%LOCALAPPDATA%\Spool\Cache\thumbnails\`

## Thumbnail Format
- JPEG, ~320px long edge
- ~20-50 KB per thumbnail

## Cache Key
- Hash of: file path + file modification time
- If the original file is modified, the cache entry is automatically invalidated

## Cache Size
- Default limit: 200 MB
- User-configurable in settings
- When limit is exceeded, evict oldest entries (LRU)
- Manual "Clear cache" button in settings

## Generation
- Thumbnails are generated in Rust backend on first access
- Sent to frontend as needed
- Cached to disk for future sessions
