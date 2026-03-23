# Add RAW Format Support

## Summary
Add RAW format decoding for preview and thumbnail generation using libraw-rs.

## Scope
- Add `libraw-rs` dependency
- RAW decoding for thumbnails (320px)
- RAW decoding for quick preview (2048px)
- RAW decoding for full resolution preview
- Supported formats: CR2, CR3, NEF, ARW, RAF, DNG, ORF, RW2
- Half-size decode mode for quick preview (faster)

## Out of Scope
- RAW editing / processing
- RAW-specific EXIF fields

## References
- Spec: `openspec/specs/core-features/spec.md`
- Spec: `openspec/specs/tech-stack/spec.md`

## Status
Pending
