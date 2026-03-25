use clap::{Parser, Subcommand};
use image::imageops;
use spool_core::metadata;
use std::collections::HashMap;
use std::path::Path;
use std::process;

#[derive(Parser)]
#[command(name = "spool", about = "Photo metadata editor for film photographers")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List supported image files in a directory
    List {
        /// Directory to scan
        path: String,
        /// Scan subdirectories recursively
        #[arg(short, long)]
        recursive: bool,
    },
    /// Read metadata from a photo
    Get {
        /// Image file path
        file: String,
        /// Specific field to read (omit to read all)
        field: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Generate a rotated preview (view-only, does not modify the original file)
    Preview {
        /// Image file path
        file: String,
        /// Rotation: 90, 180, 270 (clockwise). If omitted, auto-corrects using EXIF Orientation.
        #[arg(short, long)]
        rotate: Option<u16>,
    },
    /// Write metadata to one or more photos
    Set {
        /// Image file path(s)
        #[arg(required = true, num_args = 1..)]
        files: Vec<String>,
        /// Field name (requires exactly one file, omit when using --json)
        #[arg(short, long)]
        field: Option<String>,
        /// Field value (requires --field)
        #[arg(short, long)]
        value: Option<String>,
        /// Set multiple fields as JSON object: '{"Field":"value",...}'
        #[arg(long)]
        json: Option<String>,
    },
}

const DATE_FIELDS: &[&str] = &["DateTaken", "DateCreated", "DateModified"];

/// Normalize date input to EXIF format (YYYY:MM:DD HH:MM:SS)
fn normalize_date(input: &str) -> String {
    let trimmed = input.trim();

    // Already in EXIF format with colons in date part
    if trimmed.len() >= 10 && &trimmed[4..5] == ":" && &trimmed[7..8] == ":" {
        return trimmed.to_string();
    }

    // Replace dashes with colons in date part
    // Handles: YYYY-MM-DD, YYYY-MM-DD HH:MM, YYYY-MM-DD HH:MM:SS
    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let date_part = parts[0].replace('-', ":");

    if parts.len() == 1 {
        // Date only → default to 12:00:00
        format!("{} 12:00:00", date_part)
    } else {
        let time_part = parts[1];
        let colons = time_part.chars().filter(|c| *c == ':').count();
        if colons == 0 {
            format!("{} {}:00:00", date_part, time_part)
        } else if colons == 1 {
            // HH:MM → HH:MM:00
            format!("{} {}:00", date_part, time_part)
        } else {
            // HH:MM:SS → as-is
            format!("{} {}", date_part, time_part)
        }
    }
}

/// Build the fields to write, applying date normalization
fn build_update_fields(
    field: Option<String>,
    value: Option<String>,
    json: Option<String>,
) -> HashMap<String, String> {
    let mut updates = HashMap::new();

    if let Some(json_str) = json {
        let new_fields: HashMap<String, String> = match serde_json::from_str(&json_str) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error: invalid JSON: {}", e);
                process::exit(1);
            }
        };
        for (k, v) in new_fields {
            let val = if DATE_FIELDS.contains(&k.as_str()) {
                normalize_date(&v)
            } else {
                v
            };
            updates.insert(k, val);
        }
    } else {
        let field_name = match field {
            Some(f) => f,
            None => {
                eprintln!("Error: specify --field and --value, or use --json");
                process::exit(1);
            }
        };
        let field_value = match value {
            Some(v) => v,
            None => {
                eprintln!("Error: specify --value for field '{}'", field_name);
                process::exit(1);
            }
        };
        let val = if DATE_FIELDS.contains(&field_name.as_str()) {
            normalize_date(&field_value)
        } else {
            field_value
        };
        updates.insert(field_name, val);
    }

    updates
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { path, recursive } => {
            let dir = Path::new(&path);
            if !dir.exists() {
                eprintln!("Error: path not found: {}", path);
                process::exit(1);
            }
            let mut results = Vec::new();
            spool_core::collect_files(dir, recursive, &mut results);
            results.sort_by(|a, b| a.filename.cmp(&b.filename));
            for entry in &results {
                println!("{}", entry.path);
            }
        }
        Commands::Get { file, field, json } => {
            let path = Path::new(&file);
            if !path.exists() {
                eprintln!("Error: file not found: {}", file);
                process::exit(1);
            }
            if !spool_core::is_supported(path) {
                eprintln!("Error: unsupported file format: {}", file);
                process::exit(1);
            }
            let fields = metadata::read_all(path);
            if let Some(field_name) = field {
                match fields.get(&field_name) {
                    Some(value) => println!("{}", value),
                    None => {
                        eprintln!("Field not found: {}", field_name);
                        process::exit(1);
                    }
                }
            } else if json {
                let json_str = serde_json::to_string_pretty(&fields).unwrap();
                println!("{}", json_str);
            } else {
                let mut keys: Vec<&String> = fields.keys().collect();
                keys.sort();
                for key in keys {
                    println!("{}: {}", key, fields[key]);
                }
            }
        }
        Commands::Preview { file, rotate } => {
            let path = Path::new(&file);
            if !path.exists() {
                eprintln!("Error: file not found: {}", file);
                process::exit(1);
            }

            let img = match image::open(path) {
                Ok(img) => img,
                Err(e) => {
                    eprintln!("Error: cannot open image: {}", e);
                    process::exit(1);
                }
            };

            let rotated = if let Some(degrees) = rotate {
                // Explicit rotation
                match degrees {
                    90 => img.rotate90(),
                    180 => img.rotate180(),
                    270 => img.rotate270(),
                    _ => {
                        eprintln!("Error: --rotate must be 90, 180, or 270");
                        process::exit(1);
                    }
                }
            } else {
                // Auto-correct using EXIF Orientation
                let fields = metadata::read_all(path);
                let orientation: u16 = fields
                    .get("Orientation")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1);
                match orientation {
                    1 => img,                                                    // Normal
                    2 => image::DynamicImage::ImageRgba8(imageops::flip_horizontal(&img)),  // Flipped horizontally
                    3 => img.rotate180(),                                        // Rotated 180
                    4 => image::DynamicImage::ImageRgba8(imageops::flip_vertical(&img)),    // Flipped vertically
                    5 => {                                                       // Transposed
                        let flipped = imageops::flip_horizontal(&img);
                        image::DynamicImage::ImageRgba8(flipped).rotate90()
                    }
                    6 => img.rotate90(),                                         // Rotated 90 CW
                    7 => {                                                       // Transversed
                        let flipped = imageops::flip_horizontal(&img);
                        image::DynamicImage::ImageRgba8(flipped).rotate270()
                    }
                    8 => img.rotate270(),                                        // Rotated 270 CW
                    _ => img,
                }
            };

            // Save to temp file
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let tmp_path = std::env::temp_dir().join(format!("spool-preview-{}.jpg", stem));
            if let Err(e) = rotated.save(&tmp_path) {
                eprintln!("Error: cannot save preview: {}", e);
                process::exit(1);
            }
            // Output the path so Claude can read it
            println!("{}", tmp_path.display());
        }
        Commands::Set { files, field, value, json } => {
            // Build the update fields once
            let updates = build_update_fields(field, value, json);

            let mut errors = Vec::new();

            for file in &files {
                let path = Path::new(file);
                if !path.exists() {
                    errors.push(format!("{}: file not found", file));
                    continue;
                }
                if !spool_core::is_supported(path) {
                    errors.push(format!("{}: unsupported format", file));
                    continue;
                }

                // Read existing metadata, merge updates, write back
                let mut fields = metadata::read_all(path);
                for (k, v) in &updates {
                    fields.insert(k.clone(), v.clone());
                }

                if let Err(e) = metadata::write_all(path, &fields) {
                    errors.push(format!("{}: {}", file, e));
                }
            }

            if !errors.is_empty() {
                for err in &errors {
                    eprintln!("Error: {}", err);
                }
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_date_only() {
        assert_eq!(normalize_date("2024-12-25"), "2024:12:25 12:00:00");
    }

    #[test]
    fn test_normalize_date_with_minutes() {
        assert_eq!(normalize_date("2024-12-25 14:30"), "2024:12:25 14:30:00");
    }

    #[test]
    fn test_normalize_date_with_seconds() {
        assert_eq!(normalize_date("2024-12-25 14:30:00"), "2024:12:25 14:30:00");
    }

    #[test]
    fn test_normalize_date_exif_passthrough() {
        assert_eq!(normalize_date("2024:12:25 14:30:00"), "2024:12:25 14:30:00");
    }
}
