//! Native PDF text extractor — pure Rust, zero external deps.
//!
//! Supports:
//!   - Uncompressed content streams
//!   - FlateDecode (zlib-wrapped DEFLATE) content streams
//!   - Text operators: BT/ET, Tj, TJ, Td, T*, quote (')
//!   - Metadata extraction from Info dictionary
//!
//! Not supported (out of scope for v4.0.2):
//!   - Encrypted PDFs
//!   - Type3 / CID fonts with non-Unicode encoding maps
//!   - LZWDecode, CCITTFaxDecode, JBIG2Decode streams

use crate::inflate::inflate_zlib;

/// Extracted text and metadata from a PDF document.
#[derive(Debug, Default, Clone)]
pub struct PdfExtract {
    pub title:     Option<String>,
    pub author:    Option<String>,
    pub subject:   Option<String>,
    pub pages:     Vec<String>,   // One entry per page (best-effort)
    pub full_text: String,        // Concatenated text from all content streams
}

/// Extract text and metadata from a PDF byte slice.
pub fn extract_pdf(data: &[u8]) -> PdfExtract {
    let mut result = PdfExtract::default();

    // Extract metadata from Info dictionary
    if let Some(info_start) = find_seq(data, b"/Info") {
        extract_info_dict(data, info_start, &mut result);
    }

    // Find all content streams and extract text
    let mut pos = 0;
    while let Some(stream_start) = find_seq_from(data, b"stream", pos) {
        let dict_start = last_dict_start(data, stream_start);
        let content_start = stream_start + 6;
        // Skip optional \r\n or \n after "stream"
        let content_start = skip_eol(data, content_start);

        // Find matching "endstream"
        let Some(end_pos) = find_seq_from(data, b"endstream", content_start) else {
            pos = stream_start + 6;
            continue;
        };

        let stream_bytes = &data[content_start..end_pos];
        let is_flat = dict_has_flat(data, dict_start, stream_start);

        let decoded: Vec<u8> = if is_flat {
            inflate_zlib(stream_bytes).unwrap_or_default()
        } else {
            stream_bytes.to_vec()
        };

        let text = extract_text_operators(&decoded);
        if !text.is_empty() {
            if !result.full_text.is_empty() { result.full_text.push('\n'); }
            result.full_text.push_str(&text);
            result.pages.push(text);
        }

        pos = end_pos + 9;
    }

    result
}

// ── Text operator parser ──────────────────────────────────────────────────────

/// Extract text from a PDF content stream byte slice.
/// Parses BT/ET blocks and handles Tj, TJ, ' (quote) operators.
fn extract_text_operators(stream: &[u8]) -> String {
    let mut out = String::new();
    let mut in_text = false;
    let mut i = 0;

    while i < stream.len() {
        // Skip whitespace
        if stream[i].is_ascii_whitespace() { i += 1; continue; }

        // String literal: (...)
        if stream[i] == b'(' {
            let (s, end) = parse_pdf_string(stream, i);
            if in_text && !s.is_empty() {
                out.push_str(&s);
                out.push(' ');
            }
            i = end;
            continue;
        }

        // Array literal: [...] — for TJ operator
        if stream[i] == b'[' {
            let (strings, end) = parse_pdf_array_strings(stream, i);
            if in_text {
                for s in strings { out.push_str(&s); }
                out.push(' ');
            }
            i = end;
            continue;
        }

        // Hex string: <hex>
        if stream[i] == b'<' && stream.get(i + 1).copied() != Some(b'<') {
            let (s, end) = parse_hex_string(stream, i);
            if in_text && !s.is_empty() {
                out.push_str(&s);
                out.push(' ');
            }
            i = end;
            continue;
        }

        // Keyword / operator
        let (kw, end) = read_keyword(stream, i);
        match kw {
            b"BT" => { in_text = true; out.push('\n'); }
            b"ET" => { in_text = false; }
            b"T*" => { if in_text { out.push('\n'); } }
            b"Tj" | b"TJ" | b"'" => {}  // operands already captured above
            _ => {}
        }
        // Always advance: read_keyword returns start position when the character
        // at i is a PDF delimiter ('/', numbers, etc.) that isn't a text operator.
        i = if end > i { end } else { i + 1 };
    }

    out.trim().to_string()
}

fn read_keyword<'a>(data: &'a [u8], start: usize) -> (&'a [u8], usize) {
    let mut i = start;
    while i < data.len() && !data[i].is_ascii_whitespace()
                          && data[i] != b'(' && data[i] != b'['
                          && data[i] != b'<' && data[i] != b'/' {
        i += 1;
    }
    (&data[start..i], i)
}

/// Parse a PDF literal string (...)  returning (content, next_pos).
/// Handles balanced parentheses and backslash escapes.
fn parse_pdf_string(data: &[u8], start: usize) -> (String, usize) {
    debug_assert_eq!(data[start], b'(');
    let mut s = String::new();
    let mut depth = 0i32;
    let mut i = start;
    while i < data.len() {
        match data[i] {
            b'(' => { depth += 1; i += 1; }
            b')' => {
                depth -= 1;
                i += 1;
                if depth == 0 { break; }
            }
            b'\\' if i + 1 < data.len() => {
                i += 1;
                match data[i] {
                    b'n'  => { s.push('\n'); i += 1; }
                    b'r'  => { s.push('\r'); i += 1; }
                    b't'  => { s.push('\t'); i += 1; }
                    b'('  => { s.push('(');  i += 1; }
                    b')'  => { s.push(')');  i += 1; }
                    b'\\' => { s.push('\\'); i += 1; }
                    _ => { i += 1; }
                }
            }
            b => {
                if depth > 0 && b.is_ascii() && b >= 0x20 {
                    s.push(b as char);
                }
                i += 1;
            }
        }
    }
    (s, i)
}

/// Parse a hex string <HEXDIGITS> returning (content, next_pos).
fn parse_hex_string(data: &[u8], start: usize) -> (String, usize) {
    let mut s = String::new();
    let mut i = start + 1; // skip '<'
    while i < data.len() && data[i] != b'>' {
        if data[i].is_ascii_hexdigit() && i + 1 < data.len() && data[i + 1].is_ascii_hexdigit() {
            let hi = hex_val(data[i]);
            let lo = hex_val(data[i + 1]);
            let byte = (hi << 4) | lo;
            if byte >= 0x20 && byte < 0x7F { s.push(byte as char); }
            i += 2;
        } else {
            i += 1;
        }
    }
    (s, i + 1) // skip '>'
}

fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

/// Parse a PDF array [...] collecting all string literals within it.
fn parse_pdf_array_strings(data: &[u8], start: usize) -> (Vec<String>, usize) {
    debug_assert_eq!(data[start], b'[');
    let mut strings = Vec::new();
    let mut i = start + 1;
    while i < data.len() && data[i] != b']' {
        if data[i] == b'(' {
            let (s, end) = parse_pdf_string(data, i);
            strings.push(s);
            i = end;
        } else if data[i] == b'<' && data.get(i + 1).copied() != Some(b'<') {
            let (s, end) = parse_hex_string(data, i);
            strings.push(s);
            i = end;
        } else {
            i += 1;
        }
    }
    (strings, i + 1) // skip ']'
}

// ── Dictionary helpers ────────────────────────────────────────────────────────

fn dict_has_flat(data: &[u8], dict_start: usize, stream_pos: usize) -> bool {
    let region = &data[dict_start..stream_pos.min(data.len())];
    find_seq(region, b"/FlateDecode").is_some()
        || find_seq(region, b"/Fl ").is_some()
}

fn last_dict_start(data: &[u8], before: usize) -> usize {
    let chunk = &data[..before.min(data.len())];
    // Walk backward to find the last "<<"
    let mut i = chunk.len().saturating_sub(1);
    let mut depth = 0i32;
    while i > 0 {
        if i + 1 < chunk.len() && chunk[i] == b'>' && chunk[i + 1] == b'>' {
            depth += 1; i = i.saturating_sub(2);
        } else if i + 1 < chunk.len() && chunk[i] == b'<' && chunk[i + 1] == b'<' {
            if depth == 0 { return i; }
            depth -= 1; i = i.saturating_sub(2);
        } else {
            i = i.saturating_sub(1);
        }
    }
    0
}

fn extract_info_dict(data: &[u8], info_pos: usize, out: &mut PdfExtract) {
    let region = &data[info_pos..data.len().min(info_pos + 2048)];
    if let Some(title) = extract_dict_string(region, b"/Title") {
        out.title = Some(title);
    }
    if let Some(author) = extract_dict_string(region, b"/Author") {
        out.author = Some(author);
    }
    if let Some(subj) = extract_dict_string(region, b"/Subject") {
        out.subject = Some(subj);
    }
}

fn extract_dict_string(data: &[u8], key: &[u8]) -> Option<String> {
    let pos = find_seq(data, key)?;
    let after = pos + key.len();
    let after = skip_whitespace(data, after);
    if data.get(after).copied() == Some(b'(') {
        let (s, _) = parse_pdf_string(data, after);
        if !s.is_empty() { return Some(s); }
    }
    None
}

// ── Byte-slice utilities ──────────────────────────────────────────────────────

fn find_seq(data: &[u8], needle: &[u8]) -> Option<usize> {
    find_seq_from(data, needle, 0)
}

fn find_seq_from(data: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || data.len() < needle.len() { return None; }
    for i in from..=(data.len() - needle.len()) {
        if &data[i..i + needle.len()] == needle { return Some(i); }
    }
    None
}

fn skip_eol(data: &[u8], mut pos: usize) -> usize {
    if data.get(pos).copied() == Some(b'\r') { pos += 1; }
    if data.get(pos).copied() == Some(b'\n') { pos += 1; }
    pos
}

fn skip_whitespace(data: &[u8], mut pos: usize) -> usize {
    while pos < data.len() && data[pos].is_ascii_whitespace() { pos += 1; }
    pos
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_text_stream() {
        let stream = b"BT (Hello World) Tj ET";
        let text = extract_text_operators(stream);
        assert!(text.contains("Hello"), "got: {}", text);
        assert!(text.contains("World"), "got: {}", text);
    }

    #[test]
    fn extract_tj_array() {
        let stream = b"BT [(Hello) 10 ( World)] TJ ET";
        let text = extract_text_operators(stream);
        assert!(text.contains("Hello"), "got: {}", text);
        assert!(text.contains("World"), "got: {}", text);
    }

    #[test]
    fn parse_pdf_literal_string() {
        let s = b"(Designing Data-Intensive Applications)";
        let (text, pos) = parse_pdf_string(s, 0);
        assert_eq!(text, "Designing Data-Intensive Applications");
        assert_eq!(pos, s.len());
    }

    #[test]
    fn parse_nested_parentheses() {
        let s = b"(foo (bar) baz)";
        let (text, _) = parse_pdf_string(s, 0);
        assert!(text.contains("foo"));
        assert!(text.contains("bar"));
    }

    #[test]
    fn parse_hex_string_basic() {
        // "AB" in hex = 0x41, 0x42 = "AB"
        let s = b"<4142>";
        let (text, _) = parse_hex_string(s, 0);
        assert_eq!(text, "AB");
    }

    #[test]
    fn minimal_pdf_extraction() {
        // Construct a minimal valid PDF-like byte stream
        let pdf = b"%PDF-1.4\n\
                    1 0 obj\n<</Type /Catalog /Pages 2 0 R>>\nendobj\n\
                    4 0 obj\n<</Length 45>>\nstream\n\
                    BT /F1 12 Tf 72 720 Td (Hello PDF) Tj ET\n\
                    endstream\nendobj\n";
        let extracted = extract_pdf(pdf);
        assert!(extracted.full_text.contains("Hello PDF"),
                "got: '{}'", extracted.full_text);
    }

    #[test]
    fn find_seq_basic() {
        assert_eq!(find_seq(b"hello world", b"world"), Some(6));
        assert_eq!(find_seq(b"hello world", b"xyz"), None);
    }

    #[test]
    fn empty_pdf_does_not_panic() {
        let result = extract_pdf(b"");
        assert!(result.full_text.is_empty());
    }
}
