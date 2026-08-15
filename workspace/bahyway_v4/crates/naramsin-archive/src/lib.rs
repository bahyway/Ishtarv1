#![forbid(unsafe_code)]
//! # NARAMSIN Stage 0 — Sovereign Archive Decompression
//!
//! Named for Naram-Sin (𒈎𒋀𒁲), grandson of Sargon of Akkad, who unified
//! conquered territories under a single sovereign rule. NARAMSIN is the
//! BahyWay.Ecosystem v4.0 sovereign archive and format reader engine,
//! sitting at the Engine Tier alongside ConEngine and GeoEngine (EN-NARAMSIN-001).
//!
//! This crate owns Stage 0: detect archive format, decompress, recurse into
//! nested archives, isolate per-file extraction. Truncated/corrupt archives
//! are classified here and rejected with an NRM_TRUNCATED code — the precise
//! failure mode demonstrated by the ASHNAN BeeMDM test corpus corrupted-zip.
//!
//! ## Supported Archive Formats (Stage 0)
//! - zip (including nested zip-of-zips)
//! - tar.gz
//! - tar.bz2
//! - tar.xz
//! - 7z
//!
//! ## Explicit Non-Goals (until a sovereign module has a justified requirement)
//! - RAR, ARJ, ACE, or any proprietary format

pub mod detect;
pub mod error;
pub mod format;
pub mod zip;
pub mod tar;
pub mod gzip;

pub use error::ArchiveError;
pub use format::ArchiveFormat;
pub use detect::detect_format;
pub use zip::ExtractedFile;

/// Maximum recursion depth for nested archives (zip-bomb guard).
const MAX_RECURSION: u8 = 4;

// ── Stubs for not-yet-sovereign formats ───────────────────────────────────────

pub fn unwrap_bzip2(_data: &[u8]) -> Result<Vec<u8>, ArchiveError> {
    Err(ArchiveError::UnsupportedFormat("bzip2 sovereign module pending"))
}

pub fn unwrap_xz(_data: &[u8]) -> Result<Vec<u8>, ArchiveError> {
    Err(ArchiveError::UnsupportedFormat("xz sovereign module pending"))
}

pub fn unwrap_7z(_data: &[u8]) -> Result<Vec<u8>, ArchiveError> {
    Err(ArchiveError::UnsupportedFormat("7z sovereign module pending"))
}

// ── Main decompress entry point ───────────────────────────────────────────────

/// Stage 0 sovereign archive decompression.
/// MAX_RECURSION = 4 (zip-bomb guard).
pub fn decompress(data: &[u8], recursion_depth: u8) -> Result<Vec<ExtractedFile>, ArchiveError> {
    if data.is_empty() {
        return Err(ArchiveError::EmptyInput);
    }
    if recursion_depth > MAX_RECURSION {
        return Err(ArchiveError::MaxRecursionDepth(recursion_depth));
    }

    let fmt = detect_format(data)?;

    match fmt {
        ArchiveFormat::Zip => {
            let files = zip::extract_zip(data)?;
            // Check for nested zips and recurse
            let mut result = Vec::new();
            for file in files {
                if file.name.ends_with(".zip") {
                    match decompress(&file.data, recursion_depth + 1) {
                        Ok(nested) => result.extend(nested),
                        Err(e) => return Err(e),
                    }
                } else {
                    result.push(file);
                }
            }
            Ok(result)
        }
        ArchiveFormat::TarGz => {
            let raw = gzip::unwrap_gzip(data)?;
            tar::extract_tar(&raw)
        }
        ArchiveFormat::TarBz2 => {
            let _raw = unwrap_bzip2(data)?;
            unreachable!()
        }
        ArchiveFormat::TarXz => {
            let _raw = unwrap_xz(data)?;
            unreachable!()
        }
        ArchiveFormat::SevenZip => {
            let _raw = unwrap_7z(data)?;
            unreachable!()
        }
        ArchiveFormat::None => {
            Ok(vec![ExtractedFile { name: "raw_data".to_string(), data: data.to_vec() }])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::build_store_zip;

    #[test]
    fn decompress_valid_store_zip() {
        let content = b"hello sovereign world";
        let z = build_store_zip("hello.txt", content);
        let files = decompress(&z, 0).expect("should extract");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "hello.txt");
        assert_eq!(files[0].data, content);
    }

    #[test]
    fn decompress_truncated_zip_returns_error() {
        let content = b"hello sovereign world";
        let z = build_store_zip("hello.txt", content);
        let truncated = &z[..z.len() - 10];
        let err = decompress(truncated, 0).expect_err("should be truncated");
        assert_eq!(err, ArchiveError::Truncated);
    }

    #[test]
    fn decompress_empty_input_returns_error() {
        let err = decompress(&[], 0).expect_err("should be EmptyInput");
        assert_eq!(err, ArchiveError::EmptyInput);
    }

    #[test]
    fn decompress_recursion_depth_exceeded() {
        let err = decompress(b"dummy", 5).expect_err("should exceed max depth");
        assert_eq!(err, ArchiveError::MaxRecursionDepth(5));
    }

    #[test]
    fn decompress_plain_data_returns_raw_file() {
        // Plain data (no archive signature) → single ExtractedFile named "raw_data"
        let data = b"plaintext data no archive header";
        let files = decompress(data, 0).expect("should return raw_data");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "raw_data");
        assert_eq!(files[0].data, data);
    }

    #[test]
    fn decompress_valid_gzip_tar() {
        // Build a minimal tar in memory, then a minimal gzip wrapper
        // For this test we use a pre-built stored DEFLATE (stored block = no compression)
        // to create a valid gzip+tar stream.
        // We validate the full pipeline: gzip unwrap → tar extract
        use tar::extract_tar;

        // Build a simple tar with one file
        let file_content = b"sovereign particle data\n";
        let mut tar_data = Vec::new();

        // tar header: 512 bytes
        let mut hdr = [0u8; 512];
        // name
        let name = b"test.txt";
        hdr[..name.len()].copy_from_slice(name);
        // size in octal ASCII (24 bytes → "00000000030\0")
        let size_str = format!("{:011o}\0", file_content.len());
        hdr[124..124+size_str.len()].copy_from_slice(size_str.as_bytes());
        // typeflag = '0'
        hdr[156] = b'0';
        // checksum — compute simple sum
        let cksum: u32 = hdr.iter().map(|&b| b as u32).sum::<u32>() + 8 * b' ' as u32;
        let cksum_str = format!("{:06o}\0 ", cksum);
        hdr[148..148+cksum_str.len()].copy_from_slice(cksum_str.as_bytes());
        tar_data.extend_from_slice(&hdr);
        // data block (padded to 512)
        let mut data_block = [0u8; 512];
        data_block[..file_content.len()].copy_from_slice(file_content);
        tar_data.extend_from_slice(&data_block);
        // two zero blocks (EOF)
        tar_data.extend_from_slice(&[0u8; 1024]);

        // Verify tar extraction works directly
        let extracted = extract_tar(&tar_data).expect("tar extraction failed");
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].data, file_content);

        // Now wrap in gzip using stored DEFLATE block
        let gz = build_gzip_stored(&tar_data);
        let files = decompress(&gz, 0).expect("gzip+tar decompress failed");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "test.txt");
        assert_eq!(files[0].data, file_content);
    }

    /// Build a minimal gzip file wrapping `data` using a DEFLATE stored block.
    fn build_gzip_stored(data: &[u8]) -> Vec<u8> {
        let mut gz = Vec::new();
        // gzip header: ID1, ID2, CM, FLG, MTIME(4), XFL, OS
        gz.extend_from_slice(&[0x1F, 0x8B, 0x08, 0x00]);
        gz.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // mtime
        gz.push(0x00); // xfl
        gz.push(0xFF); // OS unknown

        // DEFLATE stored block: BFINAL=1, BTYPE=00 → byte 0x01
        // then align (no-op since 3 bits used, 5 padding bits remain in byte)
        // LEN as u16 LE, NLEN = ~LEN as u16 LE
        let len = data.len();
        // We need to handle data > 65535 with multiple stored blocks
        // For test data this is fine
        assert!(len <= 65535, "test helper only handles ≤65535 bytes");
        gz.push(0x01); // BFINAL=1, BTYPE=00 (stored)
        gz.extend_from_slice(&(len as u16).to_le_bytes());
        gz.extend_from_slice(&(!(len as u16)).to_le_bytes());
        gz.extend_from_slice(data);

        // Trailer: CRC-32 LE, ISIZE LE
        let crc = zip::crc32(data);
        gz.extend_from_slice(&crc.to_le_bytes());
        gz.extend_from_slice(&(len as u32).to_le_bytes());
        gz
    }
}
