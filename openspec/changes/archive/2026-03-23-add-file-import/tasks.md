# Add File Import — Tasks

- [x] Add `tauri-plugin-dialog` dependency (Rust + frontend)
- [x] Implement `scan_paths` Tauri command (filter supported extensions, return FileEntry list)
- [x] Implement `open_files_dialog` and `open_folder_dialog` (via @tauri-apps/plugin-dialog frontend API)
- [x] Enable drag-and-drop in `tauri.conf.json`
- [x] Replace scaffold UI with drop zone and file picker buttons
- [x] Handle drag-and-drop events in Svelte, call `scan_paths`
- [x] Display imported file list (filename + path)
- [x] Verify: drag files, drag folder, file picker all work
