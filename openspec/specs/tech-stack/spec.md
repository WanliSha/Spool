# Tech Stack Spec

## Architecture

```
┌─────────────────────────────────┐
│  Frontend (Svelte + Leaflet.js) │
│  WebView rendering layer        │
├─────────────────────────────────┤
│  Tauri IPC bridge               │
├─────────────────────────────────┤
│  Rust Backend                   │
│  EXIF, RAW decode, image I/O   │
└─────────────────────────────────┘
```

## Decisions

| Layer | Choice | Rationale |
|---|---|---|
| **Framework** | Tauri | Rust backend + lightweight WebView frontend; cross-platform (Windows, Linux, macOS) |
| **Frontend** | Svelte | Minimal learning curve for backend engineers; near-zero runtime overhead; UI needs are simple (forms, grid, image viewer) |
| **Map** | OpenStreetMap + Leaflet.js | Free, no API key, no ToS restrictions, lazy-loaded when user opens map picker |
| **Backend language** | Rust | Performance-critical: EXIF read/write, RAW decoding, image processing |
| **Image decoding** | `image` crate | Standard formats: JPEG, TIFF, PNG, WebP, BMP |
| **RAW decoding** | `libraw-rs` (C binding) | All RAW formats: CR2/CR3, NEF, ARW, RAF, DNG, ORF, RW2, etc. |

## Frontend Scope

The frontend is a thin display layer. All heavy processing happens in Rust.

- Photo grid / list view
- EXIF edit form (text inputs, selects)
- Photo preview (zoom/pan via CSS transform)
- Leaflet map picker (coordinate selection)
- Toolbar / buttons

## Cross-Platform Notes

- Frontend code is identical across all platforms
- WebView engine varies by OS (WKWebView / WebKitGTK / WebView2) — handled by Tauri
- Platform-specific concerns (file paths, permissions, RAW C bindings) are in the Rust backend
