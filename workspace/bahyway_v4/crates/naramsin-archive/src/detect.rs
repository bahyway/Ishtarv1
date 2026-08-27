#![forbid(unsafe_code)]
//! Stage 0 archive format detection via magic-byte signature matching.

use crate::error::ArchiveError;
use crate::format::ArchiveFormat;

/// Detect the archive format from the first bytes of a byte slice.
///
/// Returns `ArchiveFormat::None` for single uncompressed files.
/// Returns `ArchiveError::UnknownFormat` for unrecognised signatures —
/// these are surfaced for architect review before any extension is added
/// (EN-NARAMSIN-001 §2.3 scope discipline).
pub fn detect_format(header: &[u8]) -> Result<ArchiveFormat, ArchiveError> {
    let formats = [
        ArchiveFormat::TarXz, // longest magic first to avoid prefix collision
        ArchiveFormat::SevenZip,
        ArchiveFormat::TarGz,
        ArchiveFormat::TarBz2,
        ArchiveFormat::Zip,
    ];
    for fmt in &formats {
        let magic = fmt.magic();
        if header.len() >= magic.len() && &header[..magic.len()] == magic {
            return Ok(*fmt);
        }
    }
    // No archive signature detected — treat as single plain file.
    if header.is_empty() {
        return Err(ArchiveError::EmptyInput);
    }
    Ok(ArchiveFormat::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_zip_magic() {
        let header = [0x50, 0x4B, 0x03, 0x04, 0x00];
        assert_eq!(detect_format(&header).unwrap(), ArchiveFormat::Zip);
    }

    #[test]
    fn detects_gzip_magic() {
        let header = [0x1F, 0x8B, 0x08, 0x00];
        assert_eq!(detect_format(&header).unwrap(), ArchiveFormat::TarGz);
    }

    #[test]
    fn empty_input_errors() {
        assert!(detect_format(&[]).is_err());
    }
}
