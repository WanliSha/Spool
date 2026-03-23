# Add Crown & Flint Import — Tasks

## Backend
- [x] Add `parse_cf_json` command (read film.json, return entries)
- [x] Add `get_cf_preview` command (read {ImageNumber}.jpeg as base64)
- [x] Add `write_cf_metadata` command (write matched pairs to files)

## Frontend — CfImport Component
- [x] Create CfImport.svelte full-page component
- [x] Left panel: import scan folder, show thumbnails
- [x] Right panel: import C&F folder, parse JSON, show previews
- [x] Both panels: drag reorder, reverse, delete, clear
- [x] Synchronized scroll for alignment
- [x] Match counter in footer
- [x] Error handling: missing film.json, invalid format, missing previews

## Frontend — Integration
- [x] Add "Import from Crown & Flint" button on drop zone page
- [x] Toggle between normal mode and C&F import mode
- [x] On confirm: write metadata, switch to normal mode with photos loaded

## Verify
- [x] Build and test
