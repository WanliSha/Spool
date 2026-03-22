# Add Map Picker

## Summary
Implement the OSM + Leaflet.js coordinate picker in the Svelte frontend, with Tauri IPC to pass coordinates to the Rust backend.

## Scope
- Add Leaflet.js dependency to frontend
- Map picker component (pan, zoom, click to select, pin drag)
- Location search via Nominatim
- Tauri IPC: send selected coordinates to Rust backend
- Tauri IPC: receive existing GPS from Rust to show current pin
- Lazy load map on user action

## Out of Scope
- EXIF GPS write logic (handled by core EXIF editing change)
- Offline map tiles / caching

## References
- Spec: `openspec/specs/map-integration/spec.md`

## Status
Pending
