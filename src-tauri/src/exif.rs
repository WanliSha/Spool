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
    pub fields: HashMap<String, String>,
    pub snapshot: HashMap<String, String>,
    pub modified: bool,
}

struct ExifState {
    snapshot: HashMap<String, String>,
    current: HashMap<String, String>,
    undo_stack: Vec<(String, Option<String>)>,
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

/// Extract the tag name from an ExifTag variant (e.g. "Make", "Model")
fn tag_name(tag: &ExifTag) -> String {
    let debug = format!("{:?}", tag);
    debug.split('(').next().unwrap_or("Unknown").to_string()
}

/// Convert DMS rational values to decimal degrees
fn dms_to_decimal(rationals: &[uR64]) -> Option<f64> {
    if rationals.len() < 3 {
        return None;
    }
    let deg = rationals[0].nominator as f64 / rationals[0].denominator.max(1) as f64;
    let min = rationals[1].nominator as f64 / rationals[1].denominator.max(1) as f64;
    let sec = rationals[2].nominator as f64 / rationals[2].denominator.max(1) as f64;
    Some(deg + min / 60.0 + sec / 3600.0)
}

/// Convert a single rational to float
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

/// Extract a readable string value from an ExifTag
fn extract_value(tag: &ExifTag) -> Option<String> {
    match tag {
        // String tags
        ExifTag::Make(v) | ExifTag::Model(v) | ExifTag::LensMake(v)
        | ExifTag::LensModel(v) | ExifTag::Software(v) | ExifTag::Artist(v)
        | ExifTag::Copyright(v) | ExifTag::ImageDescription(v)
        | ExifTag::DateTimeOriginal(v) | ExifTag::CreateDate(v) | ExifTag::ModifyDate(v)
        | ExifTag::GPSLatitudeRef(v) | ExifTag::GPSLongitudeRef(v) => {
            if v.is_empty() { None } else { Some(v.clone()) }
        }
        // GPS coordinates (DMS rational → decimal)
        ExifTag::GPSLatitude(v) => dms_to_decimal(v).map(|d| format!("{:.6}", d)),
        ExifTag::GPSLongitude(v) => dms_to_decimal(v).map(|d| format!("{:.6}", d)),
        // Single rational → float
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
        // u16 tags
        ExifTag::ISO(v) => v.first().map(|n| n.to_string()),
        ExifTag::FocalLengthIn35mmFormat(v) => v.first().map(|n| n.to_string()),
        ExifTag::ExposureProgram(v) | ExifTag::MeteringMode(v)
        | ExifTag::Flash(v) | ExifTag::WhiteBalance(v)
        | ExifTag::Orientation(v) => v.first().map(|n| n.to_string()),
        ExifTag::ImageWidth(v) | ExifTag::ImageHeight(v) => v.first().map(|n| n.to_string()),
        // GPS altitude ref (u8)
        ExifTag::GPSAltitudeRef(v) => v.first().map(|n| n.to_string()),
        // User comment (bytes)
        ExifTag::UserComment(v) => {
            let s = String::from_utf8_lossy(v).to_string();
            if s.is_empty() { None } else { Some(s) }
        }
        _ => None,
    }
}

/// Convert a decimal degree float to degrees/minutes/seconds rational format
fn decimal_to_dms(decimal: f64) -> Vec<uR64> {
    let abs = decimal.abs();
    let degrees = abs.floor() as u32;
    let minutes_float = (abs - degrees as f64) * 60.0;
    let minutes = minutes_float.floor() as u32;
    let seconds_float = (minutes_float - minutes as f64) * 60.0;
    // Store seconds with 1000x precision
    let seconds_num = (seconds_float * 1000.0).round() as u32;
    vec![
        uR64 { nominator: degrees, denominator: 1 },
        uR64 { nominator: minutes, denominator: 1 },
        uR64 { nominator: seconds_num, denominator: 1000 },
    ]
}

/// Convert a float string to a single rational value (numerator/denominator)
fn float_to_rational(value: &str) -> Option<Vec<uR64>> {
    let f: f64 = value.parse().ok()?;
    // Use 10000 as denominator for good precision
    let num = (f * 10000.0).round() as u32;
    Some(vec![uR64 { nominator: num, denominator: 10000 }])
}

/// Parse a u16 value from string
fn parse_u16(value: &str) -> Option<Vec<u16>> {
    // Handle values like "100" or "400"
    let v: u16 = value.parse().ok()?;
    Some(vec![v])
}

/// Build an ExifTag from a field name and string value
fn name_to_tag(name: &str, value: &str) -> Option<ExifTag> {
    match name {
        // String tags
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
        // GPS rational tags (decimal degrees → DMS)
        "GPSLatitude" => {
            let f: f64 = value.parse().ok()?;
            Some(ExifTag::GPSLatitude(decimal_to_dms(f)))
        }
        "GPSLongitude" => {
            let f: f64 = value.parse().ok()?;
            Some(ExifTag::GPSLongitude(decimal_to_dms(f)))
        }
        "GPSAltitude" => float_to_rational(value).map(ExifTag::GPSAltitude),
        // Exposure rational tags
        "ExposureTime" => float_to_rational(value).map(ExifTag::ExposureTime),
        "FNumber" => float_to_rational(value).map(ExifTag::FNumber),
        "FocalLength" => float_to_rational(value).map(ExifTag::FocalLength),
        // Integer tags
        "ISO" => parse_u16(value).map(ExifTag::ISO),
        "Orientation" => parse_u16(value).map(ExifTag::Orientation),
        _ => None,
    }
}

fn read_exif_fields(path: &Path) -> HashMap<String, String> {
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

#[tauri::command]
pub fn read_exif(path: String, store: State<'_, ExifStore>) -> Result<ExifData, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err("File not found".to_string());
    }

    let fields = read_exif_fields(file_path);
    let state = ExifState {
        snapshot: fields.clone(),
        current: fields.clone(),
        undo_stack: Vec::new(),
    };

    let data = ExifData {
        fields: fields.clone(),
        snapshot: fields.clone(),
        modified: false,
    };

    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    states.insert(path, state);

    Ok(data)
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub path: String,
    pub field: String,
    pub value: String,
}

#[tauri::command]
pub fn update_exif(request: UpdateRequest, store: State<'_, ExifStore>) -> Result<ExifData, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let state = states.get_mut(&request.path).ok_or("File not loaded")?;

    let previous = state.current.get(&request.field).cloned();
    state.undo_stack.push((request.field.clone(), previous));

    if request.value.is_empty() {
        state.current.remove(&request.field);
    } else {
        state.current.insert(request.field, request.value);
    }

    Ok(ExifData {
        fields: state.current.clone(),
        snapshot: state.snapshot.clone(),
        modified: state.is_modified(),
    })
}

#[tauri::command]
pub fn undo_exif(path: String, store: State<'_, ExifStore>) -> Result<ExifData, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let state = states.get_mut(&path).ok_or("File not loaded")?;

    if let Some((field, previous_value)) = state.undo_stack.pop() {
        match previous_value {
            Some(val) => {
                state.current.insert(field, val);
            }
            None => {
                state.current.remove(&field);
            }
        }
    }

    Ok(ExifData {
        fields: state.current.clone(),
        snapshot: state.snapshot.clone(),
        modified: state.is_modified(),
    })
}

#[tauri::command]
pub fn reset_exif(path: String, store: State<'_, ExifStore>) -> Result<ExifData, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let state = states.get_mut(&path).ok_or("File not loaded")?;

    state.current = state.snapshot.clone();
    state.undo_stack.clear();

    Ok(ExifData {
        fields: state.current.clone(),
        snapshot: state.snapshot.clone(),
        modified: false,
    })
}

#[tauri::command]
pub fn reset_all_exif(store: State<'_, ExifStore>) -> Result<Vec<String>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut affected = Vec::new();

    for (path, state) in states.iter_mut() {
        if state.is_modified() {
            state.current = state.snapshot.clone();
            state.undo_stack.clear();
            affected.push(path.clone());
        }
    }

    Ok(affected)
}

#[tauri::command]
pub fn save_exif(path: String, store: State<'_, ExifStore>) -> Result<ExifData, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let state = states.get_mut(&path).ok_or("File not loaded")?;

    if !state.is_modified() {
        return Ok(ExifData {
            fields: state.current.clone(),
            snapshot: state.snapshot.clone(),
            modified: false,
        });
    }

    let file_path = Path::new(&path);
    let mut metadata =
        Metadata::new_from_path(file_path).unwrap_or_else(|_| Metadata::new());

    for (name, value) in &state.current {
        if let Some(tag) = name_to_tag(name, value) {
            metadata.set_tag(tag);
        }
    }

    metadata
        .write_to_file(file_path)
        .map_err(|e| format!("Cannot write EXIF: {e}"))?;

    state.snapshot = state.current.clone();
    state.undo_stack.clear();

    Ok(ExifData {
        fields: state.current.clone(),
        snapshot: state.snapshot.clone(),
        modified: false,
    })
}

#[tauri::command]
pub fn save_all_exif(store: State<'_, ExifStore>) -> Result<Vec<String>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut saved = Vec::new();
    let mut errors = Vec::new();

    let modified_paths: Vec<String> = states
        .iter()
        .filter(|(_, s)| s.is_modified())
        .map(|(p, _)| p.clone())
        .collect();

    for path in &modified_paths {
        let state = states.get(path).unwrap();
        let file_path = Path::new(path);

        let mut metadata =
            Metadata::new_from_path(file_path).unwrap_or_else(|_| Metadata::new());

        for (name, value) in &state.current {
            if let Some(tag) = name_to_tag(name, value) {
                metadata.set_tag(tag);
            }
        }

        match metadata.write_to_file(file_path) {
            Ok(_) => saved.push(path.clone()),
            Err(e) => errors.push(format!("{}: {}", path, e)),
        }
    }

    for path in &saved {
        if let Some(state) = states.get_mut(path) {
            state.snapshot = state.current.clone();
            state.undo_stack.clear();
        }
    }

    if !errors.is_empty() {
        return Err(format!("Some files failed to save: {}", errors.join(", ")));
    }

    Ok(saved)
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
