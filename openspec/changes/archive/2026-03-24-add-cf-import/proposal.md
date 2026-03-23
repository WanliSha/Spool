# Add Crown & Flint Import

## Summary
Import metadata from Crown & Flint app's JSON export and write it to scanned photos via a visual alignment interface.

## Flow
1. User clicks "Import from Crown & Flint" on the drop zone page
2. Enters a dedicated left/right panel alignment mode
3. Left panel: import scan photos folder (thumbnails, drag, reorder, reverse, delete)
4. Right panel: import C&F folder (reads film.json + optional preview JPEGs, drag, reorder, reverse, delete)
5. User aligns left and right by row — each row is a match pair
6. Confirm & Write → writes metadata to scan photos, then enters normal Spool editor

## C&F Folder Structure
- `film.json` — required, array of metadata entries
- `{ImageNumber}.jpeg` — optional preview images, may be partial or missing entirely

## JSON Field Mapping (hardcoded)
| C&F JSON | Spool unified key |
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

## Edge Cases
- Scan photos may be in reverse order
- Scan photos may be fewer than JSON entries (head/tail/middle missing)
- JSON may have erroneous entries to skip
- C&F preview images may be partial or completely absent
- ImageNumber may start from 0 or 1
- film.json missing → error
- film.json invalid format → error

## Out of Scope
- Other JSON formats (non-C&F)
- Automatic alignment / image matching

## Status
Pending
