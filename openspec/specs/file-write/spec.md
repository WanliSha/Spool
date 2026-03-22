# File Write Spec

## Principle
All edits are staged in-app. Files are only modified when the user explicitly saves.

## Edit Lifecycle

```
Import → Edit in app → Save / Save All (write to file)
                     → Undo (revert last edit)
                     → Reset / Reset All (restore to import state)
```

## In-App State

On import, Rust reads and stores an EXIF snapshot per file (in memory). All user edits modify only the in-app state, never the file on disk.

## Single File Operations

### Undo
- Reverts the last single edit operation on the selected file
- Standard undo stack per file (Ctrl+Z / Cmd+Z)

### Reset
- Restores all fields of the selected file to the state captured at import time
- Discards entire undo history for that file

### Save
- Writes the current in-app EXIF state of the selected file to disk
- Updates the import snapshot to the newly saved state
- Clears the undo history (saved state becomes the new baseline)

## Batch Operations

### Reset All
- Restores all files to their import-time state
- Discards undo history for all files

### Save All
- Saves all files that have unsaved modifications
- Same behaviour as single Save, applied to each modified file

## Unsaved Changes Indicator
- Files with unsaved modifications should be visually marked in the UI
- If user closes the app or removes files with unsaved changes, prompt for confirmation

## Write Method
- Direct write to original file (modify EXIF header in place)
- No backup copies, no sidecar files
- EXIF modification does not alter image pixel data
