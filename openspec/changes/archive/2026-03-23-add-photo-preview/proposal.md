# Add Photo Preview

## Summary
Rework the UI layout into resizable panels and add photo preview with quick mode (2048px).

## Scope
- UI layout: file list (left), right side splits into preview (top), editor (bottom-left), map (bottom-right)
- All panel borders draggable to resize
- Panels can be closed (✕) and reopened (toolbar toggle buttons)
- No selection state: show prompt text
- Rust command: `get_preview(path)` returns ~2048px JPEG base64
- Single select: full preview with zoom (scroll) and pan (drag)
- Multi select: horizontal strip of previews, scroll to slide, synchronized zoom
- Preview cached in memory for the session

## Out of Scope
- Full resolution preview (separate change)
- RAW format preview (separate change)

## References
- Spec: `openspec/specs/core-features/spec.md`

## Status
Pending
