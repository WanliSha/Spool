use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const THUMBNAIL_SIZE: u32 = 320;
const JPEG_QUALITY: u8 = 80;
const MAX_CACHE_BYTES: u64 = 200 * 1024 * 1024; // 200 MB

fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("Spool").join("thumbnails"))
}

fn cache_key(path: &str, mtime: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    hasher.update(mtime.to_le_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_mtime(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn ensure_cache_dir() -> Result<PathBuf, String> {
    let dir = cache_dir().ok_or("Cannot determine cache directory")?;
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create cache dir: {e}"))?;
    Ok(dir)
}

fn generate_thumbnail(path: &Path) -> Result<Vec<u8>, String> {
    let img = crate::decode::decode_thumbnail(path)?;
    let thumbnail = img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Triangle);
    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY);
    thumbnail
        .write_with_encoder(encoder)
        .map_err(|e| format!("Cannot encode thumbnail: {e}"))?;
    Ok(buf)
}

fn evict_if_needed(dir: &Path) {
    let mut entries: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
    let mut total_size: u64 = 0;

    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();
                let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                total_size += size;
                entries.push((entry.path(), size, mtime));
            }
        }
    }

    if total_size <= MAX_CACHE_BYTES {
        return;
    }

    // Sort oldest first
    entries.sort_by_key(|(_, _, mtime)| *mtime);

    for (path, size, _) in &entries {
        if total_size <= MAX_CACHE_BYTES {
            break;
        }
        if fs::remove_file(path).is_ok() {
            total_size -= size;
        }
    }
}

#[tauri::command]
pub fn get_thumbnail(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err("File not found".to_string());
    }

    let dir = ensure_cache_dir()?;
    let mtime = get_mtime(file_path);
    let key = cache_key(&path, mtime);
    let cached_path = dir.join(format!("{key}.jpg"));

    // Cache hit
    if cached_path.exists() {
        let data = fs::read(&cached_path).map_err(|e| format!("Cannot read cache: {e}"))?;
        return Ok(base64::engine::general_purpose::STANDARD.encode(&data));
    }

    // Cache miss — generate
    let data = generate_thumbnail(file_path)?;
    let _ = fs::write(&cached_path, &data);
    evict_if_needed(&dir);

    Ok(base64::engine::general_purpose::STANDARD.encode(&data))
}

#[derive(Serialize)]
pub struct CacheStats {
    pub size_bytes: u64,
    pub file_count: u64,
}

#[tauri::command]
pub fn get_cache_stats() -> CacheStats {
    let dir = match cache_dir() {
        Some(d) => d,
        None => return CacheStats { size_bytes: 0, file_count: 0 },
    };

    let mut size_bytes: u64 = 0;
    let mut file_count: u64 = 0;

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                size_bytes += metadata.len();
                file_count += 1;
            }
        }
    }

    CacheStats { size_bytes, file_count }
}

#[tauri::command]
pub fn clear_cache() -> Result<(), String> {
    let dir = cache_dir().ok_or("Cannot determine cache directory")?;
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("Cannot clear cache: {e}"))?;
        fs::create_dir_all(&dir).map_err(|e| format!("Cannot recreate cache dir: {e}"))?;
    }
    Ok(())
}
