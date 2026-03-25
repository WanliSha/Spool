# Spool CLI — Available Fields

All fields below are readable (`get`) and writable (`set`). The CLI writes to all applicable formats simultaneously.

## Camera & Lens (EXIF)

| Field | Description | Example |
|---|---|---|
| CameraMake | Camera manufacturer | `Nikon` |
| CameraModel | Camera model | `FM2` |
| LensMake | Lens manufacturer | `Nikon` |
| LensModel | Lens model | `Nikkor 50mm f/1.4` |
| LensInfo | Focal/aperture range (min_f, max_f, min_a, max_a) | `50, 50, 1.4, 1.4` |
| FocalLength | Focal length (mm) | `50` |
| FocalLength35mm | 35mm equivalent focal length | `75` |

## Exposure (EXIF)

| Field | Description | Example |
|---|---|---|
| ISO | Film speed / sensor sensitivity | `400` |
| Aperture | F-number | `1.4` |
| ShutterSpeed | Exposure time in seconds | `0.008` (= 1/125) |
| ExposureProgram | 0=Undefined, 1=Manual, 2=Auto, 3=Av, 4=Tv | `1` |
| MeteringMode | 1=Average, 2=CenterWeighted, 3=Spot, 5=Matrix | `5` |
| Flash | 0=No flash, 1=Fired, etc. | `0` |
| WhiteBalance | 0=Auto, 1=Manual | `0` |

## Dates (EXIF) — auto-normalized

| Field | Description |
|---|---|
| DateTaken | Date the photo was taken (DateTimeOriginal) |
| DateCreated | Date the digital file was created |
| DateModified | Date the file was last modified |

- `YYYY-MM-DD` → defaults to `12:00:00`
- `YYYY-MM-DD HH:MM` → adds `:00` seconds
- `YYYY-MM-DD HH:MM:SS` → as-is

## GPS (EXIF)

| Field | Description | Example |
|---|---|---|
| GPSLatitude | Decimal latitude | `25.0340` |
| GPSLongitude | Decimal longitude | `121.5645` |
| GPSLatitudeRef | N or S | `N` |
| GPSLongitudeRef | E or W | `E` |
| GPSAltitude | Altitude in meters | `382.0` |
| GPSAltitudeRef | 0=Above sea level, 1=Below | `0` |

Set all GPS fields at once:
```bash
spool set IMG.jpg --json '{"GPSLatitude":"25.0340","GPSLongitude":"121.5645","GPSLatitudeRef":"N","GPSLongitudeRef":"E"}'
```

- Use your own geographic knowledge to resolve place names to coordinates
- Latitude: positive = N, negative = S
- Longitude: positive = E, negative = W

## Authorship & Description (EXIF + XMP + IPTC)

| Field | EXIF | XMP | IPTC |
|---|---|---|---|
| Author | Artist | dc:creator | Byline |
| Copyright | Copyright | dc:rights | Copyright |
| Description | ImageDescription | dc:description | Caption |
| Title | — | dc:title | Headline |
| Keywords | — | dc:subject | Keywords |
| Software | Software | xmp:CreatorTool | — |
| Comment | UserComment | — | — |
| AuthorTitle | — | — | Byline Title |

Keywords are comma-separated: `landscape, film, Taipei`

## Location (XMP + IPTC)

| Field | XMP | IPTC |
|---|---|---|
| City | photoshop:City | City |
| State | photoshop:State | Province/State |
| Country | photoshop:Country | Country |
| CountryCode | Iptc4xmpCore:CountryCode | Country Code |

## Rating & Labels (XMP only)

| Field | Description | Example |
|---|---|---|
| Rating | Star rating 0-5 | `5` |
| Label | Color label | `Red` |

## Editorial (IPTC only)

| Field | Description |
|---|---|
| Credit | Credit line |
| Source | Source |
| Instructions | Special instructions |

## Image Properties (EXIF, read-only)

| Field | Description |
|---|---|
| Width | Image width in pixels |
| Height | Image height in pixels |
| Orientation | EXIF orientation tag |

## Custom Fields (XMP)

Any field starting with `spool:` is stored in the Spool custom XMP namespace:
```bash
spool set IMG.jpg --json '{"spool:FilmStock":"Kodak Portra 400","spool:DevelopProcess":"C-41"}'
```
