# Metadata Formats Spec

## Overview

Photos can contain three types of metadata. Spool displays them as unified fields — the user never needs to know which format stores what. On write, all formats are synced automatically.

## Formats

### EXIF (current)
- Camera parameters, GPS, dates
- Binary TIFF tags
- Read/write via `little_exif`
- Status: **Supported**

### XMP (planned)
- Adobe's extensible metadata format
- XML-based, stored in APP1 segment (JPEG) or tag 700 (TIFF)
- Read/write via XML parsing (`quick-xml` or similar)
- Status: **Planned**

### IPTC/IIM (planned)
- News/publishing industry standard
- Binary records, stored in APP13 segment (JPEG) or tag 33723 (TIFF)
- Status: **Planned**

## UI Strategy

- **No tabs, no format labels** — fields grouped by function, not by format
- User edits one field → Spool writes to all applicable formats
- Lightroom-style: user doesn't know EXIF/XMP/IPTC exists

## Unified Field Map

### Camera & Lens (read-only, from EXIF)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Camera Make | Make | tiff:Make | — |
| Camera Model | Model | tiff:Model | — |
| Lens Make | LensMake | exifEX:LensMake | — |
| Lens Model | LensModel | exifEX:LensModel | — |
| Lens Info | LensInfo | exifEX:LensInfo | — |
| Focal Length | FocalLength | exif:FocalLength | — |
| Focal Length (35mm) | FocalLengthIn35mmFormat | exif:FocalLengthIn35mmFilm | — |

### Exposure (read-only, from EXIF)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| ISO | ISO | exif:ISOSpeedRatings | — |
| Aperture | FNumber | exif:FNumber | — |
| Shutter Speed | ExposureTime | exif:ExposureTime | — |
| Exposure Program | ExposureProgram | exif:ExposureProgram | — |
| Metering Mode | MeteringMode | exif:MeteringMode | — |
| Flash | Flash | exif:Flash | — |
| White Balance | WhiteBalance | exif:WhiteBalance | — |

### Image (read-only, from EXIF)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Width | ImageWidth | tiff:ImageWidth | — |
| Height | ImageHeight | tiff:ImageLength | — |
| Orientation | Orientation | tiff:Orientation | — |

### Description (editable, synced)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Title | — | dc:title | Headline |
| Description | ImageDescription | dc:description | Caption-Abstract |
| Keywords | — | dc:subject | Keywords |

### People & Rights (editable, synced)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Author | Artist | dc:creator | By-line |
| Author Title | — | — | By-line Title |
| Copyright | Copyright | dc:rights | CopyrightNotice |
| Credit | — | — | Credit |
| Source | — | — | Source |

### Dates (editable, synced)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Date Taken | DateTimeOriginal | photoshop:DateCreated | DateCreated |
| Date Created | CreateDate | xmp:CreateDate | DigitalCreationDate |
| Date Modified | ModifyDate | xmp:ModifyDate | — |

### GPS (editable, from EXIF)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| GPS Latitude | GPSLatitude + GPSLatitudeRef | exif:GPSLatitude | — |
| GPS Longitude | GPSLongitude + GPSLongitudeRef | exif:GPSLongitude | — |
| GPS Altitude | GPSAltitude + GPSAltitudeRef | exif:GPSAltitude | — |

### Location (editable, IPTC/XMP only)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| City | — | photoshop:City | City |
| State/Province | — | photoshop:State | Province-State |
| Country | — | photoshop:Country | Country-PrimaryLocationName |
| Country Code | — | Iptc4xmpCore:CountryCode | Country-PrimaryLocationCode |

### Rating & Organization (editable, XMP only)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Rating | — | xmp:Rating | — |
| Label | — | xmp:Label | — |

### Other (editable)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Software | Software | xmp:CreatorTool | — |
| Comment | UserComment | — | — |
| Instructions | — | — | SpecialInstructions |

### Custom Fields (XMP spool: namespace)

| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| (user defined) | — | spool:{fieldName} | — |

## Read Priority

When the same field has values in multiple formats:
1. XMP (most authoritative — this is what Lightroom writes)
2. IPTC
3. EXIF

## Implementation Order

1. XMP read/write + custom fields + Rating/Keywords
2. IPTC read/write
3. Sync logic (write to all formats on edit)
4. UI: unified field groups (no format distinction)
