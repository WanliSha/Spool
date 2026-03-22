# Core Features Spec

## Target Audience

Primary: film photographers who need to build EXIF data from scratch (scanned files have no or incorrect metadata).

Also supports general EXIF editing for digital photos.

## Editing

### Single Photo Edit
- View and edit any EXIF/IPTC/XMP field
- Full field coverage (like ExifTool)

### Batch Edit
- Select multiple photos
- Edit one or more fields
- Only modified fields are written; unmodified fields are left untouched

### Equipment Templates
- Save a set of commonly used values (camera body, lens, film stock, etc.)
- Apply a template to selected photos (shortcut for batch edit)

## Photo Preview
- Quick preview: ~2048px long edge (default, fast)
- Full resolution: original decoded size (on demand)
- Supports JPEG, HEIF, TIFF, and RAW formats (CR3, ARW, NEF, RAF, DNG)

## GPS Coordinate Editing
- OpenStreetMap + Leaflet.js map picker
- Lazy-loaded when user opens the map
- Click to select coordinates
- Search location by name
- Apply coordinates to single or multiple photos (via batch edit)
