# Add Map Picker — Design

## Approach
Add a map picker modal that opens when clicking a "Pick on Map" button in the GPS section of the EXIF editor. Uses Leaflet.js (already installed) with OSM tiles.

## UI Flow

```
EXIF Editor Panel
  GPS Latitude:  25.0228       [📍 Pick on Map]  ← button
  GPS Longitude: 121.5411

         │ click
         ▼

┌──────────────────────────────────────┐
│  Map Picker (modal overlay)          │
│  ┌──────────────────────────┐       │
│  │ [Search: _______________]│       │
│  │                          │       │
│  │     OSM Map              │       │
│  │        📍                │       │
│  │                          │       │
│  └──────────────────────────┘       │
│  Lat: 25.0228  Lng: 121.5411       │
│  [Apply]  [Cancel]                   │
└──────────────────────────────────────┘
```

## Components

### MapPicker.svelte
- New Svelte component
- Props: `lat`, `lng` (initial position, nullable)
- Events: `onapply({ lat, lng })`, `oncancel`
- Leaflet map initialized on mount
- Click on map → place/move marker → update coordinates
- Draggable marker
- Search input → Nominatim geocoding → pan to result

### Integration with EXIF Editor
- "Pick on Map" button shown in the GPS fields section
- On apply: calls `update_exif` for GPSLatitude, GPSLongitude, GPSLatitudeRef, GPSLongitudeRef
- Existing GPS coordinates pre-populate the map pin

## Leaflet Setup
- Import `leaflet/dist/leaflet.css` for styling
- OSM tile layer: `https://tile.openstreetmap.org/{z}/{x}/{y}.png`
- Attribution required by OSM
- Default view: world (zoom 2) if no existing GPS, or centered on existing GPS (zoom 15)

## Nominatim Search
- Endpoint: `https://nominatim.openstreetmap.org/search`
- Params: `q={query}&format=json&limit=5`
- User-Agent header: `Spool/0.1.0`
- Rate limit: max 1 request/second (debounce input)
