use std::collections::HashMap;
use std::path::Path;

use crate::exif;
use crate::iptc;
use crate::xmp;

/// Unified field key mapping from EXIF keys to unified keys
pub fn exif_to_unified(exif_key: &str) -> Option<&str> {
    match exif_key {
        "Make" => Some("CameraMake"),
        "Model" => Some("CameraModel"),
        "LensMake" => Some("LensMake"),
        "LensModel" => Some("LensModel"),
        "LensInfo" => Some("LensInfo"),
        "FocalLength" => Some("FocalLength"),
        "FocalLengthIn35mmFormat" => Some("FocalLength35mm"),
        "ISO" => Some("ISO"),
        "FNumber" => Some("Aperture"),
        "ExposureTime" => Some("ShutterSpeed"),
        "ExposureProgram" => Some("ExposureProgram"),
        "MeteringMode" => Some("MeteringMode"),
        "Flash" => Some("Flash"),
        "WhiteBalance" => Some("WhiteBalance"),
        "ImageWidth" => Some("Width"),
        "ImageHeight" => Some("Height"),
        "Orientation" => Some("Orientation"),
        "Artist" => Some("Author"),
        "Copyright" => Some("Copyright"),
        "ImageDescription" => Some("Description"),
        "DateTimeOriginal" => Some("DateTaken"),
        "CreateDate" => Some("DateCreated"),
        "ModifyDate" => Some("DateModified"),
        "Software" => Some("Software"),
        "UserComment" => Some("Comment"),
        "GPSLatitude" => Some("GPSLatitude"),
        "GPSLongitude" => Some("GPSLongitude"),
        "GPSLatitudeRef" => Some("GPSLatitudeRef"),
        "GPSLongitudeRef" => Some("GPSLongitudeRef"),
        "GPSAltitude" => Some("GPSAltitude"),
        "GPSAltitudeRef" => Some("GPSAltitudeRef"),
        _ => None,
    }
}

/// Read all metadata from a file, merging EXIF + XMP + IPTC
/// Priority: XMP > IPTC > EXIF for overlapping fields
pub fn read_all(path: &Path) -> HashMap<String, String> {
    let mut fields = HashMap::new();

    // 1. Read EXIF (lowest priority for overlapping fields)
    let exif_fields = exif::read_exif_fields_raw(path);
    for (k, v) in &exif_fields {
        if let Some(unified) = exif_to_unified(k) {
            fields.insert(unified.to_string(), v.clone());
        }
    }

    // 2. Read IPTC (overrides EXIF for shared fields)
    let iptc_fields = iptc::read_iptc(path);
    for (k, v) in &iptc_fields {
        fields.insert(k.clone(), v.clone());
    }

    // 3. Read XMP (highest priority)
    let xmp_fields = xmp::read_xmp(path);
    for (k, v) in &xmp_fields {
        fields.insert(k.clone(), v.clone());
    }

    fields
}

/// Write metadata to all applicable formats
pub fn write_all(path: &Path, fields: &HashMap<String, String>) -> Result<(), String> {
    // Write EXIF
    exif::write_exif_fields(path, fields)?;

    // Write XMP
    xmp::write_xmp(path, fields)?;

    // Write IPTC (only if there are IPTC-relevant fields)
    let iptc_keys = [
        "Title", "Description", "Author", "AuthorTitle", "Copyright",
        "Keywords", "City", "State", "Country", "CountryCode",
        "Credit", "Source", "Instructions",
    ];
    if iptc_keys.iter().any(|k| fields.contains_key(*k)) {
        iptc::write_iptc(path, fields)?;
    }

    Ok(())
}
