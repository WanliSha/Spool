use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;
use std::collections::HashMap;
use std::path::Path;

fn tag_name(tag: &ExifTag) -> String {
    let debug = format!("{:?}", tag);
    debug.split('(').next().unwrap_or("Unknown").to_string()
}

fn dms_to_decimal(rationals: &[uR64]) -> Option<f64> {
    if rationals.len() < 3 {
        return None;
    }
    let deg = rationals[0].nominator as f64 / rationals[0].denominator.max(1) as f64;
    let min = rationals[1].nominator as f64 / rationals[1].denominator.max(1) as f64;
    let sec = rationals[2].nominator as f64 / rationals[2].denominator.max(1) as f64;
    Some(deg + min / 60.0 + sec / 3600.0)
}

fn rational_to_float(rationals: &[uR64]) -> Option<f64> {
    if rationals.is_empty() {
        return None;
    }
    let r = &rationals[0];
    if r.denominator == 0 {
        return None;
    }
    Some(r.nominator as f64 / r.denominator as f64)
}

fn extract_value(tag: &ExifTag) -> Option<String> {
    match tag {
        ExifTag::Make(v) | ExifTag::Model(v) | ExifTag::LensMake(v)
        | ExifTag::LensModel(v) | ExifTag::Software(v) | ExifTag::Artist(v)
        | ExifTag::Copyright(v) | ExifTag::ImageDescription(v)
        | ExifTag::DateTimeOriginal(v) | ExifTag::CreateDate(v) | ExifTag::ModifyDate(v)
        | ExifTag::GPSLatitudeRef(v) | ExifTag::GPSLongitudeRef(v) => {
            if v.is_empty() { None } else { Some(v.clone()) }
        }
        ExifTag::GPSLatitude(v) => dms_to_decimal(v).map(|d| format!("{:.6}", d)),
        ExifTag::GPSLongitude(v) => dms_to_decimal(v).map(|d| format!("{:.6}", d)),
        ExifTag::GPSAltitude(v) | ExifTag::ExposureTime(v)
        | ExifTag::FNumber(v) | ExifTag::FocalLength(v) => {
            rational_to_float(v).map(|f| format!("{}", f))
        }
        ExifTag::LensInfo(v) => {
            if v.is_empty() { return None; }
            let parts: Vec<String> = v.iter()
                .map(|r| format!("{}", r.nominator as f64 / r.denominator.max(1) as f64))
                .collect();
            Some(parts.join(", "))
        }
        ExifTag::ISO(v) => v.first().map(|n| n.to_string()),
        ExifTag::FocalLengthIn35mmFormat(v) => v.first().map(|n| n.to_string()),
        ExifTag::ExposureProgram(v) | ExifTag::MeteringMode(v)
        | ExifTag::Flash(v) | ExifTag::WhiteBalance(v)
        | ExifTag::Orientation(v) => v.first().map(|n| n.to_string()),
        ExifTag::ImageWidth(v) | ExifTag::ImageHeight(v) => v.first().map(|n| n.to_string()),
        ExifTag::GPSAltitudeRef(v) => v.first().map(|n| n.to_string()),
        ExifTag::UserComment(v) => {
            let s = String::from_utf8_lossy(v).to_string();
            if s.is_empty() { None } else { Some(s) }
        }
        _ => None,
    }
}

pub fn decimal_to_dms(decimal: f64) -> Vec<uR64> {
    let abs = decimal.abs();
    let degrees = abs.floor() as u32;
    let minutes_float = (abs - degrees as f64) * 60.0;
    let minutes = minutes_float.floor() as u32;
    let seconds_float = (minutes_float - minutes as f64) * 60.0;
    let seconds_num = (seconds_float * 1000.0).round() as u32;
    vec![
        uR64 { nominator: degrees, denominator: 1 },
        uR64 { nominator: minutes, denominator: 1 },
        uR64 { nominator: seconds_num, denominator: 1000 },
    ]
}

fn float_to_rational(value: &str) -> Option<Vec<uR64>> {
    let f: f64 = value.parse().ok()?;
    let num = (f * 10000.0).round() as u32;
    Some(vec![uR64 { nominator: num, denominator: 10000 }])
}

fn parse_u16(value: &str) -> Option<Vec<u16>> {
    let v: u16 = value.parse().ok()?;
    Some(vec![v])
}

pub fn name_to_tag(name: &str, value: &str) -> Option<ExifTag> {
    match name {
        "Make" => Some(ExifTag::Make(value.to_string())),
        "Model" => Some(ExifTag::Model(value.to_string())),
        "LensMake" => Some(ExifTag::LensMake(value.to_string())),
        "LensModel" => Some(ExifTag::LensModel(value.to_string())),
        "Software" => Some(ExifTag::Software(value.to_string())),
        "Artist" => Some(ExifTag::Artist(value.to_string())),
        "Copyright" => Some(ExifTag::Copyright(value.to_string())),
        "ImageDescription" => Some(ExifTag::ImageDescription(value.to_string())),
        "DateTimeOriginal" => Some(ExifTag::DateTimeOriginal(value.to_string())),
        "CreateDate" => Some(ExifTag::CreateDate(value.to_string())),
        "ModifyDate" => Some(ExifTag::ModifyDate(value.to_string())),
        "GPSLatitudeRef" => Some(ExifTag::GPSLatitudeRef(value.to_string())),
        "GPSLongitudeRef" => Some(ExifTag::GPSLongitudeRef(value.to_string())),
        "UserComment" => Some(ExifTag::UserComment(value.as_bytes().to_vec())),
        "GPSLatitude" => {
            let f: f64 = value.parse().ok()?;
            Some(ExifTag::GPSLatitude(decimal_to_dms(f)))
        }
        "GPSLongitude" => {
            let f: f64 = value.parse().ok()?;
            Some(ExifTag::GPSLongitude(decimal_to_dms(f)))
        }
        "GPSAltitude" => float_to_rational(value).map(ExifTag::GPSAltitude),
        "GPSAltitudeRef" => {
            let v: u8 = value.parse().ok()?;
            Some(ExifTag::GPSAltitudeRef(vec![v]))
        }
        "ExposureTime" => float_to_rational(value).map(ExifTag::ExposureTime),
        "FNumber" => float_to_rational(value).map(ExifTag::FNumber),
        "FocalLength" => float_to_rational(value).map(ExifTag::FocalLength),
        "LensInfo" => {
            // Expects "min_focal, max_focal, min_aperture, max_aperture" e.g. "50, 50, 1.4, 1.4"
            let parts: Vec<f64> = value.split(',').filter_map(|s| s.trim().parse().ok()).collect();
            if parts.len() == 4 {
                Some(ExifTag::LensInfo(parts.iter().map(|f| {
                    let num = (f * 10000.0).round() as u32;
                    uR64 { nominator: num, denominator: 10000 }
                }).collect()))
            } else {
                None
            }
        }
        "ISO" => parse_u16(value).map(ExifTag::ISO),
        "FocalLengthIn35mmFormat" => parse_u16(value).map(ExifTag::FocalLengthIn35mmFormat),
        "ExposureProgram" => parse_u16(value).map(ExifTag::ExposureProgram),
        "MeteringMode" => parse_u16(value).map(ExifTag::MeteringMode),
        "Flash" => parse_u16(value).map(ExifTag::Flash),
        "WhiteBalance" => parse_u16(value).map(ExifTag::WhiteBalance),
        "Orientation" => parse_u16(value).map(ExifTag::Orientation),
        _ => None,
    }
}

/// Read EXIF fields from file
pub fn read_exif_fields_raw(path: &Path) -> HashMap<String, String> {
    let mut fields = HashMap::new();

    let metadata = match Metadata::new_from_path(path) {
        Ok(m) => m,
        Err(_) => return fields,
    };

    let tags_to_read: Vec<ExifTag> = vec![
        ExifTag::Make(String::new()),
        ExifTag::Model(String::new()),
        ExifTag::LensMake(String::new()),
        ExifTag::LensModel(String::new()),
        ExifTag::LensInfo(vec![]),
        ExifTag::Software(String::new()),
        ExifTag::Artist(String::new()),
        ExifTag::Copyright(String::new()),
        ExifTag::ImageDescription(String::new()),
        ExifTag::DateTimeOriginal(String::new()),
        ExifTag::CreateDate(String::new()),
        ExifTag::ModifyDate(String::new()),
        ExifTag::ISO(vec![]),
        ExifTag::FNumber(vec![]),
        ExifTag::ExposureTime(vec![]),
        ExifTag::FocalLength(vec![]),
        ExifTag::FocalLengthIn35mmFormat(vec![]),
        ExifTag::ExposureProgram(vec![]),
        ExifTag::MeteringMode(vec![]),
        ExifTag::Flash(vec![]),
        ExifTag::WhiteBalance(vec![]),
        ExifTag::ImageWidth(vec![]),
        ExifTag::ImageHeight(vec![]),
        ExifTag::Orientation(vec![]),
        ExifTag::GPSLatitudeRef(String::new()),
        ExifTag::GPSLatitude(vec![]),
        ExifTag::GPSLongitudeRef(String::new()),
        ExifTag::GPSLongitude(vec![]),
        ExifTag::GPSAltitudeRef(vec![]),
        ExifTag::GPSAltitude(vec![]),
        ExifTag::UserComment(Vec::new()),
    ];

    for tag in &tags_to_read {
        let name = tag_name(tag);
        if let Some(found_tag) = metadata.get_tag(tag).next() {
            if let Some(value) = extract_value(found_tag) {
                fields.insert(name, value);
            }
        }
    }

    fields
}

/// Map unified field keys back to EXIF tag names
pub fn unified_to_exif(key: &str) -> Option<&str> {
    match key {
        "CameraMake" => Some("Make"),
        "CameraModel" => Some("Model"),
        "LensMake" => Some("LensMake"),
        "LensModel" => Some("LensModel"),
        "Author" => Some("Artist"),
        "Copyright" => Some("Copyright"),
        "Description" => Some("ImageDescription"),
        "DateTaken" => Some("DateTimeOriginal"),
        "DateCreated" => Some("CreateDate"),
        "DateModified" => Some("ModifyDate"),
        "Software" => Some("Software"),
        "Comment" => Some("UserComment"),
        "GPSLatitude" => Some("GPSLatitude"),
        "GPSLongitude" => Some("GPSLongitude"),
        "GPSLatitudeRef" => Some("GPSLatitudeRef"),
        "GPSLongitudeRef" => Some("GPSLongitudeRef"),
        "GPSAltitude" => Some("GPSAltitude"),
        "GPSAltitudeRef" => Some("GPSAltitudeRef"),
        "ISO" => Some("ISO"),
        "Aperture" => Some("FNumber"),
        "ShutterSpeed" => Some("ExposureTime"),
        "FocalLength" => Some("FocalLength"),
        "LensInfo" => Some("LensInfo"),
        "FocalLength35mm" => Some("FocalLengthIn35mmFormat"),
        "ExposureProgram" => Some("ExposureProgram"),
        "MeteringMode" => Some("MeteringMode"),
        "Flash" => Some("Flash"),
        "WhiteBalance" => Some("WhiteBalance"),
        "Orientation" => Some("Orientation"),
        _ => None,
    }
}

/// Write EXIF fields to a file using unified field keys
pub fn write_exif_fields(path: &Path, fields: &HashMap<String, String>) -> Result<(), String> {
    let mut metadata = Metadata::new_from_path(path).unwrap_or_else(|_| Metadata::new());

    for (unified_key, value) in fields {
        if let Some(exif_key) = unified_to_exif(unified_key) {
            if let Some(tag) = name_to_tag(exif_key, value) {
                metadata.set_tag(tag);
            }
        }
    }

    metadata
        .write_to_file(path)
        .map_err(|e| format!("Cannot write EXIF: {e}"))
}
