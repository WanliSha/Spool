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
│  Metadata, RAW decode, image IO │
└─────────────────────────────────┘
```

## Decisions

| Layer | Choice | Rationale |
|---|---|---|
| **Framework** | Tauri | Rust backend + lightweight WebView frontend; cross-platform (Windows, Linux, macOS) |
| **Frontend** | Svelte | Minimal learning curve for backend engineers; near-zero runtime overhead |
| **Map** | OpenStreetMap + Leaflet.js | Free, no API key, no ToS restrictions, lazy-loaded |
| **Backend language** | Rust | Performance-critical: metadata read/write, RAW decoding, image processing |
| **Image decoding** | `image` crate | Standard formats: JPEG, TIFF, PNG, WebP, BMP |
| **RAW decoding** | `rawler` (pure Rust) | RAW formats: CR2, CR3, NEF, ARW, RAF, DNG, ORF, RW2, PEF. Embedded JPEG extraction for fast preview. |
| **EXIF** | `little_exif` | Pure Rust EXIF read/write with field-level access |
| **XMP** | Custom parser (`quick-xml`) | XML-based metadata read/write in JPEG APP1 / TIFF tag 700 |
| **IPTC** | Custom parser | Binary IPTC-IIM read/write in JPEG APP13 |

## Cross-Platform Notes

- Frontend code is identical across all platforms
- WebView engine varies by OS (WKWebView / WebKitGTK / WebView2) — handled by Tauri
- Platform-specific concerns (file paths, permissions) are in the Rust backend
