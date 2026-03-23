# Add Photo Preview — Tasks

## Backend
- [x] Add `get_preview` Rust command (decode + resize to 2048px + base64 JPEG)
- [x] Add in-memory preview cache

## Frontend — Layout
- [x] Restructure right side into three panels (preview, editor, map)
- [x] Implement draggable splitters between panels
- [x] Panel close button (✕) on each panel header
- [x] Toolbar toggle buttons for each panel
- [x] No-selection state: show prompt text
- [x] Panel layout adapts when panels are closed

## Frontend — Preview Component
- [x] Create PhotoPreview.svelte component
- [x] Single select: display 2048px image with zoom (scroll) and pan (drag)
- [x] Multi select: horizontal strip with scroll, synchronized zoom
- [x] Double click to reset zoom to fit
- [x] Load preview via get_preview command

## Verify
- [x] Build and test single select preview
- [x] Build and test multi select preview
- [x] Build and test panel resize and toggle
