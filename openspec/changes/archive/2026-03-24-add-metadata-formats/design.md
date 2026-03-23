# Add Metadata Formats — Design

## Architecture

```
Current:
  exif.rs → little_exif → read/write EXIF only

New:
  metadata.rs (new, replaces exif.rs as the main module)
    ├── read_all(path) → HashMap<String, String>
    │     ├── read EXIF via little_exif
    │     ├── read XMP via quick-xml (parse APP1 XML)
    │     ├── read IPTC via manual binary parsing (APP13)
    │     └── merge with priority: XMP > IPTC > EXIF
    │
    ├── write_all(path, fields)
    │     ├── write EXIF via little_exif
    │     ├── write XMP via quick-xml
    │     └── write IPTC via binary encoding
    │
    xmp.rs (new)
    │  ├── read_xmp(path) → HashMap<String, String>
    │  └── write_xmp(path, fields)
    │
    iptc.rs (new)
       ├── read_iptc(path) → HashMap<String, String>
       └── write_iptc(path, fields)
```

## Key Design Decisions

### Unified Field Keys
All three formats map to a single set of UI field keys. The field key is what the frontend sees:

```
"Title"       → reads from XMP dc:title / IPTC Headline
"Author"      → reads from XMP dc:creator / IPTC By-line / EXIF Artist
"Rating"      → reads from XMP xmp:Rating
"Keywords"    → reads from XMP dc:subject / IPTC Keywords (comma-separated string)
"spool:Film"  → reads from XMP spool:Film (custom field)
```

### XMP Implementation
- Read: scan JPEG for APP1 segment containing `<x:xmpmeta`, parse XML with `quick-xml`
- Write: modify existing XMP XML or create new one, write back to APP1 segment
- For TIFF/DNG: read/write TIFF tag 700
- Use `img-parts` crate to safely replace JPEG segments without corrupting the file

### IPTC Implementation
- Read: scan JPEG for APP13 segment, parse IPTC-IIM binary records
- Write: encode records, write back to APP13 segment
- Record format: 1C 02 {dataset} {size:2} {data}

### Custom Fields
- Stored in settings.json as array of { name, type }
- Written to XMP under spool: namespace
- `<rdf:Description xmlns:spool="http://spool.app/ns/1.0/" spool:FilmStock="Portra 400"/>`

### ExifData Struct (unchanged)
The existing ExifData struct with `fields: HashMap<String, String>` works as-is.
The keys just expand from EXIF-only tags to unified field names.

## Dependencies
- `quick-xml` — XML parsing for XMP (already in project via rawler dep)
- `img-parts` — safe JPEG segment manipulation for writing XMP/IPTC back

## Frontend Changes
- Replace FIELD_LABELS with grouped sections
- Add Rating widget (clickable stars)
- Add Keywords input (tag-style)
- Add Location fields
- Add Custom Fields section
- Settings: manage custom field definitions
