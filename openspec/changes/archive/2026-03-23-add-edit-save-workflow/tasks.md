# Add Edit & Save Workflow — Tasks

- [x] Add `little_exif` dependency
- [x] Implement EXIF state management module (ExifState, snapshot, undo stack)
- [x] Implement `read_exif` command (read fields, store snapshot)
- [x] Implement `update_exif` command (edit in memory, push undo)
- [x] Implement `undo_exif` command (pop undo stack)
- [x] Implement `reset_exif` and `reset_all_exif` commands
- [x] Implement `save_exif` and `save_all_exif` commands (write to disk)
- [x] Implement `get_modified_files` command
- [x] Update frontend: add EXIF editor panel (click file → show fields)
- [x] Update frontend: edit fields → call update_exif
- [x] Update frontend: Save, Save All, Undo, Reset, Reset All buttons
- [x] Update frontend: modified indicator on file list items
- [x] Verify build and test with real images
