//! enkidullm-ingest — Sovereign document ingestion pipeline.
//!
//! Three ingestion paths:
//!   PDF  → inflate.rs (FlateDecode) + pdf.rs (content stream parser)
//!   Epub → inflate.rs (ZIP DEFLATE) + epub.rs (XHTML tag stripper)
//!   Text → chunker.rs (sentence boundary) + tokenizer.rs (tribal classification)
//!
//! All pure Rust — no lopdf, no epub crate, no PyMuPDF.
//! DEFLATE (RFC 1951) decompressor is implemented natively in inflate.rs.

#![forbid(unsafe_code)]

pub mod inflate;
pub mod pdf;
pub mod epub;
pub mod chunker;
pub mod tokenizer;

pub use inflate::{inflate, inflate_zlib};
pub use pdf::{extract_pdf, PdfExtract};
pub use epub::{extract_epub, EpubExtract};
pub use chunker::{chunk_text, ChunkConfig, TextChunk};
pub use tokenizer::{tokenize, TokenUnit, TokenClass, TokenStats};

/// Auto-detect format from file extension and extract text.
pub fn ingest_file(path: &str, data: &[u8]) -> IngestResult {
    let path_lc = path.to_ascii_lowercase();
    if path_lc.ends_with(".pdf") {
        let extracted = extract_pdf(data);
        IngestResult {
            title:     extracted.title,
            author:    extracted.author,
            isbn:      None,
            publisher: None,
            full_text: extracted.full_text,
            format:    DocFormat::Pdf,
        }
    } else if path_lc.ends_with(".epub") {
        let extracted = extract_epub(data);
        IngestResult {
            title:     extracted.title,
            author:    extracted.author,
            isbn:      extracted.isbn,
            publisher: extracted.publisher,
            full_text: extracted.full_text,
            format:    DocFormat::Epub,
        }
    } else {
        // Plain text fallback
        let text = String::from_utf8_lossy(data).into_owned();
        IngestResult {
            title: None, author: None, isbn: None, publisher: None,
            full_text: text,
            format: DocFormat::PlainText,
        }
    }
}

/// Document format detected at ingest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocFormat { Pdf, Epub, PlainText }

/// Unified result from any document format.
#[derive(Debug, Clone)]
pub struct IngestResult {
    pub title:     Option<String>,
    pub author:    Option<String>,
    pub isbn:      Option<String>,
    pub publisher: Option<String>,
    pub full_text: String,
    pub format:    DocFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_plain_text() {
        let data = b"Hello World. This is a sovereign test.";
        let result = ingest_file("test.txt", data);
        assert_eq!(result.format, DocFormat::PlainText);
        assert!(result.full_text.contains("sovereign"));
    }

    #[test]
    fn ingest_pdf_detects_format() {
        let pdf = b"%PDF-1.4\n4 0 obj\n<</Length 30>>\nstream\nBT (Test Text) Tj ET\nendstream\nendobj\n";
        let result = ingest_file("book.pdf", pdf);
        assert_eq!(result.format, DocFormat::Pdf);
    }

    #[test]
    fn ingest_epub_detects_format() {
        let result = ingest_file("book.epub", b"PK"); // minimal trigger
        assert_eq!(result.format, DocFormat::Epub);
    }
}
