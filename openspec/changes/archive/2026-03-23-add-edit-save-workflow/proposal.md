# Add Edit & Save Workflow

## Summary
Implement the in-app edit staging, undo, revert, and save workflow for EXIF modifications.

## Scope
- EXIF snapshot on import (store original state per file in memory)
- In-app edit state management (edits don't touch files)
- Undo: revert last edit per file
- Reset: restore single file to import snapshot
- Reset All: restore all files to import snapshot
- Save: write single file EXIF to disk
- Save All: write all modified files to disk
- Unsaved changes indicator in UI
- Confirmation prompt on close/remove with unsaved changes

## Out of Scope
- Specific EXIF field editing UI (separate change)
- Batch edit logic (separate change)

## References
- Spec: `openspec/specs/file-write/spec.md`

## Status
Pending
