# Crown & Flint Import Spec

## Overview
Import metadata from the Crown & Flint iOS app's JSON export and write it to scanned film photos via a visual alignment interface.

## C&F Export Structure

```
C&F folder/
  film.json           ← required, array of metadata entries
  {ImageNumber}.jpeg   ← optional preview images, may be partial or absent
```

## JSON Format (hardcoded mapping)

| C&F JSON field | Spool unified key |
|---|---|
| Make | CameraMake |
| Model | CameraModel |
| LensMake | LensMake |
| LensModel | LensModel |
| LensInfo | LensInfo |
| FocalLength | FocalLength |
| FocalLengthIn35mmFormat | FocalLength35mm |
| ExposureTime | ShutterSpeed |
| FNumber | Aperture |
| ISO | ISO |
| DateTimeOriginal | DateTaken |
| Description | Description |
| Keywords (array) | Keywords (comma-separated) |
| GPSLatitude + GPSLatitudeRef | GPSLatitude + GPSLatitudeRef |
| GPSLongitude + GPSLongitudeRef | GPSLongitude + GPSLongitudeRef |
| UserComment | Comment |

## Flow

1. User clicks "Import from Crown & Flint" on drop zone
2. Full-page alignment mode: left panel (scan photos) + right panel (C&F data)
3. Each side independently: import folder, reverse, delete, drag reorder, clear
4. Same row = matched pair. Visual comparison via thumbnails / previews
5. Confirm & Write: writes metadata to scan photos
6. Enters normal Spool editor with photos loaded

## Edge Cases

- Scan photos may be in reverse order → Reverse button
- Scan photos may be fewer than JSON entries → unmatched rows show empty
- JSON may have erroneous entries → delete individual entries
- C&F preview images may be partial or completely absent → placeholder shown
- ImageNumber may start from 0 or 1
- film.json missing → error
- film.json invalid format → error
