pub mod exif;
pub mod iptc;
pub mod metadata;
pub mod xmp;

use serde::Serialize;
use std::fs;
use std::path::Path;

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "tif", "tiff", "png", "webp", "bmp",
    "cr2", "cr3", "nef", "arw", "raf", "dng", "orf", "rw2",
];

#[derive(Serialize)]
pub struct FileEntry {
    pub path: String,
    pub filename: String,
    pub size: u64,
}

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn collect_files(path: &Path, recursive: bool, results: &mut Vec<FileEntry>) {
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
