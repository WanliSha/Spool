# Add Full Resolution Preview

## Summary
Add a full resolution preview mode that decodes the image at original size, with tile-based loading for large images.

## Scope
- Rust command: `get_preview(path, mode)` with mode "full"
- Full decode of original image
- Tile-based loading for images > 4096px (load visible area on demand)
- UI toggle between quick and full resolution
- Performance: keep quick preview as default, full resolution on user request

## Out of Scope
- RAW format support (separate change)

## References
- Spec: `openspec/specs/core-features/spec.md`

## Status
Pending
