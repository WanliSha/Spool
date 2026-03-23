use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

#[derive(Clone, Serialize)]
pub struct ExifData {
    pub path: String,
    pub fields: HashMap<String, String>,
    pub snapshot: HashMap<String, String>,
    pub modified: bool,
}

struct ExifState {
    snapshot: HashMap<String, String>,
    current: HashMap<String, String>,
}

impl ExifState {
    fn is_modified(&self) -> bool {
        self.current != self.snapshot
    }
}

pub struct ExifStore {
    states: Mutex<HashMap<String, ExifState>>,
}

impl ExifStore {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }
}

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

fn decimal_to_dms(decimal: f64) -> Vec<uR64> {
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

fn name_to_tag(name: &str, value: &str) -> Option<ExifTag> {
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
        "ExposureTime" => float_to_rational(value).map(ExifTag::ExposureTime),
        "FNumber" => float_to_rational(value).map(ExifTag::FNumber),
        "FocalLength" => float_to_rational(value).map(ExifTag::FocalLength),
        "ISO" => parse_u16(value).map(ExifTag::ISO),
        "Orientation" => parse_u16(value).map(ExifTag::Orientation),
        _ => None,
    }
}

/// Read EXIF fields from file (public for metadata module)
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
fn unified_to_exif(key: &str) -> Option<&str> {
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
        "ISO" => Some("ISO"),
        "Aperture" => Some("FNumber"),
        "ShutterSpeed" => Some("ExposureTime"),
        "FocalLength" => Some("FocalLength"),
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

// === Tauri Commands ===

#[tauri::command]
pub fn read_exif(path: String, store: State<'_, ExifStore>) -> Result<ExifData, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err("File not found".to_string());
    }

    let fields = read_exif_fields_raw(file_path);
    let state = ExifState {
        snapshot: fields.clone(),
        current: fields.clone(),
    };

    let data = ExifData {
        path: path.clone(),
        fields: fields.clone(),
        snapshot: fields.clone(),
        modified: false,
    };

    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    states.insert(path, state);

    Ok(data)
}

#[tauri::command]
pub fn get_exif_batch(paths: Vec<String>, store: State<'_, ExifStore>) -> Result<Vec<ExifData>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for path in &paths {
        // Load if not already loaded
        if !states.contains_key(path) {
            let file_path = Path::new(path);
            if !file_path.exists() {
                continue;
            }
            let fields = read_exif_fields_raw(file_path);
            states.insert(
                path.clone(),
                ExifState {
                    snapshot: fields.clone(),
                    current: fields.clone(),
                },
            );
        }

        if let Some(state) = states.get(path) {
            results.push(ExifData {
                path: path.clone(),
                fields: state.current.clone(),
                snapshot: state.snapshot.clone(),
                modified: state.is_modified(),
            });
        }
    }

    Ok(results)
}

#[derive(Deserialize)]
pub struct BatchUpdateRequest {
    pub paths: Vec<String>,
    pub field: String,
    pub value: String,
}

#[tauri::command]
pub fn update_exif_batch(
    request: BatchUpdateRequest,
    store: State<'_, ExifStore>,
) -> Result<Vec<ExifData>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for path in &request.paths {
        if let Some(state) = states.get_mut(path) {
            if request.value.is_empty() {
                state.current.remove(&request.field);
            } else {
                state.current.insert(request.field.clone(), request.value.clone());
            }
            results.push(ExifData {
                path: path.clone(),
                fields: state.current.clone(),
                snapshot: state.snapshot.clone(),
                modified: state.is_modified(),
            });
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn save_exif_batch(paths: Vec<String>, store: State<'_, ExifStore>) -> Result<Vec<ExifData>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for path in &paths {
        if let Some(state) = states.get_mut(path) {
            if !state.is_modified() {
                results.push(ExifData {
                    path: path.clone(),
                    fields: state.current.clone(),
                    snapshot: state.snapshot.clone(),
                    modified: false,
                });
                continue;
            }

            let file_path = Path::new(path);
            let mut metadata =
                Metadata::new_from_path(file_path).unwrap_or_else(|_| Metadata::new());

            for (name, value) in &state.current {
                if let Some(tag) = name_to_tag(name, value) {
                    metadata.set_tag(tag);
                }
            }

            match metadata.write_to_file(file_path) {
                Ok(_) => {
                    state.snapshot = state.current.clone();
                    results.push(ExifData {
                        path: path.clone(),
                        fields: state.current.clone(),
                        snapshot: state.snapshot.clone(),
                        modified: false,
                    });
                }
                Err(e) => errors.push(format!("{}: {}", path, e)),
            }
        }
    }

    if !errors.is_empty() {
        return Err(format!("Some files failed to save: {}", errors.join(", ")));
    }

    Ok(results)
}

#[tauri::command]
pub fn reset_exif_batch(paths: Vec<String>, store: State<'_, ExifStore>) -> Result<Vec<ExifData>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for path in &paths {
        if let Some(state) = states.get_mut(path) {
            state.current = state.snapshot.clone();
            results.push(ExifData {
                path: path.clone(),
                fields: state.current.clone(),
                snapshot: state.snapshot.clone(),
                modified: false,
            });
        }
    }

    Ok(results)
}

#[derive(Deserialize)]
pub struct RestoreSnapshotEntry {
    pub path: String,
    pub snapshot: HashMap<String, String>,
}

#[tauri::command]
pub fn restore_snapshot_batch(
    entries: Vec<RestoreSnapshotEntry>,
    store: State<'_, ExifStore>,
) -> Result<Vec<ExifData>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for entry in &entries {
        if let Some(state) = states.get_mut(&entry.path) {
            state.snapshot = entry.snapshot.clone();
            results.push(ExifData {
                path: entry.path.clone(),
                fields: state.current.clone(),
                snapshot: state.snapshot.clone(),
                modified: state.is_modified(),
            });
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn get_modified_files(store: State<'_, ExifStore>) -> Result<Vec<String>, String> {
    let states = store.states.lock().map_err(|e| e.to_string())?;
    let modified: Vec<String> = states
        .iter()
        .filter(|(_, s)| s.is_modified())
        .map(|(p, _)| p.clone())
        .collect();
    Ok(modified)
}
