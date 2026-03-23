# Add Full Resolution Preview — Design

## Approach

Simple toggle between quick (2048px) and full resolution. No tile-based loading for now — send the full image as base64 over IPC. Tauri handles large payloads well.

## Rust Backend

Add `mode` parameter to existing `get_preview`:
- `mode = "quick"` (default): resize to 2048px, cached
- `mode = "full"`: original resolution, encode as JPEG 95 quality, separate cache key

Cache keys: `{path}:quick` and `{path}:full`

## Frontend

- Toggle button in preview toolbar: "Quick" / "Full"
- Default: quick mode
- Click "Full" → loads full resolution (may take 1-3s for large files)
- Show loading indicator during full resolution decode
- Full resolution only for single select (multi-select grid stays quick)
- Mode resets to quick when navigating to another photo
