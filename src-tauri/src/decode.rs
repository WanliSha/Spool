use image::DynamicImage;
use rawler::analyze;
use rawler::decoders::RawDecodeParams;
use std::fs;
use std::io::Cursor;
use std::path::Path;

const RAW_EXTENSIONS: &[&str] = &[
    "cr2", "cr3", "nef", "arw", "raf", "dng", "orf", "rw2", "pef", "srw", "erf",
];

fn is_raw(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| RAW_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Extract the largest valid embedded JPEG from a RAW file.
/// Uses proper JPEG segment parsing to find real boundaries.
fn extract_largest_jpeg(path: &Path) -> Result<DynamicImage, String> {
    let data = fs::read(path).map_err(|e| format!("Cannot read file: {e}"))?;
    let mut candidates: Vec<(usize, usize)> = Vec::new();

    let mut i = 0;
    while i + 2 < data.len() {
        if data[i] == 0xFF && data[i + 1] == 0xD8 && data[i + 2] == 0xFF {
            if let Some(end) = parse_jpeg_end(&data, i) {
                let size = end - i;
                if size > 50000 {
                    candidates.push((i, size));
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }

    // Sort by size descending, try largest first
    candidates.sort_by(|a, b| b.1.cmp(&a.1));

    for (offset, size) in candidates.iter().take(5) {
        let jpeg_data = &data[*offset..*offset + *size];
        let cursor = Cursor::new(jpeg_data);
        if let Ok(img) = image::load(cursor, image::ImageFormat::Jpeg) {
            return Ok(img);
        }
    }

    Err("No valid embedded JPEG found".to_string())
}

/// Parse JPEG segment structure to find the real end (EOI marker).
/// Returns the byte position after the EOI marker.
fn parse_jpeg_end(data: &[u8], start: usize) -> Option<usize> {
    let mut j = start + 2;
    while j + 1 < data.len() {
        if data[j] != 0xFF {
            j += 1;
            continue;
        }
        let marker = data[j + 1];
        match marker {
            0x00 => { j += 2; } // Stuffed byte
            0xD9 => { return Some(j + 2); } // EOI — end of image
            0xD8 => { return None; } // Another SOI — nested, abort
            0xD0..=0xD7 => { j += 2; } // RST markers (no length)
            0xDA => {
                // SOS — Start of Scan: has a header, then entropy-coded data
                if j + 3 >= data.len() { return None; }
                let seg_len = u16::from_be_bytes([data[j + 2], data[j + 3]]) as usize;
                j += 2 + seg_len;
                // Scan entropy-coded segment: skip until next real marker
                while j + 1 < data.len() {
                    if data[j] == 0xFF && data[j + 1] != 0x00
                        && !(0xD0..=0xD7).contains(&data[j + 1])
                    {
                        break;
                    }
                    j += 1;
                }
            }
            _ => {
                // Marker with length field
                if j + 3 >= data.len() { return None; }
                let seg_len = u16::from_be_bytes([data[j + 2], data[j + 3]]) as usize;
                j += 2 + seg_len;
            }
        }
    }
    None
}

/// Decode an image for thumbnail
pub fn decode_thumbnail(path: &Path) -> Result<DynamicImage, String> {
    if is_raw(path) {
        // Try fast embedded JPEG extraction first
        if let Ok(img) = extract_largest_jpeg(path) {
            return Ok(img);
        }
        // Fall back to rawler
        let params = RawDecodeParams::default();
        if let Ok(img) = analyze::extract_thumbnail_pixels(path, &params) {
            return Ok(img);
        }
    }
    image::open(path).map_err(|e| format!("Cannot open image: {e}"))
}

/// Decode an image for preview
pub fn decode_preview(path: &Path) -> Result<DynamicImage, String> {
    if is_raw(path) {
        if let Ok(img) = extract_largest_jpeg(path) {
            return Ok(img);
        }
        let params = RawDecodeParams::default();
        if let Ok(img) = analyze::extract_preview_pixels(path, &params) {
            return Ok(img);
        }
    }
    image::open(path).map_err(|e| format!("Cannot open image: {e}"))
}

/// Decode an image at full resolution
pub fn decode_full(path: &Path) -> Result<DynamicImage, String> {
    if is_raw(path) {
        // For full resolution, try rawler first for best quality
        let params = RawDecodeParams::default();
        if let Ok(img) = analyze::extract_full_pixels(path, &params) {
            return Ok(img);
        }
        // Fall back to embedded JPEG
        if let Ok(img) = extract_largest_jpeg(path) {
            return Ok(img);
        }
    }
    image::open(path).map_err(|e| format!("Cannot open image: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dng_decode() {
        let path = Path::new("/Users/arthurwang/Downloads/IMG_2769.DNG");
        if !path.exists() {
            println!("DNG file not found, skipping");
            return;
        }
        println!("Testing thumbnail...");
        match decode_thumbnail(path) {
            Ok(img) => println!("Thumbnail OK: {}x{}", img.width(), img.height()),
            Err(e) => println!("Thumbnail FAILED: {e}"),
        }
        println!("Testing preview...");
        match decode_preview(path) {
            Ok(img) => println!("Preview OK: {}x{}", img.width(), img.height()),
            Err(e) => println!("Preview FAILED: {e}"),
        }
    }
}
