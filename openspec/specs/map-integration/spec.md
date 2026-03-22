# Map Integration Spec

## Overview
Provide a map-based coordinate picker for editing GPS EXIF fields, using OpenStreetMap as the tile source and Leaflet.js as the interactive map library.

## Decision: OSM over Google Maps

| Factor | Google Maps | OpenStreetMap |
|---|---|---|
| Cost | API key + usage fees | Free |
| ToS | Grey area for desktop apps | Open license |
| Auth | Requires Google OAuth | None |
| Quality | Excellent | Good (sufficient for coordinate picking) |
| Dependencies | Requires WebView + API key management | Leaflet.js only |

Chosen: **OpenStreetMap + Leaflet.js** — free, no API key, no login, no ToS restrictions.

## Behaviour

### Loading
- Lazy-loaded: map is NOT initialized at app startup
- Map loads only when user opens the coordinate picker
- First load requires network; subsequent use reuses the initialized instance

### Interactions
- Pan (drag to move)
- Zoom (scroll wheel / pinch / buttons)
- Click to place pin and select coordinates
- Search location by name (via Nominatim geocoding API)
- Drag pin to adjust coordinates

### Data Flow
```
User clicks on map
  → Leaflet returns lat/lng
  → Sent to Rust backend via Tauri IPC
  → Written to EXIF GPS fields

Photo has existing GPS
  → Rust reads EXIF GPS fields
  → Sends lat/lng to frontend
  → Leaflet centers map and places pin
```

### Batch Apply
- User selects coordinates on map
- Can apply to multiple selected photos (via batch edit)

## Technical Notes
- Leaflet.js runs in the same WebView as the Svelte frontend — no extra WebView needed
- Nominatim usage policy: max 1 request/second, include app User-Agent
- Tile source: `https://tile.openstreetmap.org/{z}/{x}/{y}.png`
