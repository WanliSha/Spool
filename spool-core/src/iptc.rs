use std::collections::HashMap;
use std::fs;
use std::path::Path;

// IPTC-IIM dataset numbers (record 2)
const IPTC_HEADLINE: u8 = 105;
const IPTC_CAPTION: u8 = 120;
const IPTC_BYLINE: u8 = 80;
const IPTC_BYLINE_TITLE: u8 = 85;
const IPTC_COPYRIGHT: u8 = 116;
const IPTC_KEYWORDS: u8 = 25;
const IPTC_CITY: u8 = 90;
const IPTC_STATE: u8 = 95;
const IPTC_COUNTRY: u8 = 101;
const IPTC_COUNTRY_CODE: u8 = 100;
const IPTC_CREDIT: u8 = 110;
const IPTC_SOURCE: u8 = 115;
const IPTC_DATE_CREATED: u8 = 55;
const IPTC_INSTRUCTIONS: u8 = 40;

/// Read IPTC metadata from a JPEG file
pub fn read_iptc(path: &Path) -> HashMap<String, String> {
    let mut fields = HashMap::new();

    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => return fields,
    };

    let iptc_data = match extract_iptc_data(&data) {
        Some(d) => d,
        None => return fields,
    };

    parse_iptc_records(&iptc_data, &mut fields);
    fields
}

/// Write IPTC metadata to a JPEG file
pub fn write_iptc(path: &Path, fields: &HashMap<String, String>) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Cannot read file: {e}"))?;

    let iptc_segment = build_iptc_segment(fields);
    let new_data = replace_or_insert_iptc(&data, &iptc_segment)?;
    fs::write(path, new_data).map_err(|e| format!("Cannot write file: {e}"))?;

    Ok(())
}

/// Extract IPTC data from JPEG APP13 segment
fn extract_iptc_data(data: &[u8]) -> Option<Vec<u8>> {
    let photoshop_marker = b"Photoshop 3.0\0";
    let mut i = 2; // Skip SOI

    while i + 3 < data.len() {
        if data[i] == 0xFF && data[i + 1] == 0xED {
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            let seg_end = i + 2 + seg_len;
            if seg_end <= data.len() {
                let seg_data = &data[i + 4..seg_end];
                // Look for IPTC-NAA record (8BIM marker with type 0x0404)
                if let Some(iptc_start) = find_iptc_in_photoshop(seg_data) {
                    return Some(iptc_start.to_vec());
                }
                // If no Photoshop wrapper, try parsing directly as IPTC
                if seg_data.len() > 3 && seg_data[0] == 0x1C {
                    return Some(seg_data.to_vec());
                }
            }
            i = seg_end;
        } else if data[i] == 0xFF && data[i + 1] == 0xDA {
            break;
        } else if data[i] == 0xFF && data[i + 1] >= 0xE0 {
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            i += 2 + seg_len;
        } else {
            i += 1;
        }
    }
    None
}

/// Find IPTC data within Photoshop 3.0 resource block
fn find_iptc_in_photoshop(data: &[u8]) -> Option<&[u8]> {
    let header = b"Photoshop 3.0\0";
    if data.len() < header.len() || &data[..header.len()] != header {
        return None;
    }

    let mut i = header.len();
    // Skip 8BIM marker
    while i + 11 < data.len() {
        if &data[i..i + 4] == b"8BIM" {
            let resource_type = u16::from_be_bytes([data[i + 4], data[i + 5]]);
            // Skip pascal string (name)
            let name_len = data[i + 6] as usize;
            let padded_name_len = if (name_len + 1) % 2 != 0 {
                name_len + 2
            } else {
                name_len + 1
            };
            let data_len_pos = i + 6 + padded_name_len;
            if data_len_pos + 4 > data.len() {
                break;
            }
            let data_len = u32::from_be_bytes([
                data[data_len_pos],
                data[data_len_pos + 1],
                data[data_len_pos + 2],
                data[data_len_pos + 3],
            ]) as usize;
            let data_start = data_len_pos + 4;

            if resource_type == 0x0404 && data_start + data_len <= data.len() {
                return Some(&data[data_start..data_start + data_len]);
            }

            let padded_data_len = if data_len % 2 != 0 {
                data_len + 1
            } else {
                data_len
            };
            i = data_start + padded_data_len;
        } else {
            break;
        }
    }
    None
}

/// Parse IPTC-IIM records
fn parse_iptc_records(data: &[u8], fields: &mut HashMap<String, String>) {
    let mut keywords: Vec<String> = Vec::new();
    let mut i = 0;

    while i + 4 < data.len() {
        if data[i] != 0x1C {
            i += 1;
            continue;
        }
        let record = data[i + 1];
        let dataset = data[i + 2];
        let length = u16::from_be_bytes([data[i + 3], data[i + 4]]) as usize;
        i += 5;

        if record != 2 || i + length > data.len() {
            i += length;
            continue;
        }

        let value = String::from_utf8_lossy(&data[i..i + length]).to_string();
        i += length;

        match dataset {
            IPTC_HEADLINE => { fields.insert("Title".into(), value); }
            IPTC_CAPTION => { fields.insert("Description".into(), value); }
            IPTC_BYLINE => { fields.insert("Author".into(), value); }
            IPTC_BYLINE_TITLE => { fields.insert("AuthorTitle".into(), value); }
            IPTC_COPYRIGHT => { fields.insert("Copyright".into(), value); }
            IPTC_KEYWORDS => { keywords.push(value); }
            IPTC_CITY => { fields.insert("City".into(), value); }
            IPTC_STATE => { fields.insert("State".into(), value); }
            IPTC_COUNTRY => { fields.insert("Country".into(), value); }
            IPTC_COUNTRY_CODE => { fields.insert("CountryCode".into(), value); }
            IPTC_CREDIT => { fields.insert("Credit".into(), value); }
            IPTC_SOURCE => { fields.insert("Source".into(), value); }
            IPTC_DATE_CREATED => { fields.insert("IPTC_DateCreated".into(), value); }
            IPTC_INSTRUCTIONS => { fields.insert("Instructions".into(), value); }
            _ => {}
        }
    }

    if !keywords.is_empty() {
        fields.insert("Keywords".into(), keywords.join(", "));
    }
}

/// Build IPTC binary records from fields
fn build_iptc_records(fields: &HashMap<String, String>) -> Vec<u8> {
    let mut data = Vec::new();

    let field_map: &[(&str, u8)] = &[
        ("Title", IPTC_HEADLINE),
        ("Description", IPTC_CAPTION),
        ("Author", IPTC_BYLINE),
        ("AuthorTitle", IPTC_BYLINE_TITLE),
        ("Copyright", IPTC_COPYRIGHT),
        ("City", IPTC_CITY),
        ("State", IPTC_STATE),
        ("Country", IPTC_COUNTRY),
        ("CountryCode", IPTC_COUNTRY_CODE),
        ("Credit", IPTC_CREDIT),
        ("Source", IPTC_SOURCE),
        ("IPTC_DateCreated", IPTC_DATE_CREATED),
        ("Instructions", IPTC_INSTRUCTIONS),
    ];

    for (key, dataset) in field_map {
        if let Some(val) = fields.get(*key) {
            if !val.is_empty() {
                write_iptc_record(&mut data, *dataset, val.as_bytes());
            }
        }
    }

    // Keywords: each keyword is a separate record
    if let Some(val) = fields.get("Keywords") {
        for kw in val.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            write_iptc_record(&mut data, IPTC_KEYWORDS, kw.as_bytes());
        }
    }

    data
}

fn write_iptc_record(data: &mut Vec<u8>, dataset: u8, value: &[u8]) {
    data.push(0x1C);
    data.push(0x02); // Record 2
    data.push(dataset);
    let len = value.len() as u16;
    data.extend_from_slice(&len.to_be_bytes());
    data.extend_from_slice(value);
}

/// Build APP13 segment with Photoshop 3.0 wrapper
fn build_iptc_segment(fields: &HashMap<String, String>) -> Vec<u8> {
    let iptc_records = build_iptc_records(fields);
    if iptc_records.is_empty() {
        return Vec::new();
    }

    let header = b"Photoshop 3.0\0";
    // 8BIM resource for IPTC-NAA (0x0404)
    let mut resource = Vec::new();
    resource.extend_from_slice(b"8BIM");
    resource.extend_from_slice(&0x0404u16.to_be_bytes());
    resource.push(0); // Empty pascal string
    resource.push(0); // Padding
    resource.extend_from_slice(&(iptc_records.len() as u32).to_be_bytes());
    resource.extend_from_slice(&iptc_records);
    if resource.len() % 2 != 0 {
        resource.push(0); // Pad to even
    }

    let payload_len = header.len() + resource.len();
    let seg_len = (payload_len + 2) as u16;

    let mut segment = Vec::new();
    segment.push(0xFF);
    segment.push(0xED);
    segment.extend_from_slice(&seg_len.to_be_bytes());
    segment.extend_from_slice(header);
    segment.extend_from_slice(&resource);
    segment
}

/// Replace existing APP13 or insert new one
fn replace_or_insert_iptc(data: &[u8], iptc_segment: &[u8]) -> Result<Vec<u8>, String> {
    if iptc_segment.is_empty() {
        return Ok(data.to_vec());
    }

    let mut result = Vec::new();
    let mut i = 0;
    let mut written = false;

    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err("Not a valid JPEG".to_string());
    }
    result.extend_from_slice(&data[0..2]);
    i = 2;

    while i + 3 < data.len() {
        if data[i] == 0xFF && data[i + 1] == 0xED {
            // Replace existing APP13
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            result.extend_from_slice(iptc_segment);
            written = true;
            i += 2 + seg_len;
        } else if data[i] == 0xFF && data[i + 1] >= 0xE0 && data[i + 1] <= 0xEF {
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            let seg_end = i + 2 + seg_len;
            result.extend_from_slice(&data[i..seg_end.min(data.len())]);
            i = seg_end.min(data.len());
        } else {
            if !written {
                result.extend_from_slice(iptc_segment);
                written = true;
            }
            result.extend_from_slice(&data[i..]);
            return Ok(result);
        }
    }

    if !written {
        result.extend_from_slice(iptc_segment);
    }
    result.extend_from_slice(&data[i..]);
    Ok(result)
}
