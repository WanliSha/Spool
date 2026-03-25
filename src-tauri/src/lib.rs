mod cf_import;
mod decode;
mod exif;
mod metadata;
mod preview;
mod settings;
mod thumbnail;

use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct FileEntry {
    path: String,
    filename: String,
    size: u64,
}

impl From<spool_core::FileEntry> for FileEntry {
    fn from(e: spool_core::FileEntry) -> Self {
        Self {
            path: e.path,
            filename: e.filename,
            size: e.size,
        }
    }
}

#[tauri::command]
fn scan_paths(
    paths: Vec<String>,
    recursive: Option<bool>,
    settings_store: tauri::State<'_, settings::SettingsStore>,
) -> Vec<FileEntry> {
    let recursive = recursive.unwrap_or_else(|| settings_store.get_recursive());
    let mut core_results = Vec::new();
    for p in &paths {
        spool_core::collect_files(Path::new(p), recursive, &mut core_results);
    }
    let mut results: Vec<FileEntry> = core_results.into_iter().map(FileEntry::from).collect();
    results.sort_by(|a, b| a.filename.cmp(&b.filename));
    results
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .manage(exif::ExifStore::new())
        .manage(metadata::MetadataStore::new())
        .manage(preview::PreviewCache::new())
        .manage(settings::SettingsStore::new())
        .invoke_handler(tauri::generate_handler![
            scan_paths,
            thumbnail::get_thumbnail,
            thumbnail::get_cache_stats,
            thumbnail::clear_cache,
            exif::read_exif,
            exif::get_exif_batch,
            exif::update_exif_batch,
            exif::save_exif_batch,
            exif::reset_exif_batch,
            exif::restore_snapshot_batch,
            exif::get_modified_files,
            metadata::read_metadata,
            metadata::get_metadata_batch,
            metadata::update_metadata_batch,
            metadata::save_metadata_batch,
            metadata::reset_metadata_batch,
            metadata::restore_metadata_snapshot_batch,
            metadata::get_modified_metadata_files,
            preview::get_preview,
            settings::load_settings,
            settings::save_settings,
            cf_import::parse_cf_json,
            cf_import::get_cf_preview,
            cf_import::write_cf_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
