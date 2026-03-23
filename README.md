# Spool

Cross-platform photo metadata editor built for film photographers.

Built with Rust (Tauri) + Svelte + OpenStreetMap.

![Spool Editor](docs/Edit-Example.png)

## Features

- **Unified metadata editing** — EXIF, XMP, and IPTC in one interface, auto-synced on write
- **Built for film photographers** — fill in camera, lens, exposure, GPS for scanned photos from scratch
- **Crown & Flint import** — visually align scan photos with C&F app data, one-click metadata write
- **GPS coordinate picker** — OpenStreetMap + Leaflet embedded map with location search
- **Photo preview** — quick (2048px) and 1:1 full resolution with zoom, pan, rotate
- **Multi-select & batch edit** — Cmd+click, Shift+click, per-field dirty tracking
- **RAW format support** — CR2, CR3, NEF, ARW, RAF, DNG, ORF, RW2, PEF
- **Custom metadata fields** — user-defined fields stored in XMP (spool: namespace)
- **Rating & keywords** — star rating, keywords, location fields
- **Light / Dark / System theme**
- **Cross-platform** — Windows, Linux, macOS

## Crown & Flint Import

Import metadata from the [Crown & Flint](https://crownandflint.com) film photography app. Visually align your scanned photos with C&F's recorded exposure data, then write everything in one click.

![Crown & Flint Import](docs/C&F-Example.png)

## Installation

### macOS (Homebrew)
```
brew tap WanliSha/spool
brew install --cask spool
```

### Download
Download the latest release from [GitHub Releases](https://github.com/WanliSha/Spool/releases).

## License

Spool is dual-licensed:

- **Open Source**: [GPLv3](./LICENSE)
- **Commercial**: For closed-source or proprietary use, see [Commercial License](./LICENSE-COMMERCIAL.md)
