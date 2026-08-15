#![forbid(unsafe_code)]
//! record_router — normalise ANY ingested entry into `Vec<RawRecord>`.
//!
//! The router is the universal adapter layer between raw file bytes and the
//! BeeMDM ETL station chain.  Every format — CSV, JSON, XML, Parquet, Excel,
//! PDF, images, point clouds — exits this module as a uniform `Vec<RawRecord>`
//! so that adad-gate, data-structure-station, VGCA beam, and the scoring engine
//! work without modification.
//!
//! ## Routing table
//!
//! | MediaCategory   | Strategy                                                  |
//! |-----------------|-----------------------------------------------------------|
//! | `Csv`           | parse_csv (full row-per-record)                           |
//! | `Json`          | parse_json_records (one record per top-level object)      |
//! | `Ndjson`        | parse_ndjson_records (one record per line)                |
//! | `Xml`           | parse_xml_records (one record per repeating element)      |
//! | `StructuredData`| blob_record (metadata particle: type + size + hash)       |
//! | `Image`         | image_record (metadata particle + basic EXIF markers)     |
//! | `PointCloud`    | blob_record labelled as point-cloud                       |
//! | `Media`         | blob_record labelled as media                             |
//! | `Archive`       | blob_record (nested archives log a warning)               |
//! | `Opaque`        | blob_record with content hash only                        |

use media_type_detector::{MediaType, MediaCategory, detect as detect_media_type};
use crate::kaki_generator::{RawRecord, fnv1a_32};

// ── Public API ────────────────────────────────────────────────────────────────

/// Normalise `data` (one file extracted from an archive, or a standalone file)
/// into a list of `RawRecord`s ready for the ETL station chain.
///
/// `name` is the file name (used in blob records and error messages only).
pub fn records_from_entry(name: &str, data: &[u8]) -> Vec<RawRecord> {
    let mt = detect_media_type(data);
    match mt {
        MediaType::Csv => crate::kaki_generator::parse_csv(data),

        MediaType::Json => parse_json_records(data),

        MediaType::Ndjson => parse_ndjson_records(data),

        MediaType::Xml => parse_xml_records(data),

        // Opaque structured formats: Parquet, Excel, PDF, BSON, CBOR, Word, ODS …
        // → single metadata particle; downstream specialist processes can decode further.
        _ if mt.category() == MediaCategory::StructuredData => {
            vec![opaque_record(name, data, mt)]
        }

        _ if mt.category() == MediaCategory::Image => {
            vec![image_record(name, data, mt)]
        }

        _ if mt.category() == MediaCategory::PointCloud => {
            vec![opaque_record(name, data, mt)]
        }

        _ if mt.category() == MediaCategory::Media => {
            vec![opaque_record(name, data, mt)]
        }

        // Nested archive inside an archive — log and return a blob record.
        _ if mt.category() == MediaCategory::Archive => {
            eprintln!("[record_router] WARN: nested archive detected inside entry '{}' ({}), \
                       producing metadata particle — extract manually for full ingestion",
                name, mt.label());
            vec![opaque_record(name, data, mt)]
        }

        _ => vec![opaque_record(name, data, mt)],
    }
}

// ── JSON parser ───────────────────────────────────────────────────────────────

/// Parse a JSON byte slice into one `RawRecord` per top-level object.
///
/// Supports:
///   - JSON array of flat objects:  `[{"k":"v",...}, ...]`
///   - Single flat JSON object:     `{"k":"v",...}`
///   - Nested values are serialised as their raw JSON substring.
pub fn parse_json_records(data: &[u8]) -> Vec<RawRecord> {
    let Ok(text) = std::str::from_utf8(data) else { return Vec::new(); };
    let s = text.trim();

    if s.starts_with('[') {
        // Array — collect each {...} top-level element
        parse_json_array(s)
    } else if s.starts_with('{') {
        // Single object
        parse_json_object(s)
            .map(|r| vec![r])
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

/// Parse NDJSON: one JSON object per line.
pub fn parse_ndjson_records(data: &[u8]) -> Vec<RawRecord> {
    let Ok(text) = std::str::from_utf8(data) else { return Vec::new(); };
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let l = line.trim();
            if l.starts_with('{') {
                parse_json_object(l).map(|mut r| { r.seq = (i + 1) as u32; r })
            } else {
                None
            }
        })
        .collect()
}

// ── XML parser ────────────────────────────────────────────────────────────────

/// Parse an XML byte slice into one `RawRecord` per repeating leaf element.
///
/// Strategy: find the first level of repeating elements inside the document root
/// and treat each one as a record.  Each record's direct child elements become
/// header/value pairs.
pub fn parse_xml_records(data: &[u8]) -> Vec<RawRecord> {
    let Ok(text) = std::str::from_utf8(data) else { return Vec::new(); };

    // Collect all <tag>...</tag> segments at depth 2
    let segments = collect_child_segments(text);
    if segments.is_empty() { return Vec::new(); }

    // Derive headers from the first segment
    let first_headers = extract_xml_children(&segments[0]);
    if first_headers.is_empty() { return Vec::new(); }

    let headers: Vec<String> = first_headers.into_iter().map(|(k, _)| k).collect();

    segments.iter().enumerate().map(|(i, seg)| {
        let children = extract_xml_children(seg);
        let values: Vec<Vec<u8>> = headers.iter().map(|h| {
            children.iter()
                .find(|(k, _)| k == h)
                .map(|(_, v)| v.as_bytes().to_vec())
                .unwrap_or_default()
        }).collect();
        RawRecord { headers: headers.clone(), values, seq: (i + 1) as u32 }
    }).collect()
}

// ── Opaque / Image blob records ───────────────────────────────────────────────

/// A single `RawRecord` carrying file-level metadata for formats BeeMDM cannot
/// decode natively.  The station chain will score this particle based on the
/// presence/absence of these metadata fields.
fn opaque_record(name: &str, data: &[u8], mt: MediaType) -> RawRecord {
    let hash = content_hash(data);
    let headers = vec![
        "file_name".to_string(),
        "media_type".to_string(),
        "media_label".to_string(),
        "size_bytes".to_string(),
        "content_hash".to_string(),
    ];
    let values = vec![
        name.as_bytes().to_vec(),
        mt.mime_type().as_bytes().to_vec(),
        mt.label().as_bytes().to_vec(),
        data.len().to_string().into_bytes(),
        format!("{:08x}", hash).into_bytes(),
    ];
    RawRecord { headers, values, seq: 1 }
}

/// Metadata record for image files — same as opaque but also probes for
/// known EXIF / TIFF header landmarks to extract basic geo metadata.
fn image_record(name: &str, data: &[u8], mt: MediaType) -> RawRecord {
    let hash  = content_hash(data);
    let mut headers = vec![
        "file_name".to_string(),
        "media_type".to_string(),
        "media_label".to_string(),
        "size_bytes".to_string(),
        "content_hash".to_string(),
    ];
    let mut values = vec![
        name.as_bytes().to_vec(),
        mt.mime_type().as_bytes().to_vec(),
        mt.label().as_bytes().to_vec(),
        data.len().to_string().into_bytes(),
        format!("{:08x}", hash).into_bytes(),
    ];

    // For TIFF/GeoTIFF: probe width × height from IFD entry tags 256 (ImageWidth)
    // and 257 (ImageLength) — both at offset 8 from TIFF header.
    if mt == MediaType::Tiff && data.len() >= 10 {
        let le = data[0] == 0x49; // little-endian marker
        if let Some((w, h)) = probe_tiff_dimensions(data, le) {
            headers.push("image_width".to_string());
            values.push(w.to_string().into_bytes());
            headers.push("image_height".to_string());
            values.push(h.to_string().into_bytes());
        }
    }

    // For JPEG: probe JFIF/Exif APP0/APP1 for width × height (SOF0 marker 0xFFC0)
    if mt == MediaType::Jpeg {
        if let Some((w, h)) = probe_jpeg_dimensions(data) {
            headers.push("image_width".to_string());
            values.push(w.to_string().into_bytes());
            headers.push("image_height".to_string());
            values.push(h.to_string().into_bytes());
        }
    }

    // For PNG: width and height are at bytes 16-23 of the IHDR chunk
    if mt == MediaType::Png && data.len() >= 24 {
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        headers.push("image_width".to_string());
        values.push(w.to_string().into_bytes());
        headers.push("image_height".to_string());
        values.push(h.to_string().into_bytes());
    }

    RawRecord { headers, values, seq: 1 }
}

// ── TIFF dimension probe ──────────────────────────────────────────────────────

fn probe_tiff_dimensions(data: &[u8], le: bool) -> Option<(u32, u32)> {
    let read16 = |off: usize| -> Option<u16> {
        if data.len() < off + 2 { return None; }
        let b = [data[off], data[off + 1]];
        Some(if le { u16::from_le_bytes(b) } else { u16::from_be_bytes(b) })
    };
    let read32 = |off: usize| -> Option<u32> {
        if data.len() < off + 4 { return None; }
        let b = [data[off], data[off + 1], data[off + 2], data[off + 3]];
        Some(if le { u32::from_le_bytes(b) } else { u32::from_be_bytes(b) })
    };

    let ifd0_offset = read32(4)? as usize;
    let entry_count = read16(ifd0_offset)? as usize;
    let mut width = None::<u32>;
    let mut height = None::<u32>;

    for i in 0..entry_count {
        let entry_off = ifd0_offset + 2 + i * 12;
        let tag  = read16(entry_off)?;
        let typ  = read16(entry_off + 2)?;
        let val_off = entry_off + 8;
        let val: u32 = match typ {
            1 => data.get(val_off).copied()? as u32,   // BYTE
            3 => read16(val_off)? as u32,               // SHORT
            4 => read32(val_off)?,                      // LONG
            _ => continue,
        };
        match tag {
            256 => width  = Some(val),
            257 => height = Some(val),
            _   => {}
        }
        if width.is_some() && height.is_some() { break; }
    }
    Some((width?, height?))
}

// ── JPEG dimension probe ──────────────────────────────────────────────────────

fn probe_jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2usize; // skip FFD8
    while i + 3 < data.len() {
        if data[i] != 0xFF { break; }
        let marker = data[i + 1];
        let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        // SOF0 (0xC0) or SOF2 (0xC2) — both have: 1-byte precision, 2-byte height, 2-byte width
        if (marker == 0xC0 || marker == 0xC2) && i + 8 < data.len() {
            let h = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
            let w = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
            return Some((w, h));
        }
        if length < 2 { break; }
        i += 2 + length;
    }
    None
}

// ── JSON internals ────────────────────────────────────────────────────────────

fn parse_json_array(s: &str) -> Vec<RawRecord> {
    let mut records = Vec::new();
    let mut depth   = 0i32;
    let mut start   = None::<usize>;
    let mut seq     = 1u32;
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        match c {
            '{' => {
                if depth == 1 { start = Some(i); }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 1 {
                    if let Some(st) = start {
                        let obj_src: String = chars[st..=i].iter().collect();
                        if let Some(mut r) = parse_json_object(&obj_src) {
                            r.seq = seq;
                            records.push(r);
                            seq += 1;
                        }
                    }
                    start = None;
                }
            }
            '[' => { if depth == 0 { depth = 1; } }
            _   => {}
        }
    }
    records
}

/// Parse a single JSON object string `{...}` into a RawRecord.
/// Nested objects/arrays are kept as their raw JSON substring value.
fn parse_json_object(s: &str) -> Option<RawRecord> {
    let pairs = extract_json_pairs(s);
    if pairs.is_empty() { return None; }
    let headers: Vec<String>  = pairs.iter().map(|(k, _)| k.clone()).collect();
    let values:  Vec<Vec<u8>> = pairs.iter().map(|(_, v)| v.as_bytes().to_vec()).collect();
    Some(RawRecord { headers, values, seq: 1 })
}

/// Extract key-value pairs from a flat JSON object string.
/// Returns `(key, value_as_string)` pairs; nested objects/arrays are raw JSON.
fn extract_json_pairs(s: &str) -> Vec<(String, String)> {
    let mut pairs  = Vec::new();
    let mut pos    = 0usize;
    let bytes      = s.as_bytes();
    let len        = bytes.len();

    // Skip leading '{'
    while pos < len && bytes[pos] != b'{' { pos += 1; }
    pos += 1;

    loop {
        // Skip whitespace + commas
        while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\n'
            || bytes[pos] == b'\r' || bytes[pos] == b'\t' || bytes[pos] == b',') {
            pos += 1;
        }
        if pos >= len || bytes[pos] == b'}' { break; }

        // Read key (quoted string)
        let Some(key) = read_json_string(bytes, &mut pos) else { break; };

        // Skip whitespace + colon
        while pos < len && bytes[pos] != b':' { pos += 1; }
        pos += 1;
        while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') { pos += 1; }

        // Read value
        let Some(val) = read_json_value(bytes, &mut pos) else { break; };
        pairs.push((key, val));
    }
    pairs
}

fn read_json_string(bytes: &[u8], pos: &mut usize) -> Option<String> {
    while *pos < bytes.len() && bytes[*pos] != b'"' { *pos += 1; }
    if *pos >= bytes.len() { return None; }
    *pos += 1; // skip opening "
    let start = *pos;
    while *pos < bytes.len() {
        if bytes[*pos] == b'\\' { *pos += 2; continue; }
        if bytes[*pos] == b'"' { break; }
        *pos += 1;
    }
    let s = std::str::from_utf8(&bytes[start..*pos]).ok()?.to_string();
    *pos += 1; // skip closing "
    Some(s)
}

fn read_json_value(bytes: &[u8], pos: &mut usize) -> Option<String> {
    if *pos >= bytes.len() { return None; }
    match bytes[*pos] {
        b'"' => read_json_string(bytes, pos),
        b'{' | b'[' => {
            let start = *pos;
            let open  = bytes[*pos];
            let close = if open == b'{' { b'}' } else { b']' };
            let mut depth = 0i32;
            while *pos < bytes.len() {
                if bytes[*pos] == open  { depth += 1; }
                if bytes[*pos] == close {
                    depth -= 1;
                    if depth == 0 { *pos += 1; break; }
                }
                *pos += 1;
            }
            Some(std::str::from_utf8(&bytes[start..*pos]).ok()?.to_string())
        }
        _ => {
            // Number, boolean, null
            let start = *pos;
            while *pos < bytes.len() && !matches!(bytes[*pos], b',' | b'}' | b']' | b' ' | b'\n' | b'\r') {
                *pos += 1;
            }
            Some(std::str::from_utf8(&bytes[start..*pos]).ok()?.trim().to_string())
        }
    }
}

// ── XML internals ─────────────────────────────────────────────────────────────

/// Find all `<tag>...</tag>` segments at depth 1 (direct children of root).
fn collect_child_segments(text: &str) -> Vec<String> {
    let mut segs   = Vec::new();
    let bytes      = text.as_bytes();
    let mut i      = 0usize;
    let mut depth  = 0i32;
    let mut seg_start = 0usize;

    // Skip XML declaration and processing instructions
    while i < bytes.len() && (bytes[i] != b'<' || (i + 1 < bytes.len() && bytes[i + 1] == b'?')) {
        if bytes[i] == b'<' && bytes[i + 1] == b'?' {
            while i < bytes.len() && bytes[i] != b'>' { i += 1; }
        }
        i += 1;
    }

    while i < bytes.len() {
        if bytes[i] == b'<' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                // Closing tag
                depth -= 1;
                if depth == 1 {
                    // Closing a depth-2 element → end of segment
                    let close_end = text[i..].find('>').map(|p| i + p + 1).unwrap_or(i + 1);
                    segs.push(text[seg_start..close_end].to_string());
                }
                while i < bytes.len() && bytes[i] != b'>' { i += 1; }
            } else if i + 1 < bytes.len() && bytes[i + 1] == b'!' {
                // Comment or DOCTYPE — skip
                while i < bytes.len() && bytes[i] != b'>' { i += 1; }
            } else {
                // Opening tag
                depth += 1;
                if depth == 2 { seg_start = i; }
                // Check for self-closing
                let tag_end = text[i..].find('>').unwrap_or(0);
                if tag_end > 0 && bytes[i + tag_end - 1] == b'/' {
                    depth -= 1;
                    i += tag_end;
                } else {
                    while i < bytes.len() && bytes[i] != b'>' { i += 1; }
                }
            }
        }
        i += 1;
    }
    segs
}

/// Extract `(tag_name, text_content)` pairs from a single `<record>...</record>`.
fn extract_xml_children(segment: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let bytes     = segment.as_bytes();
    let mut i     = 0usize;
    // Skip outer opening tag
    while i < bytes.len() && bytes[i] != b'>' { i += 1; }
    i += 1;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'/' { break; } // end of parent
            if i + 1 < bytes.len() && bytes[i + 1] == b'!' { // skip comment/CDATA
                while i < bytes.len() && bytes[i] != b'>' { i += 1; }
                i += 1;
                continue;
            }
            // Opening child tag — read tag name
            i += 1;
            let name_start = i;
            while i < bytes.len() && !matches!(bytes[i], b'>' | b' ' | b'\t' | b'\n' | b'\r' | b'/') {
                i += 1;
            }
            let tag = std::str::from_utf8(&bytes[name_start..i]).unwrap_or("").to_string();
            // Skip to end of opening tag
            while i < bytes.len() && bytes[i] != b'>' { i += 1; }
            if i < bytes.len() && i > 0 && bytes[i - 1] == b'/' {
                // Self-closing
                pairs.push((tag, String::new()));
                i += 1;
                continue;
            }
            i += 1; // skip >
            // Read text content until closing tag
            let text_start = i;
            // Advance to matching closing tag (simple: first '<')
            while i < bytes.len() && bytes[i] != b'<' { i += 1; }
            let text = std::str::from_utf8(&bytes[text_start..i]).unwrap_or("").trim().to_string();
            pairs.push((tag, text));
            // Skip closing </tag>
            while i < bytes.len() && bytes[i] != b'>' { i += 1; }
            i += 1;
        } else {
            i += 1;
        }
    }
    pairs
}

// ── Content hash ─────────────────────────────────────────────────────────────

/// FNV-1a 32-bit hash over all bytes — fast, deterministic content fingerprint.
fn content_hash(data: &[u8]) -> u32 {
    fnv1a_32(data)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_entry_routes_to_raw_records() {
        let data = b"name,city\nAli,Baghdad\nFatima,Basra\n";
        let records = records_from_entry("data.csv", data);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].values[0], b"Ali");
    }

    #[test]
    fn json_array_routes_correctly() {
        let data = b"[{\"name\":\"Ali\",\"city\":\"Baghdad\"},{\"name\":\"Fatima\",\"city\":\"Basra\"}]";
        let records = records_from_entry("data.json", data);
        assert_eq!(records.len(), 2);
        assert!(records[0].headers.contains(&"name".to_string()));
    }

    #[test]
    fn ndjson_routes_one_record_per_line() {
        let data = b"{\"id\":\"1\",\"name\":\"Ali\"}\n{\"id\":\"2\",\"name\":\"Fatima\"}\n";
        let records = records_from_entry("data.ndjson", data);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn xml_routes_one_record_per_element() {
        let data = b"<records>\
                       <record><name>Ali</name><city>Baghdad</city></record>\
                       <record><name>Fatima</name><city>Basra</city></record>\
                     </records>";
        let records = records_from_entry("data.xml", data);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].values[0], b"Ali");
    }

    #[test]
    fn png_produces_metadata_record() {
        let mut png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        // IHDR chunk: 4 bytes len + 4 bytes type + 4 width + 4 height + ...
        png_header.extend_from_slice(&[0, 0, 0, 13]);  // length
        png_header.extend_from_slice(b"IHDR");
        png_header.extend_from_slice(&[0, 0, 3, 200]); // width = 968
        png_header.extend_from_slice(&[0, 0, 2, 88]);  // height = 600
        png_header.extend_from_slice(&[8, 2, 0, 0]);  // bit depth + color type + padding
        let records = records_from_entry("photo.png", &png_header);
        assert_eq!(records.len(), 1);
        let r = &records[0];
        assert!(r.headers.contains(&"media_type".to_string()));
        assert!(r.headers.contains(&"image_width".to_string()));
    }

    #[test]
    fn pdf_produces_single_blob_record() {
        let data = b"%PDF-1.7\nbinary content...";
        let records = records_from_entry("report.pdf", data);
        assert_eq!(records.len(), 1);
        assert!(records[0].headers.contains(&"content_hash".to_string()));
    }

    #[test]
    fn nested_zip_produces_warning_record() {
        let data = &[0x50u8, 0x4B, 0x03, 0x04, 0, 0, 0, 0, 0, 0];
        let records = records_from_entry("inner.zip", data);
        assert_eq!(records.len(), 1);
        assert!(records[0].headers.contains(&"media_type".to_string()));
    }

    #[test]
    fn json_object_parsed_correctly() {
        let data = b"{\"national_id\":\"12345\",\"name\":\"Ali\",\"city\":\"Najaf\"}";
        let records = parse_json_records(data);
        assert_eq!(records.len(), 1);
        let r = &records[0];
        let idx = r.headers.iter().position(|h| h == "national_id").unwrap();
        assert_eq!(r.values[idx], b"12345");
    }
}
