use little_exif::metadata::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

use spool_core::exif::{name_to_tag, read_exif_fields_raw};

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
