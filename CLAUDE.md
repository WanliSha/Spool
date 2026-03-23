# Spool

Cross-platform photo metadata editor built for film photographers.
Supports EXIF, XMP, and IPTC — unified UI, automatic sync across formats.
Rust (Tauri) + Svelte + Leaflet.js (OpenStreetMap).

## Project Context
- Specs: `openspec/specs/`
- Changes: `openspec/changes/`
- Roadmap: `openspec/ROADMAP.md`
- OpenSpec config: `openspec/config.yaml`

## Tech Stack
- Backend: Rust (Tauri)
- Frontend: Svelte
- Map: OpenStreetMap + Leaflet.js
- Image decoding: `image` crate + `rawler` (pure Rust RAW)
- Metadata: `little_exif` (EXIF) + custom XMP/IPTC parsers
- License: GPLv3 + Commercial dual licensing

## Conventions
- Documentation and code in English
- Use OpenSpec workflow: specs for decisions, changes for implementation
