# Multi-Select & Save Workflow

## Summary
Replace single-select with multi-select support. Rework save/undo/reset to operate on selection. EXIF panel shows common values across selected files with per-field dirty tracking.

## Scope
- Multi-select: Cmd+click toggle, Shift+click range, Select All / Deselect All
- Checkbox shown on files when multi-selected
- Drag handle for reordering files in list
- EXIF panel: common value / "Mixed" / empty display for multi-select
- Per-field dirty tracking (only touched fields written on save)
- Per-field edit indicator (● dot) and reset button (✕)
- Global undo stack (including undo save)
- Save = save selected modified files (remove Save All)
- Reset = discard unsaved changes on selected files (back to last save)
- Map: show blue pin only when all selected files have same GPS
- Unsaved changes confirmation on close

## Out of Scope
- Equipment templates (separate change)
- Photo preview improvements (separate change)

## References
- Spec: `openspec/specs/file-write/spec.md`

## Status
Pending
