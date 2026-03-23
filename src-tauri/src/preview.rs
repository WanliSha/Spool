use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

const PREVIEW_SIZE: u32 = 2048;
const JPEG_QUALITY: u8 = 85;

pub struct PreviewCache {
    cache: Mutex<HashMap<String, String>>,
}

impl PreviewCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }
}

#[tauri::command]
pub fn get_preview(path: String, store: State<'_, PreviewCache>) -> Result<String, String> {
    // Check cache
    {
        let cache = store.cache.lock().map_err(|e| e.to_string())?;
        if let Some(data) = cache.get(&path) {
            return Ok(data.clone());
        }
    }

    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err("File not found".to_string());
    }

    let img = image::open(file_path).map_err(|e| format!("Cannot open image: {e}"))?;

    // Resize to fit within PREVIEW_SIZE, maintaining aspect ratio
    let preview = if img.width() > PREVIEW_SIZE || img.height() > PREVIEW_SIZE {
        img.resize(PREVIEW_SIZE, PREVIEW_SIZE, FilterType::Lanczos3)
    } else {
        img
    };

    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY);
    preview
        .write_with_encoder(encoder)
        .map_err(|e| format!("Cannot encode preview: {e}"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);

    // Store in cache
    {
        let mut cache = store.cache.lock().map_err(|e| e.to_string())?;
        cache.insert(path, b64.clone());
    }

    Ok(b64)
}
