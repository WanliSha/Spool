use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;

/// Read XMP metadata from a JPEG file.
/// Scans APP1 segments for XMP data (identified by "http://ns.adobe.com/xap/1.0/").
pub fn read_xmp(path: &Path) -> HashMap<String, String> {
    let mut fields = HashMap::new();

    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => return fields,
    };

    let xmp_xml = match extract_xmp_xml(&data) {
        Some(xml) => xml,
        None => return fields,
    };

    parse_xmp_fields(&xmp_xml, &mut fields);
    fields
}

/// Write XMP metadata to a JPEG file.
pub fn write_xmp(path: &Path, fields: &HashMap<String, String>) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Cannot read file: {e}"))?;

    let xmp_xml = build_xmp_xml(fields);
    let xmp_bytes = build_xmp_segment(&xmp_xml);

    let new_data = replace_or_insert_xmp(&data, &xmp_bytes)?;
    fs::write(path, new_data).map_err(|e| format!("Cannot write file: {e}"))?;

    Ok(())
}

/// Extract XMP XML string from JPEG data
fn extract_xmp_xml(data: &[u8]) -> Option<String> {
    let xmp_marker = b"http://ns.adobe.com/xap/1.0/\0";
    let mut i = 2; // Skip SOI

    while i + 4 < data.len() {
        if data[i] == 0xFF && data[i + 1] == 0xE1 {
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            let seg_start = i + 4;
            let seg_end = i + 2 + seg_len;
            if seg_end <= data.len() {
                let seg_data = &data[seg_start..seg_end];
                if seg_data.len() > xmp_marker.len()
                    && seg_data.windows(xmp_marker.len()).any(|w| w == xmp_marker)
                {
                    // Find the start of XML after the marker
                    if let Some(pos) = seg_data
                        .windows(xmp_marker.len())
                        .position(|w| w == xmp_marker)
                    {
                        let xml_start = pos + xmp_marker.len();
                        if xml_start < seg_data.len() {
                            let xml = String::from_utf8_lossy(&seg_data[xml_start..]).to_string();
                            return Some(xml);
                        }
                    }
                }
            }
            i = seg_end;
        } else if data[i] == 0xFF && data[i + 1] == 0xDA {
            break; // SOS - no more APP segments
        } else if data[i] == 0xFF && data[i + 1] >= 0xE0 {
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            i += 2 + seg_len;
        } else {
            i += 1;
        }
    }
    None
}

/// Parse XMP XML and extract fields into the map using unified field keys
fn parse_xmp_fields(xml: &str, fields: &mut HashMap<String, String>) {
    // Simple attribute and element parsing for common XMP fields
    // XMP stores values as either XML attributes or child elements

    // dc: namespace
    extract_simple_value(xml, "dc:title", fields, "Title");
    extract_simple_value(xml, "dc:description", fields, "Description");
    extract_simple_value(xml, "dc:creator", fields, "Author");
    extract_simple_value(xml, "dc:rights", fields, "Copyright");
    extract_bag_values(xml, "dc:subject", fields, "Keywords");

    // xmp: namespace
    extract_attr_value(xml, "xmp:Rating", fields, "Rating");
    extract_attr_value(xml, "xmp:Label", fields, "Label");
    extract_attr_value(xml, "xmp:CreateDate", fields, "XMP_CreateDate");
    extract_attr_value(xml, "xmp:ModifyDate", fields, "XMP_ModifyDate");
    extract_attr_value(xml, "xmp:CreatorTool", fields, "Software");

    // photoshop: namespace
    extract_attr_value(xml, "photoshop:DateCreated", fields, "XMP_DateCreated");
    extract_attr_value(xml, "photoshop:City", fields, "City");
    extract_attr_value(xml, "photoshop:State", fields, "State");
    extract_attr_value(xml, "photoshop:Country", fields, "Country");

    // Iptc4xmpCore:
    extract_attr_value(xml, "Iptc4xmpCore:CountryCode", fields, "CountryCode");

    // spool: custom namespace
    extract_spool_fields(xml, fields);
}

/// Extract a value from an XML attribute like: xmp:Rating="5"
fn extract_attr_value(xml: &str, attr: &str, fields: &mut HashMap<String, String>, key: &str) {
    let pattern = format!("{}=\"", attr);
    if let Some(pos) = xml.find(&pattern) {
        let start = pos + pattern.len();
        if let Some(end) = xml[start..].find('"') {
            let val = &xml[start..start + end];
            if !val.is_empty() {
                fields.insert(key.to_string(), val.to_string());
            }
        }
    }
}

/// Extract a simple value from an element like: <dc:title><rdf:Alt><rdf:li>value</rdf:li></rdf:Alt></dc:title>
fn extract_simple_value(xml: &str, tag: &str, fields: &mut HashMap<String, String>, key: &str) {
    // First try as attribute
    extract_attr_value(xml, tag, fields, key);
    if fields.contains_key(key) {
        return;
    }

    // Try as element with rdf:li child
    let open = format!("<{}", tag);
    if let Some(pos) = xml.find(&open) {
        // Look for rdf:li content
        let after = &xml[pos..];
        if let Some(li_start) = after.find("<rdf:li") {
            let li_after = &after[li_start..];
            if let Some(gt) = li_after.find('>') {
                let content_start = gt + 1;
                if let Some(end) = li_after[content_start..].find("</rdf:li") {
                    let val = &li_after[content_start..content_start + end];
                    let val = val.trim();
                    if !val.is_empty() {
                        fields.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
    }
}

/// Extract bag values (like keywords) from: <dc:subject><rdf:Bag><rdf:li>a</rdf:li><rdf:li>b</rdf:li></rdf:Bag></dc:subject>
fn extract_bag_values(xml: &str, tag: &str, fields: &mut HashMap<String, String>, key: &str) {
    let open = format!("<{}", tag);
    if let Some(pos) = xml.find(&open) {
        let close = format!("</{}>", tag);
        if let Some(end_pos) = xml[pos..].find(&close) {
            let section = &xml[pos..pos + end_pos];
            let mut values = Vec::new();
            let mut search_from = 0;
            while let Some(li_pos) = section[search_from..].find("<rdf:li") {
                let abs_pos = search_from + li_pos;
                if let Some(gt) = section[abs_pos..].find('>') {
                    let content_start = abs_pos + gt + 1;
                    if let Some(end) = section[content_start..].find("</rdf:li") {
                        let val = section[content_start..content_start + end].trim();
                        if !val.is_empty() {
                            values.push(val.to_string());
                        }
                        search_from = content_start + end;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if !values.is_empty() {
                fields.insert(key.to_string(), values.join(", "));
            }
        }
    }
}

/// Extract custom spool: namespace fields
fn extract_spool_fields(xml: &str, fields: &mut HashMap<String, String>) {
    let prefix = "spool:";
    let mut search_from = 0;
    while let Some(pos) = xml[search_from..].find(prefix) {
        let abs_pos = search_from + pos;
        let after = &xml[abs_pos + prefix.len()..];

        // Could be attribute: spool:FilmStock="Portra 400"
        if let Some(eq) = after.find('=') {
            let name = &after[..eq];
            if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let val_start = eq + 2; // skip ="
                if let Some(val_end) = after[val_start..].find('"') {
                    let val = &after[val_start..val_start + val_end];
                    if !val.is_empty() {
                        fields.insert(
                            format!("spool:{}", name),
                            val.to_string(),
                        );
                    }
                }
            }
        }
        search_from = abs_pos + prefix.len();
    }
}

/// Build XMP XML from unified fields
pub fn build_xmp_xml(fields: &HashMap<String, String>) -> String {
    let mut attrs = Vec::new();
    let mut elements = Vec::new();

    // Collect namespace prefixes needed
    let mut ns = Vec::new();
    ns.push(("xmlns:x", "adobe:ns:meta/"));
    ns.push(("xmlns:rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"));
    ns.push(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
    ns.push(("xmlns:xmp", "http://ns.adobe.com/xap/1.0/"));
    ns.push(("xmlns:photoshop", "http://ns.adobe.com/photoshop/1.0/"));
    ns.push(("xmlns:Iptc4xmpCore", "http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/"));

    let has_spool = fields.keys().any(|k| k.starts_with("spool:"));
    if has_spool {
        ns.push(("xmlns:spool", "http://spool.app/ns/1.0/"));
    }

    // Simple attribute fields
    let attr_map: &[(&str, &str)] = &[
        ("Rating", "xmp:Rating"),
        ("Label", "xmp:Label"),
        ("XMP_CreateDate", "xmp:CreateDate"),
        ("XMP_ModifyDate", "xmp:ModifyDate"),
        ("Software", "xmp:CreatorTool"),
        ("XMP_DateCreated", "photoshop:DateCreated"),
        ("City", "photoshop:City"),
        ("State", "photoshop:State"),
        ("Country", "photoshop:Country"),
        ("CountryCode", "Iptc4xmpCore:CountryCode"),
    ];

    for (key, xmp_attr) in attr_map {
        if let Some(val) = fields.get(*key) {
            if !val.is_empty() {
                attrs.push(format!("{}=\"{}\"", xmp_attr, xml_escape(val)));
            }
        }
    }

    // Spool custom fields as attributes
    for (key, val) in fields {
        if key.starts_with("spool:") && !val.is_empty() {
            attrs.push(format!("{}=\"{}\"", key, xml_escape(val)));
        }
    }

    // Structured elements (dc:title, dc:description, dc:creator, dc:rights as rdf:Alt/rdf:Seq)
    let alt_map: &[(&str, &str)] = &[
        ("Title", "dc:title"),
        ("Description", "dc:description"),
        ("Copyright", "dc:rights"),
    ];

    for (key, tag) in alt_map {
        if let Some(val) = fields.get(*key) {
            if !val.is_empty() {
                elements.push(format!(
                    "<{}><rdf:Alt><rdf:li xml:lang=\"x-default\">{}</rdf:li></rdf:Alt></{}>",
                    tag,
                    xml_escape(val),
                    tag
                ));
            }
        }
    }

    // dc:creator as rdf:Seq
    if let Some(val) = fields.get("Author") {
        if !val.is_empty() {
            elements.push(format!(
                "<dc:creator><rdf:Seq><rdf:li>{}</rdf:li></rdf:Seq></dc:creator>",
                xml_escape(val)
            ));
        }
    }

    // dc:subject (keywords) as rdf:Bag
    if let Some(val) = fields.get("Keywords") {
        if !val.is_empty() {
            let keywords: Vec<&str> = val.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if !keywords.is_empty() {
                let lis: Vec<String> = keywords
                    .iter()
                    .map(|k| format!("<rdf:li>{}</rdf:li>", xml_escape(k)))
                    .collect();
                elements.push(format!(
                    "<dc:subject><rdf:Bag>{}</rdf:Bag></dc:subject>",
                    lis.join("")
                ));
            }
        }
    }

    let ns_str: String = ns.iter().map(|(k, v)| format!(" {}=\"{}\"", k, v)).collect();
    let attrs_str: String = attrs.iter().map(|a| format!("\n            {}", a)).collect();
    let elems_str: String = elements.iter().map(|e| format!("\n          {}", e)).collect();

    format!(
        r#"<?xpacket begin="﻿" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF{}>
    <rdf:Description rdf:about=""{}>{}</rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
        ns_str, attrs_str, elems_str
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Build APP1 segment bytes for XMP
fn build_xmp_segment(xmp_xml: &str) -> Vec<u8> {
    let header = b"http://ns.adobe.com/xap/1.0/\0";
    let payload_len = header.len() + xmp_xml.len();
    let seg_len = (payload_len + 2) as u16;

    let mut segment = Vec::new();
    segment.push(0xFF);
    segment.push(0xE1);
    segment.extend_from_slice(&seg_len.to_be_bytes());
    segment.extend_from_slice(header);
    segment.extend_from_slice(xmp_xml.as_bytes());
    segment
}

/// Replace existing XMP APP1 segment or insert a new one
fn replace_or_insert_xmp(data: &[u8], xmp_segment: &[u8]) -> Result<Vec<u8>, String> {
    let xmp_marker = b"http://ns.adobe.com/xap/1.0/\0";
    let mut result = Vec::new();
    let mut i = 0;
    let mut xmp_written = false;

    // Copy SOI
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err("Not a valid JPEG".to_string());
    }
    result.extend_from_slice(&data[0..2]);
    i = 2;

    while i + 3 < data.len() {
        if data[i] == 0xFF && data[i + 1] == 0xE1 {
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            let seg_end = i + 2 + seg_len;
            let seg_data = if seg_end <= data.len() {
                &data[i + 4..seg_end]
            } else {
                &data[i + 4..]
            };

            if seg_data.len() > xmp_marker.len()
                && seg_data.windows(xmp_marker.len()).any(|w| w == xmp_marker)
            {
                // Replace XMP segment
                result.extend_from_slice(xmp_segment);
                xmp_written = true;
                i = seg_end.min(data.len());
                continue;
            }
            // Keep non-XMP APP1 (e.g. EXIF)
            result.extend_from_slice(&data[i..seg_end.min(data.len())]);
            i = seg_end.min(data.len());
        } else if data[i] == 0xFF && data[i + 1] >= 0xE0 && data[i + 1] <= 0xEF {
            // Other APP segment — keep it
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            let seg_end = i + 2 + seg_len;
            result.extend_from_slice(&data[i..seg_end.min(data.len())]);
            i = seg_end.min(data.len());
        } else {
            // Non-APP marker — if XMP not yet written, insert before this
            if !xmp_written {
                result.extend_from_slice(xmp_segment);
                xmp_written = true;
            }
            // Copy rest of file
            result.extend_from_slice(&data[i..]);
            return Ok(result);
        }
    }

    if !xmp_written {
        result.extend_from_slice(xmp_segment);
    }
    result.extend_from_slice(&data[i..]);
    Ok(result)
}
