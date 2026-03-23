# Add Crown & Flint Import — Design

## Architecture

```
+page.svelte
  │
  ├── Normal mode (current)
  │     file list + preview + editor + map
  │
  └── C&F Import mode (new)
        CfImport.svelte (full-page component)
          ├── Left panel: scan photos
          │     Import folder → thumbnails list
          │     Drag reorder, reverse, delete
          │
          ├── Right panel: C&F data
          │     Import folder → parse film.json + load previews
          │     Drag reorder, reverse, delete
          │
          └── Confirm & Write
                Map JSON fields → unified keys
                Write metadata to each matched scan photo
                Switch to normal mode with photos loaded
```

## Rust Backend

### New command: `parse_cf_json(path) -> Vec<CfEntry>`
- Read film.json from the folder
- Parse and return structured entries
- Include ImageNumber for preview image matching

### New command: `write_metadata_from_cf(pairs: Vec<{photo_path, cf_entry}>)`
- For each pair, map C&F fields to unified keys
- Write via metadata::write_all

### Thumbnail reuse
- Scan photos: use existing get_thumbnail command
- C&F previews: read {ImageNumber}.jpeg directly, return as base64

## Frontend — CfImport.svelte

### State
- `scanPhotos`: array of {path, filename, thumbnail}
- `cfEntries`: array of {index, data, preview}
- `mode`: "setup" | "align"

### Left panel
- [Import Folder] button → load scan photos, generate thumbnails
- [Reverse] [Clear] buttons
- Each item: thumbnail + filename + [✗] delete + drag handle

### Right panel
- [Import Folder] button → parse film.json, load preview JPEGs
- [Reverse] [Clear] buttons
- Each item: preview (or placeholder) + metadata summary + [✗] delete + drag handle

### Alignment
- Left and right scroll synchronized
- Same row = matched pair
- Unmatched (one side longer) shows empty space
- Footer shows: "Matched: N pairs"

### Confirm & Write
- Only matched pairs (both sides have an item) are written
- Write metadata to scan photo files
- Switch to normal Spool mode with scan photos loaded
