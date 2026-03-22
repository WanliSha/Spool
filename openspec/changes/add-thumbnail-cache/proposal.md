# Add Thumbnail Cache

## Summary
Implement disk-based thumbnail caching in the Rust backend.

## Scope
- Generate 320px JPEG thumbnails in Rust
- Cache to OS-standard cache directory
- Cache key: hash of file path + modification time
- LRU eviction when cache exceeds size limit
- Expose cache stats and clear action for settings page

## Out of Scope
- Settings UI (separate change)

## References
- Spec: `openspec/specs/thumbnail-cache/spec.md`

## Status
Pending
