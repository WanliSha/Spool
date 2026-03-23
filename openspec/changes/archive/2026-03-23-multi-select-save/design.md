# Multi-Select & Save Workflow — Design

## Architecture Changes

### Frontend State (major rework)

```
Before:
  selectedFile: FileEntry | null     (single)
  exifData: ExifData | null          (single file's data)

After:
  selectedPaths: Set<string>         (multi-select)
  mergedFields: { [field]: string }  (common values / "Mixed" / empty)
  mergedSnapshot: { [field]: string } (common snapshot values for saved GPS)
  dirtyFields: Set<string>           (fields touched by user in this editing session)
  undoStack: UndoEntry[]             (global, frontend-managed)
```

### Undo Stack (move to frontend)

The undo stack moves from per-file Rust state to a global frontend stack, because:
- It needs to track multi-file batch operations
- It needs to track save operations
- It's UI-level state, not data-level state

```typescript
type UndoEntry = {
  type: "edit" | "save";
  // For edit: which files/field were changed
  files: string[];
  field?: string;
  previousValues: { [path: string]: string | null };
  // For save: the pre-save snapshots
  previousSnapshots?: { [path: string]: { [field: string]: string } };
};
```

### Rust Backend Changes

- Remove per-file undo_stack from ExifState
- Add `update_exif_batch(paths, field, value)` command
- Add `save_exif_batch(paths)` command
- Add `reset_exif_batch(paths)` command
- Add `get_exif_batch(paths)` command — returns ExifData for each path
- Keep `read_exif` for initial load

### Merged Fields Computation (frontend)

```
Given selectedPaths = [path1, path2, path3]
Load ExifData for each via get_exif_batch

For each field:
  values = [data1.fields[field], data2.fields[field], data3.fields[field]]
  unique = new Set(values.filter(v => v !== undefined))

  if unique.size === 0 → display ""
  if unique.size === 1 → display that value
  if unique.size > 1  → display "Mixed"
```

### Save Flow

```
User clicks Save:
  1. Get selectedPaths that are modified
  2. For each dirty field: update_exif_batch(selectedPaths, field, value)
  3. save_exif_batch(selectedPaths)
  4. Push save entry to undo stack
  5. Clear dirtyFields
  6. Refresh mergedFields
```

## UI Changes

### File List
- Drag handle (≡) always visible on left
- Checkbox appears when >=2 files selected
- Cmd+click toggles selection
- Shift+click selects range
- Single click = single select (deselects others)
- Select All / Deselect All buttons in toolbar

### Editor Panel Header
- Single select: show filename
- Multi select: show "N photos selected"
- Buttons: Undo, Reset, Save

### Editor Fields
- Per-field dirty indicator (● orange dot) on right when touched
- Per-field reset button (✕) on right when touched
- "Mixed" shown as placeholder text in input, not as actual value

### Map
- Single select: same as current
- Multi select, all same GPS: show blue pin at that GPS
- Multi select, mixed/no GPS: no blue pin, default view
