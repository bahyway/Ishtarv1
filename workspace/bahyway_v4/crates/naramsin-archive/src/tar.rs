#![forbid(unsafe_code)]
//! POSIX tar reader — 512-byte headers, regular files only.

use crate::error::ArchiveError;
use crate::zip::ExtractedFile;

const BLOCK: usize = 512;

/// Extract regular files from a tar byte slice.
/// Returns `ArchiveError::Truncated` for corrupt or truncated input.
pub fn extract_tar(data: &[u8]) -> Result<Vec<ExtractedFile>, ArchiveError> {
    if data.is_empty() {
        return Err(ArchiveError::EmptyInput);
    }
    let mut results = Vec::new();
    let mut pos = 0usize;
    let mut zero_blocks = 0u32;

    loop {
        if pos + BLOCK > data.len() {
            // End of data without two zero blocks — treat as truncated only if
            // we haven't seen any zero blocks yet and there are files
            if zero_blocks == 0 && !results.is_empty() {
                return Err(ArchiveError::Truncated);
            }
            break;
        }

        let hdr = &data[pos..pos + BLOCK];

        // Check for zero block (EOF marker)
        if hdr.iter().all(|&b| b == 0) {
            zero_blocks += 1;
            pos += BLOCK;
            if zero_blocks >= 2 {
                break;
            }
            continue;
        }
        zero_blocks = 0;

        // Parse header fields
        let name_bytes = &hdr[0..100];
        let name = parse_string(name_bytes);

        let size_bytes = &hdr[124..136];
        let size = parse_octal(size_bytes).ok_or(ArchiveError::Truncated)?;

        let typeflag = hdr[156];

        pos += BLOCK;

        // Data blocks
        let data_blocks = (size + BLOCK - 1) / BLOCK;
        let data_end = pos + data_blocks * BLOCK;

        if data_end > data.len() {
            return Err(ArchiveError::Truncated);
        }

        match typeflag {
            b'0' | 0 => {
                // Regular file
                if size > data.len() {
                    return Err(ArchiveError::Truncated);
                }
                let file_data = data[pos..pos + size].to_vec();
                if !name.is_empty() {
                    results.push(ExtractedFile { name, data: file_data });
                }
            }
            b'5' => { /* directory — skip */ }
            b'1' | b'2' => { /* hard/symlink — skip */ }
            _ => { /* unknown type — skip */ }
        }

        pos = data_end;
    }

    Ok(results)
}

fn parse_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn parse_octal(bytes: &[u8]) -> Option<usize> {
    // Skip leading spaces and nulls, take octal digits
    let s = bytes.iter()
        .skip_while(|&&b| b == b' ' || b == 0)
        .take_while(|&&b| b >= b'0' && b <= b'7')
        .copied()
        .collect::<Vec<u8>>();
    if s.is_empty() { return Some(0); }
    let s = core::str::from_utf8(&s).ok()?;
    usize::from_str_radix(s, 8).ok()
}
