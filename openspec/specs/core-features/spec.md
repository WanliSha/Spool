# Core Features Spec

## Target Audience

Primary: film photographers who need to build metadata from scratch (scanned files have no or incorrect metadata).

Also supports general metadata editing for digital photos.

## Metadata Editing

- Unified editor: EXIF, XMP, IPTC displayed as one set of fields grouped by function
- All fields editable (camera, lens, exposure, dates, GPS, location, rating, etc.)
- Custom fields via XMP spool: namespace (user-defined in settings)
- Automatic sync across EXIF, XMP, IPTC on write
- See `metadata-formats` spec for full field mapping

## Multi-Select & Batch Edit

- Cmd+click toggle, Shift+click range, Select All / Deselect All
- Multi-select shows common values / "Mixed" for differing fields
- Per-field dirty tracking: only touched fields written on save
- Global undo stack (including undo save)
- See `file-write` spec for full behaviour

## Photo Preview

- Single select: one photo at a time, arrow key / wheel navigation
- Multi select: auto-layout grid of all selected photos
- Zoom (wheel, +/- buttons), pan (drag), rotate CW/CCW (single only)
- Quick mode (~2048px, default) and 1:1 full resolution mode
- Double-click to reset zoom

## RAW Format Support

- Embedded JPEG extraction for fast preview (custom JPEG segment parser)
- Fallback to `rawler` crate for full RAW decode
- Supported: CR2, CR3, NEF, ARW, RAF, DNG, ORF, RW2, PEF

## GPS Coordinate Editing

- OpenStreetMap + Leaflet.js embedded map picker
- Lazy-loaded, with Nominatim location search
- Blue pin (saved GPS), red pin (unsaved selection)
- Multi-select: blue pin only when all selected share same GPS

## Crown & Flint Import

- Import metadata from Crown & Flint app's JSON export (film.json)
- Visual left/right alignment: scan photos vs C&F entries
- Both sides: drag reorder, reverse, delete
- C&F preview images displayed when available
- Confirm & Write: writes to photos, then enters normal editor

## Rating & Keywords

- Star rating widget (1-5, clickable)
- Keywords field (comma-separated, stored in XMP dc:subject + IPTC Keywords)

## Equipment Templates

- Save a set of commonly used values (camera body, lens, film stock, etc.)
- Apply a template to selected photos (shortcut for batch edit)
- Status: **Planned**
