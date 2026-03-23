# Metadata Formats Spec

## Overview

Photos can contain three types of metadata. Spool displays them as unified fields — the user never needs to know which format stores what. On write, all formats are synced automatically.

## Formats

### EXIF
- Camera parameters, GPS, dates
- Binary TIFF tags
- Read/write via `little_exif`
- Status: **Supported**

### XMP
- Adobe's extensible metadata format
- XML-based, stored in APP1 segment (JPEG) or tag 700 (TIFF)
- Read/write via custom XML parser (`quick-xml`)
- Status: **Supported**

### IPTC/IIM
- News/publishing industry standard
- Binary records, stored in APP13 segment (JPEG) or tag 33723 (TIFF)
- Read/write via custom binary parser
- Status: **Supported**

## UI Strategy

- **No tabs, no format labels** — fields grouped by function, not by format
- User edits one field → Spool writes to all applicable formats
- Lightroom-style: user doesn't know EXIF/XMP/IPTC exists

## Unified Field Groups

### Camera & Lens
| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Camera Make | Make | tiff:Make | — |
| Camera Model | Model | tiff:Model | — |
| Lens Make | LensMake | exifEX:LensMake | — |
| Lens Model | LensModel | exifEX:LensModel | — |
| Lens Info | LensInfo | exifEX:LensInfo | — |
| Focal Length | FocalLength | exif:FocalLength | — |
| Focal Length (35mm) | FocalLengthIn35mmFormat | exif:FocalLengthIn35mmFilm | — |

### Exposure
| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| ISO | ISO | exif:ISOSpeedRatings | — |
| Aperture | FNumber | exif:FNumber | — |
| Shutter Speed | ExposureTime | exif:ExposureTime | — |
| Exposure Program | ExposureProgram | exif:ExposureProgram | — |
| Metering Mode | MeteringMode | exif:MeteringMode | — |
| Flash | Flash | exif:Flash | — |
| White Balance | WhiteBalance | exif:WhiteBalance | — |

### Description
| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Title | — | dc:title | Headline |
| Description | ImageDescription | dc:description | Caption-Abstract |
| Keywords | — | dc:subject | Keywords |

### People & Rights
| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Author | Artist | dc:creator | By-line |
| Author Title | — | — | By-line Title |
| Copyright | Copyright | dc:rights | CopyrightNotice |
| Credit | — | — | Credit |
| Source | — | — | Source |

### Dates
| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Date Taken | DateTimeOriginal | photoshop:DateCreated | DateCreated |
| Date Created | CreateDate | xmp:CreateDate | DigitalCreationDate |
| Date Modified | ModifyDate | xmp:ModifyDate | — |

### GPS
| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| GPS Latitude | GPSLatitude + GPSLatitudeRef | exif:GPSLatitude | — |
| GPS Longitude | GPSLongitude + GPSLongitudeRef | exif:GPSLongitude | — |
| GPS Altitude | GPSAltitude + GPSAltitudeRef | exif:GPSAltitude | — |

### Location
| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| City | — | photoshop:City | City |
| State/Province | — | photoshop:State | Province-State |
| Country | — | photoshop:Country | Country-PrimaryLocationName |
| Country Code | — | Iptc4xmpCore:CountryCode | Country-PrimaryLocationCode |

### Rating & Organization
| UI Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Rating | — | xmp:Rating | — |
| Label | — | xmp:Label | — |

### Other
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

## All Fields Editable

Every field in the editor is editable, including camera/lens/exposure fields. This is essential for film photographers who need to build metadata from scratch for scanned photos.
