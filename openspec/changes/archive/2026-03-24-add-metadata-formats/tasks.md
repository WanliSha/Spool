# Add Metadata Formats — Tasks

## Backend — XMP
- [x] Add `quick-xml` and `img-parts` dependencies
- [x] Create xmp.rs: read XMP from JPEG (APP1) and TIFF (tag 700)
- [x] Parse XMP XML to extract standard fields (dc:*, xmp:*, photoshop:*, exif:*, Iptc4xmpCore:*)
- [x] Parse XMP custom spool: namespace fields
- [x] Write XMP: modify or create XMP XML, write back to file

## Backend — IPTC
- [x] Create iptc.rs: read IPTC from JPEG (APP13)
- [x] Parse IPTC-IIM binary records to extract fields
- [x] Write IPTC: encode records, write back to file

## Backend — Unified Metadata
- [x] Create metadata.rs: unified read_all / write_all
- [x] Define unified field key mapping (UI key → EXIF + XMP + IPTC)
- [x] Read: merge from all formats with priority XMP > IPTC > EXIF
- [x] Write: sync to all applicable formats
- [x] Update Tauri commands to use metadata.rs instead of exif.rs
- [x] Custom fields: read/write from settings + XMP spool: namespace

## Frontend — Grouped Editor
- [x] Replace flat field list with grouped sections (Camera, Exposure, Description, People, Dates, GPS, Location, Rating, Custom)
- [x] Add Rating widget (clickable stars)
- [x] Add Keywords input (comma-separated)
- [x] Add Location fields (City, State, Country)
- [x] Add Custom Fields section (dynamic from settings)

## Frontend — Settings
- [x] Add custom field management in settings (add/remove)

## Verify
- [x] Build and test
