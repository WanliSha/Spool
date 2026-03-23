use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub recursive_folder_loading: bool,
    pub cache_size_limit_mb: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            recursive_folder_loading: false,
            cache_size_limit_mb: 200,
        }
    }
}

pub struct SettingsStore {
    settings: Mutex<AppSettings>,
}

impl SettingsStore {
    pub fn new() -> Self {
        let settings = load_from_disk().unwrap_or_default();
        Self {
            settings: Mutex::new(settings),
        }
    }

    pub fn get_recursive(&self) -> bool {
        self.settings.lock().unwrap().recursive_folder_loading
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("Spool").join("settings.json"))
}

fn load_from_disk() -> Option<AppSettings> {
    let path = config_path()?;
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_to_disk(settings: &AppSettings) -> Result<(), String> {
    let path = config_path().ok_or("Cannot determine config directory")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create config dir: {e}"))?;
    }
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| format!("Cannot write settings: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn load_settings(store: State<'_, SettingsStore>) -> Result<AppSettings, String> {
    let settings = store.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn save_settings(
    settings: AppSettings,
    store: State<'_, SettingsStore>,
) -> Result<(), String> {
    save_to_disk(&settings)?;
    let mut current = store.settings.lock().map_err(|e| e.to_string())?;
    *current = settings;
    Ok(())
}
