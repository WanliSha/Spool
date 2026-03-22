mod exif;
mod thumbnail;

use serde::Serialize;
use std::fs;
use std::path::Path;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "tif", "tiff", "png", "webp", "bmp",
    "cr2", "cr3", "nef", "arw", "raf", "dng", "orf", "rw2",
];

#[derive(Serialize)]
struct FileEntry {
    path: String,
    filename: String,
    size: u64,
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn collect_files(path: &Path, recursive: bool, results: &mut Vec<FileEntry>) {
    if path.is_file() {
        if is_supported(path) {
            if let Ok(metadata) = fs::metadata(path) {
                results.push(FileEntry {
                    path: path.to_string_lossy().to_string(),
                    filename: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    size: metadata.len(),
                });
            }
        }
    } else if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    collect_files(&entry_path, recursive, results);
                } else if recursive && entry_path.is_dir() {
                    collect_files(&entry_path, recursive, results);
                }
            }
        }
    }
}

#[tauri::command]
fn scan_paths(paths: Vec<String>, recursive: bool) -> Vec<FileEntry> {
    let mut results = Vec::new();
    for p in &paths {
        collect_files(Path::new(p), recursive, &mut results);
    }
    results.sort_by(|a, b| a.filename.cmp(&b.filename));
    results
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(exif::ExifStore::new())
        .invoke_handler(tauri::generate_handler![
            scan_paths,
            thumbnail::get_thumbnail,
            thumbnail::get_cache_stats,
            thumbnail::clear_cache,
            exif::read_exif,
            exif::update_exif,
            exif::undo_exif,
            exif::reset_exif,
            exif::reset_all_exif,
            exif::save_exif,
            exif::save_all_exif,
            exif::get_modified_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
