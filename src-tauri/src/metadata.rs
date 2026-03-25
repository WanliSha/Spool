use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

use spool_core::metadata::{read_all, write_all};

#[derive(Clone, Serialize)]
pub struct MetadataFields {
    pub path: String,
    pub fields: HashMap<String, String>,
    pub snapshot: HashMap<String, String>,
    pub modified: bool,
}

struct MetadataState {
    snapshot: HashMap<String, String>,
    current: HashMap<String, String>,
}

impl MetadataState {
    fn is_modified(&self) -> bool {
        self.current != self.snapshot
    }
}

pub struct MetadataStore {
    states: Mutex<HashMap<String, MetadataState>>,
}

impl MetadataStore {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }
}

// === Tauri Commands ===

#[tauri::command]
pub fn read_metadata(path: String, store: State<'_, MetadataStore>) -> Result<MetadataFields, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err("File not found".to_string());
    }

    let fields = read_all(file_path);
    let state = MetadataState {
        snapshot: fields.clone(),
        current: fields.clone(),
    };

    let data = MetadataFields {
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
pub fn get_metadata_batch(paths: Vec<String>, store: State<'_, MetadataStore>) -> Result<Vec<MetadataFields>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for path in &paths {
        if !states.contains_key(path) {
            let file_path = Path::new(path);
            if !file_path.exists() {
                continue;
            }
            let fields = read_all(file_path);
            states.insert(
                path.clone(),
                MetadataState {
                    snapshot: fields.clone(),
                    current: fields.clone(),
                },
            );
        }

        if let Some(state) = states.get(path) {
            results.push(MetadataFields {
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
pub fn update_metadata_batch(
    request: BatchUpdateRequest,
    store: State<'_, MetadataStore>,
) -> Result<Vec<MetadataFields>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for path in &request.paths {
        if let Some(state) = states.get_mut(path) {
            if request.value.is_empty() {
                state.current.remove(&request.field);
            } else {
                state.current.insert(request.field.clone(), request.value.clone());
            }
            results.push(MetadataFields {
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
pub fn save_metadata_batch(paths: Vec<String>, store: State<'_, MetadataStore>) -> Result<Vec<MetadataFields>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for path in &paths {
        if let Some(state) = states.get_mut(path) {
            if !state.is_modified() {
                results.push(MetadataFields {
                    path: path.clone(),
                    fields: state.current.clone(),
                    snapshot: state.snapshot.clone(),
                    modified: false,
                });
                continue;
            }

            let file_path = Path::new(path);
            match write_all(file_path, &state.current) {
                Ok(_) => {
                    state.snapshot = state.current.clone();
                    results.push(MetadataFields {
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
pub fn reset_metadata_batch(paths: Vec<String>, store: State<'_, MetadataStore>) -> Result<Vec<MetadataFields>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for path in &paths {
        if let Some(state) = states.get_mut(path) {
            state.current = state.snapshot.clone();
            results.push(MetadataFields {
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
pub fn restore_metadata_snapshot_batch(
    entries: Vec<RestoreSnapshotEntry>,
    store: State<'_, MetadataStore>,
) -> Result<Vec<MetadataFields>, String> {
    let mut states = store.states.lock().map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for entry in &entries {
        if let Some(state) = states.get_mut(&entry.path) {
            state.snapshot = entry.snapshot.clone();
            results.push(MetadataFields {
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
pub fn get_modified_metadata_files(store: State<'_, MetadataStore>) -> Result<Vec<String>, String> {
    let states = store.states.lock().map_err(|e| e.to_string())?;
    let modified: Vec<String> = states
        .iter()
        .filter(|(_, s)| s.is_modified())
        .map(|(p, _)| p.clone())
        .collect();
    Ok(modified)
}
