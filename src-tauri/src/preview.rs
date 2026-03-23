use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

const QUICK_SIZE: u32 = 2048;
const QUICK_QUALITY: u8 = 85;
const FULL_QUALITY: u8 = 95;

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
pub fn get_preview(
    path: String,
    mode: Option<String>,
    store: State<'_, PreviewCache>,
) -> Result<String, String> {
    let mode = mode.unwrap_or_else(|| "quick".to_string());
    let cache_key = format!("{}:{}", path, mode);

    // Check cache
    {
        let cache = store.cache.lock().map_err(|e| e.to_string())?;
        if let Some(data) = cache.get(&cache_key) {
            return Ok(data.clone());
        }
    }

    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err("File not found".to_string());
    }

    let (output, quality) = if mode == "full" {
        let img = crate::decode::decode_full(file_path)?;
        (img, FULL_QUALITY)
    } else {
        let img = crate::decode::decode_preview(file_path)?;
        let resized = if img.width() > QUICK_SIZE || img.height() > QUICK_SIZE {
            img.resize(QUICK_SIZE, QUICK_SIZE, FilterType::Lanczos3)
        } else {
            img
        };
        (resized, QUICK_QUALITY)
    };

    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    output
        .write_with_encoder(encoder)
        .map_err(|e| format!("Cannot encode preview: {e}"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);

    // Store in cache
    {
        let mut cache = store.cache.lock().map_err(|e| e.to_string())?;
        cache.insert(cache_key, b64.clone());
    }

    Ok(b64)
}
