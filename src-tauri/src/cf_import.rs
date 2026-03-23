use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::metadata;

#[derive(Deserialize)]
struct CfJsonEntry {
    #[serde(rename = "ImageNumber")]
    image_number: String,
    #[serde(rename = "Make", default)]
    make: Option<String>,
    #[serde(rename = "Model", default)]
    model: Option<String>,
    #[serde(rename = "LensMake", default)]
    lens_make: Option<String>,
    #[serde(rename = "LensModel", default)]
    lens_model: Option<String>,
    #[serde(rename = "LensInfo", default)]
    lens_info: Option<String>,
    #[serde(rename = "FocalLength", default)]
    focal_length: Option<f64>,
    #[serde(rename = "FocalLengthIn35mmFormat", default)]
    focal_length_35mm: Option<String>,
    #[serde(rename = "ExposureTime", default)]
    exposure_time: Option<f64>,
    #[serde(rename = "FNumber", default)]
    f_number: Option<String>,
    #[serde(rename = "ISO", default)]
    iso: Option<f64>,
    #[serde(rename = "DateTimeOriginal", default)]
    date_time_original: Option<String>,
    #[serde(rename = "Description", default)]
    description: Option<String>,
    #[serde(rename = "Keywords", default)]
    keywords: Option<Vec<String>>,
    #[serde(rename = "GPSLatitude", default)]
    gps_latitude: Option<f64>,
    #[serde(rename = "GPSLongitude", default)]
    gps_longitude: Option<f64>,
    #[serde(rename = "GPSLatitudeRef", default)]
    gps_latitude_ref: Option<String>,
    #[serde(rename = "GPSLongitudeRef", default)]
    gps_longitude_ref: Option<String>,
    #[serde(rename = "UserComment", default)]
    user_comment: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct CfEntry {
    pub image_number: String,
    pub summary: String,
    pub fields: HashMap<String, String>,
}

fn entry_to_fields(e: &CfJsonEntry) -> HashMap<String, String> {
    let mut f = HashMap::new();

    if let Some(v) = &e.make { f.insert("CameraMake".into(), v.clone()); }
    if let Some(v) = &e.model { f.insert("CameraModel".into(), v.clone()); }
    if let Some(v) = &e.lens_make { f.insert("LensMake".into(), v.clone()); }
    if let Some(v) = &e.lens_model { f.insert("LensModel".into(), v.clone()); }
    if let Some(v) = &e.lens_info { f.insert("LensInfo".into(), v.clone()); }
    if let Some(v) = e.focal_length { f.insert("FocalLength".into(), v.to_string()); }
    if let Some(v) = &e.focal_length_35mm {
        // Extract number from "50 mm" format
        let num = v.trim_end_matches(" mm").trim();
        f.insert("FocalLength35mm".into(), num.to_string());
    }
    if let Some(v) = e.exposure_time { f.insert("ShutterSpeed".into(), v.to_string()); }
    if let Some(v) = &e.f_number { f.insert("Aperture".into(), v.clone()); }
    if let Some(v) = e.iso { f.insert("ISO".into(), (v as u32).to_string()); }
    if let Some(v) = &e.date_time_original { f.insert("DateTaken".into(), v.clone()); }
    if let Some(v) = &e.description { f.insert("Description".into(), v.clone()); }
    if let Some(v) = &e.keywords {
        if !v.is_empty() {
            f.insert("Keywords".into(), v.join(", "));
        }
    }
    if let Some(v) = e.gps_latitude { f.insert("GPSLatitude".into(), format!("{:.6}", v)); }
    if let Some(v) = e.gps_longitude { f.insert("GPSLongitude".into(), format!("{:.6}", v)); }
    if let Some(v) = &e.gps_latitude_ref { f.insert("GPSLatitudeRef".into(), v.clone()); }
    if let Some(v) = &e.gps_longitude_ref { f.insert("GPSLongitudeRef".into(), v.clone()); }
    if let Some(v) = &e.user_comment {
        if !v.is_empty() {
            f.insert("Comment".into(), v.clone());
        }
    }

    f
}

fn entry_summary(e: &CfJsonEntry) -> String {
    let mut parts = Vec::new();
    if let Some(v) = &e.f_number { parts.push(format!("f/{}", v)); }
    if let Some(v) = e.focal_length { parts.push(format!("{}mm", v)); }
    if let Some(v) = &e.date_time_original { parts.push(v.clone()); }
    parts.join("  ")
}

#[tauri::command]
pub fn parse_cf_json(folder_path: String) -> Result<Vec<CfEntry>, String> {
    let json_path = Path::new(&folder_path).join("film.json");
    if !json_path.exists() {
        return Err("No film.json found in the selected folder".into());
    }

    let data = fs::read_to_string(&json_path)
        .map_err(|e| format!("Cannot read film.json: {e}"))?;

    let entries: Vec<CfJsonEntry> = serde_json::from_str(&data)
        .map_err(|e| format!("Invalid JSON format: {e}"))?;

    let result: Vec<CfEntry> = entries
        .iter()
        .map(|e| CfEntry {
            image_number: e.image_number.clone(),
            summary: entry_summary(e),
            fields: entry_to_fields(e),
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub fn get_cf_preview(folder_path: String, image_number: String) -> Result<Option<String>, String> {
    let preview_path = Path::new(&folder_path).join(format!("{}.jpeg", image_number));
    if !preview_path.exists() {
        return Ok(None);
    }

    let data = fs::read(&preview_path)
        .map_err(|e| format!("Cannot read preview: {e}"))?;

    Ok(Some(base64::engine::general_purpose::STANDARD.encode(&data)))
}

#[derive(Deserialize)]
pub struct CfWritePair {
    pub photo_path: String,
    pub fields: HashMap<String, String>,
}

#[tauri::command]
pub fn write_cf_metadata(pairs: Vec<CfWritePair>) -> Result<usize, String> {
    let mut written = 0;
    let mut errors = Vec::new();

    for pair in &pairs {
        let path = Path::new(&pair.photo_path);
        match metadata::write_all(path, &pair.fields) {
            Ok(_) => written += 1,
            Err(e) => errors.push(format!("{}: {}", pair.photo_path, e)),
        }
    }

    if !errors.is_empty() {
        return Err(format!(
            "Wrote {}/{} files. Errors: {}",
            written,
            pairs.len(),
            errors.join(", ")
        ));
    }

    Ok(written)
}
