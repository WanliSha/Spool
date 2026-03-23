# Add Metadata Formats (XMP + IPTC)

## Summary
Add XMP and IPTC read/write support alongside existing EXIF. Unified UI grouped by function, not format. Automatic sync across all three formats on write. Custom fields via XMP spool: namespace.

## Scope
- XMP read/write (XML parsing in JPEG APP1 / TIFF tag 700)
- IPTC read/write (binary records in JPEG APP13 / TIFF tag 33723)
- Unified field groups in editor panel (Camera, Exposure, Description, People, Dates, GPS, Location, Rating, Custom)
- Read priority: XMP > IPTC > EXIF
- Write sync: edit one field → update all applicable formats
- Custom fields: user-defined in settings, stored in XMP spool: namespace
- Rating (1-5 stars) and Keywords support
- Location text fields (City, State, Country)

## Out of Scope
- XMP sidecar files (.xmp)
- IPTC Extensions (stored in XMP, can add later)
- MakerNote (vendor-specific, read-only at best)

## References
- Spec: `openspec/specs/metadata-formats/spec.md`

## Status
Pending
