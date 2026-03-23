# Rework Preview Navigation

## Summary
Change preview from static display to a primary browsing interface with swipe navigation, rotation, and context-aware behavior for single vs multi-select.

## Scope

### Single Select
- Preview shows current photo, swipe left/right to navigate through all photos
- Swiping updates file list selection, editor, and map to match
- Zoom (pinch / +- buttons), pan (drag), double-click to reset
- Rotate 90° CW/CCW buttons (visual only, does not modify file)
- Show filename and position indicator (e.g. "photo3.tif (3/36)")

### Multi Select
- Preview shows only selected photos, swipe to browse within selection
- Show filename and position within selection (e.g. "photo3.tif (2/5 selected)")
- Editor and map always show shared values (do NOT follow preview swipe)
- Zoom and pan available, no rotate, no single-photo editing
- No rotation buttons

### Zoom Gesture Fix
- Trackpad pinch (ctrlKey wheel) = zoom
- Trackpad two-finger scroll = swipe left/right to navigate
- Mouse scroll = zoom
- +/- buttons as alternative

## Out of Scope
- Full resolution preview (separate change)
- Modifying EXIF orientation on rotate (separate change)

## Status
Pending
