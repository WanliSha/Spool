# Multi-Select & Save Workflow — Tasks

## Backend
- [x] Remove per-file undo_stack from ExifState
- [x] Add `update_exif_batch` command (update field on multiple files)
- [x] Add `save_exif_batch` command (save multiple files)
- [x] Add `reset_exif_batch` command (reset multiple files to snapshot)
- [x] Add `get_exif_batch` command (return ExifData for multiple paths)
- [x] Add `restore_snapshot_batch` command (for undo save: restore previous snapshots)

## Frontend — Selection
- [x] Replace `selectedFile` with `selectedPaths` Set
- [x] Implement Cmd+click toggle selection
- [x] Implement Shift+click range selection
- [x] Add Select All / Deselect All buttons to toolbar
- [x] Show checkbox on file items when multi-selected (>=2)
- [x] Add drag handle (≡) for reordering
- [x] Implement drag-to-reorder file list

## Frontend — EXIF Panel
- [x] Compute merged fields from selected files (common / Mixed / empty)
- [x] Compute merged snapshot for map GPS
- [x] Track dirty fields (per-field touched state)
- [x] Show "Mixed" as placeholder in editable inputs
- [x] Show dirty indicator (● dot) and reset button (✕) per field
- [x] Update field: call update_exif_batch for all selected files
- [x] Reset single field: clear dirty flag, recompute merged value

## Frontend — Undo
- [x] Implement global undo stack (frontend-managed)
- [x] Push edit entries on field change
- [x] Push save entries on save
- [x] Undo edit: restore previous values via update_exif_batch
- [x] Undo save: restore previous snapshots via restore_snapshot_batch

## Frontend — Save / Reset
- [x] Save: save selected modified files via save_exif_batch
- [x] Reset: reset selected files via reset_exif_batch
- [x] Remove Save All / Reset All buttons
- [x] Unsaved changes confirmation on close/clear

## Frontend — Map
- [x] Multi-select: show blue pin only when all selected have same GPS
- [x] Map coordinate selection applies to all selected files

## Verify
- [x] Build and test single select still works
- [x] Build and test multi-select edit + save
- [x] Build and test undo (edit and save)
