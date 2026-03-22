# Core Features Spec

## Target Audience

Primary: film photographers who need to build EXIF data from scratch (scanned files have no or incorrect metadata).

Also supports general EXIF editing for digital photos.

## Editing

### Single Photo Edit
- View and edit EXIF fields
- Uses `little_exif` crate (pure Rust, read+write)

### Supported EXIF Fields

**Editable (string fields):**
- Camera: Make, Model
- Lens: LensMake, LensModel
- Metadata: Software, Artist, Copyright, ImageDescription
- Dates: DateTimeOriginal, CreateDate, ModifyDate
- GPS: GPSLatitudeRef, GPSLongitudeRef
- Other: UserComment

**Read-only (numeric/rational fields):**
- Exposure: ISO, FNumber, ExposureTime, ExposureProgram, MeteringMode, Flash, WhiteBalance
- Lens: LensInfo, FocalLength, FocalLengthIn35mmFormat
- Image: ImageWidth, ImageHeight, Orientation
- GPS: GPSLatitude, GPSLongitude, GPSAltitudeRef, GPSAltitude

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
