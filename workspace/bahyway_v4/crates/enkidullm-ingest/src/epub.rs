//! Native Epub reader — pure Rust, zero external deps.
//!
//! Epub is a renamed ZIP archive containing XHTML content files and an OPF
//! metadata descriptor.  This reader:
//!   1. Scans the ZIP local-file-header sequence
//!   2. Extracts STORED (method=0) and DEFLATE (method=8) entries
//!   3. Parses content.opf for title/author/ISBN metadata
//!   4. Strips HTML tags from XHTML content files to yield plain text

use crate::inflate::inflate;

/// Extracted text and metadata from an Epub document.
#[derive(Debug, Default, Clone)]
pub struct EpubExtract {
    pub title:     Option<String>,
    pub author:    Option<String>,
    pub isbn:      Option<String>,
    pub publisher: Option<String>,
    pub chapters:  Vec<String>,   // One entry per content XHTML file
    pub full_text: String,
}

/// Extract text and metadata from an Epub (ZIP) byte slice.
pub fn extract_epub(data: &[u8]) -> EpubExtract {
    let mut result = EpubExtract::default();
    let entries = read_zip_entries(data);

    for entry in &entries {
        let name_lc = entry.name.to_ascii_lowercase();

        // Metadata: OPF package document
        if name_lc.ends_with(".opf") || name_lc.contains("content.opf") {
            if let Ok(text) = core::str::from_utf8(&entry.data) {
                parse_opf(text, &mut result);
            }
            continue;
        }

        // Content: XHTML files
        if name_lc.ends_with(".xhtml") || name_lc.ends_with(".html") || name_lc.ends_with(".htm") {
            if let Ok(text) = core::str::from_utf8(&entry.data) {
                let plain = strip_html(text);
                if !plain.trim().is_empty() {
                    result.full_text.push_str(&plain);
                    result.full_text.push('\n');
                    result.chapters.push(plain);
                }
            }
        }
    }

    result
}

// ── ZIP entry reader ──────────────────────────────────────────────────────────

struct ZipEntry {
    name: String,
    data: Vec<u8>,
}

/// Scan a ZIP byte slice for all local file headers and extract their payloads.
fn read_zip_entries(data: &[u8]) -> Vec<ZipEntry> {
    const LFH_SIG: [u8; 4] = [0x50, 0x4B, 0x03, 0x04]; // PK\x03\x04
    let mut entries = Vec::new();
    let mut pos = 0;

    while pos + 30 <= data.len() {
        if &data[pos..pos + 4] != &LFH_SIG {
            pos += 1;
            continue;
        }

        // Local file header layout:
        //   [4]  signature
        //   [2]  version needed
        //   [2]  general purpose bit flag
        //   [2]  compression method (0=stored, 8=deflated)
        //   [2]  last mod time
        //   [2]  last mod date
        //   [4]  crc-32
        //   [4]  compressed size
        //   [4]  uncompressed size
        //   [2]  file name length
        //   [2]  extra field length
        //   [n]  file name
        //   [m]  extra field
        //   [k]  file data

        let method       = u16::from_le_bytes([data[pos+6],  data[pos+7]]);
        let comp_size    = u32::from_le_bytes([data[pos+18], data[pos+19], data[pos+20], data[pos+21]]);
        let fname_len    = u16::from_le_bytes([data[pos+26], data[pos+27]]) as usize;
        let extra_len    = u16::from_le_bytes([data[pos+28], data[pos+29]]) as usize;

        let name_start   = pos + 30;
        let data_start   = name_start + fname_len + extra_len;
        let data_end     = data_start + comp_size as usize;

        if data_end > data.len() { pos += 4; continue; }

        let name_bytes   = &data[name_start..name_start + fname_len];
        let name         = String::from_utf8_lossy(name_bytes).into_owned();
        let file_data    = &data[data_start..data_end];

        let decoded = match method {
            0 => file_data.to_vec(),                // STORED
            8 => inflate(file_data).unwrap_or_default(), // DEFLATED
            _ => { pos = data_end; continue; }      // Unsupported method: skip
        };

        entries.push(ZipEntry { name, data: decoded });
        pos = data_end;
    }

    entries
}

// ── OPF metadata parser ───────────────────────────────────────────────────────

fn parse_opf(xml: &str, out: &mut EpubExtract) {
    // Dublin Core elements in OPF
    if out.title.is_none() {
        out.title = extract_xml_text(xml, "dc:title");
    }
    if out.author.is_none() {
        out.author = extract_xml_text(xml, "dc:creator")
            .or_else(|| extract_xml_text(xml, "dc:author"));
    }
    if out.publisher.is_none() {
        out.publisher = extract_xml_text(xml, "dc:publisher");
    }
    if out.isbn.is_none() {
        // Try <dc:identifier opf:scheme="ISBN">...</dc:identifier>
        out.isbn = find_isbn_identifier(xml);
    }
}

fn extract_xml_text(xml: &str, tag: &str) -> Option<String> {
    let open  = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let gt    = xml[start..].find('>')? + start;
    let end   = xml[gt..].find(&close)? + gt;
    let text  = xml[gt + 1..end].trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

fn find_isbn_identifier(xml: &str) -> Option<String> {
    let lc = xml.to_ascii_lowercase();
    let mut search = 0;
    // Search for the opening tag (with '<') so extract_xml_text can find it.
    while let Some(pos) = lc[search..].find("<dc:identifier") {
        let abs_pos = search + pos;
        let chunk = &xml[abs_pos..abs_pos.saturating_add(256).min(xml.len())];
        if chunk.to_ascii_lowercase().contains("isbn") {
            if let Some(text) = extract_xml_text(chunk, "dc:identifier") {
                let cleaned: String = text.chars()
                    .filter(|c| c.is_ascii_digit() || *c == 'X')
                    .collect();
                if cleaned.len() >= 10 { return Some(cleaned); }
            }
        }
        search = abs_pos + 14;
    }
    None
}

// ── HTML tag stripper ─────────────────────────────────────────────────────────

/// Strip HTML/XHTML tags, decode basic entities, return plain text.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut chars = html.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        match c {
            '<' => { in_tag = true; }
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            '&' if !in_tag => {
                // Decode basic HTML entities
                let rest: String = chars.by_ref().take(8).map(|(_, c)| c).collect();
                let (decoded, remainder) = decode_entity(&rest);
                out.push_str(decoded);
                // The characters after ';' need re-inserting — best effort
                if let Some(semi) = remainder.find(';') {
                    let tail = &remainder[semi + 1..];
                    for tc in tail.chars() {
                        match tc {
                            '<' => { in_tag = true; }
                            '>' => { in_tag = false; out.push(' '); }
                            c if !in_tag => out.push(c),
                            _ => {}
                        }
                    }
                }
            }
            c if !in_tag => {
                if c == '\n' || c == '\r' || c == '\t' { out.push(' '); }
                else { out.push(c); }
            }
            _ => {}
        }
    }

    // Collapse multiple spaces
    let mut result = String::with_capacity(out.len());
    let mut prev_space = false;
    for c in out.chars() {
        if c == ' ' {
            if !prev_space { result.push(' '); }
            prev_space = true;
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result.trim().to_string()
}

fn decode_entity(s: &str) -> (&'static str, &str) {
    if s.starts_with("amp;")  { return ("&",  &s[4..]); }
    if s.starts_with("lt;")   { return ("<",  &s[3..]); }
    if s.starts_with("gt;")   { return (">",  &s[3..]); }
    if s.starts_with("quot;") { return ("\"", &s[5..]); }
    if s.starts_with("nbsp;") { return (" ",  &s[5..]); }
    if s.starts_with("apos;") { return ("'",  &s[5..]); }
    ("&", s)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_basic() {
        let html = "<p>Hello <b>World</b></p>";
        let text = strip_html(html);
        assert!(text.contains("Hello"), "got: '{}'", text);
        assert!(text.contains("World"), "got: '{}'", text);
        assert!(!text.contains('<'));
    }

    #[test]
    fn strip_html_entities() {
        let html = "Tom &amp; Jerry &lt;cartoon&gt;";
        let text = strip_html(html);
        assert!(text.contains('&'), "got: '{}'", text);
    }

    #[test]
    fn extract_xml_text_basic() {
        let xml = "<dc:title>Designing Data-Intensive Applications</dc:title>";
        let title = extract_xml_text(xml, "dc:title").unwrap();
        assert_eq!(title, "Designing Data-Intensive Applications");
    }

    #[test]
    fn extract_xml_text_with_attrs() {
        let xml = r#"<dc:creator opf:role="aut">Martin Kleppmann</dc:creator>"#;
        let author = extract_xml_text(xml, "dc:creator").unwrap();
        assert_eq!(author, "Martin Kleppmann");
    }

    #[test]
    fn empty_epub_does_not_panic() {
        let result = extract_epub(b"");
        assert!(result.full_text.is_empty());
        assert!(result.chapters.is_empty());
    }

    #[test]
    fn zip_stored_entry_roundtrip() {
        // Build a minimal ZIP with one STORED entry: "hello.txt" = "Hello World"
        let content = b"Hello World";
        let fname = b"hello.txt";
        let mut zip = Vec::new();

        // Local file header
        zip.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]); // signature
        zip.extend_from_slice(&[0x14, 0x00]); // version needed
        zip.extend_from_slice(&[0x00, 0x00]); // flags
        zip.extend_from_slice(&[0x00, 0x00]); // method=STORED
        zip.extend_from_slice(&[0x00, 0x00]); // mod time
        zip.extend_from_slice(&[0x00, 0x00]); // mod date
        zip.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // crc32
        let len = content.len() as u32;
        zip.extend_from_slice(&len.to_le_bytes());   // compressed size
        zip.extend_from_slice(&len.to_le_bytes());   // uncompressed size
        zip.extend_from_slice(&(fname.len() as u16).to_le_bytes()); // fname len
        zip.extend_from_slice(&[0x00, 0x00]); // extra len
        zip.extend_from_slice(fname);
        zip.extend_from_slice(content);

        let entries = read_zip_entries(&zip);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "hello.txt");
        assert_eq!(entries[0].data, b"Hello World");
    }

    #[test]
    fn isbn_extraction_from_opf() {
        let opf = r#"<dc:identifier opf:scheme="ISBN">978-1491903698</dc:identifier>"#;
        let isbn = find_isbn_identifier(opf);
        assert_eq!(isbn, Some("9781491903698".into()));
    }
}
