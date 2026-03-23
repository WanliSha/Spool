# File Write Spec

## Principle
All edits are staged in-app. Files are only modified when the user explicitly saves.

## Edit Lifecycle

```
Import → Edit in app → Save (write selected files)
                     → Undo (revert last operation, global stack)
                     → Reset (discard unsaved changes on selected files)
```

## In-App State

On import, Rust reads and stores an metadata snapshot per file (in memory). All user edits modify only the in-app state, never the file on disk.

## Selection

- Single click to select one file
- Cmd+click to toggle additional files
- Shift+click to select a range
- Select All / Deselect All buttons in toolbar
- Checkbox appears on each file when multi-selected (>=2)
- Drag handle always visible for reordering within the app

## EXIF Panel (Multi-Select)

- All same value → display that value
- Different values → display "Mixed"
- All empty → display empty
- Track which fields the user has **touched** (dirty flag per field)
- Only touched fields are written on save; untouched fields are skipped
- Touched field with value → write that value to all selected files
- Touched field cleared → delete that field from all selected files
- Visual indicator on edited fields: orange dot (●) + reset button (✕) on the right

## Undo

- Global operation stack (not per-file)
- Each entry records: affected files, field, previous values
- Undo restores the previous state of the affected files
- Save is also recorded in the undo stack
- Undo save: restores app state to pre-save, marks files as modified (does NOT rewrite the file)
- Cmd+Z shortcut

## Reset

- Discards unsaved changes on selected files
- Restores to the last saved state (not the import-time state)
- If file was never saved, restores to import-time state

## Save

- Saves all selected files that have unsaved modifications
- Writes touched fields to disk
- Updates snapshot to current state
- No separate "Save All" — save always operates on selection

## Unsaved Changes Indicator

- Files with unsaved modifications visually marked in file list (orange dot)
- Closing the app or clearing files with unsaved changes prompts confirmation:
  "You have unsaved changes. Save before closing?" [Save] [Discard] [Cancel]

## Map (Multi-Select)

- All selected files have same GPS → show blue pin (Saved)
- GPS differs or mixed → no blue pin
- User selects coordinate (red pin) → applies to all selected files on save

## Write Method
- Direct write to original file (modify EXIF header in place)
- No backup copies, no sidecar files
- EXIF modification does not alter image pixel data
