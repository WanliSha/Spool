use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

#[derive(Clone, Serialize)]
pub struct ExifData {
    pub fields: HashMap<String, String>,
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

/// Extract the string value from a string-type ExifTag
fn extract_string_value(tag: &ExifTag) -> Option<String> {
    // For string tags, the inner value is a String
    // For numeric tags, the inner value is a Vec<u8/u16/u32>
    // We use Debug format and parse it
    let debug = format!("{:?}", tag);

    // String tags look like: Make("Canon")
    // Numeric tags look like: ISO([100])
    // Rational tags look like: FNumber([(56, 10)])

    if let Some(start) = debug.find('(') {
        let inner = &debug[start + 1..debug.len() - 1];

        // String value: "value"
        if inner.starts_with('"') && inner.ends_with('"') {
            let val = inner[1..inner.len() - 1].to_string();
            if !val.is_empty() {
                return Some(val);
            }
            return None;
        }

        // Vec value: [1, 2, 3] or [(1, 2)]
        if inner.starts_with('[') && inner.ends_with(']') {
            let vec_inner = &inner[1..inner.len() - 1].trim();
            if vec_inner.is_empty() {
                return None;
            }
            return Some(vec_inner.to_string());
        }

        if !inner.is_empty() {
            return Some(inner.to_string());
        }
    }
    None
}

/// Build an ExifTag from a field name and string value
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
            if let Some(value) = extract_string_value(found_tag) {
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
pub fn save_exif(path: String, store: State<'_, ExifStore>) -> Result<(), String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let state = states.get_mut(&path).ok_or("File not loaded")?;

    if !state.is_modified() {
        return Ok(());
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

    Ok(())
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
