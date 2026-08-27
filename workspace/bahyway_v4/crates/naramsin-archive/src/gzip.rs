#![forbid(unsafe_code)]
//! gzip header unwrapper — RFC 1952.
//! Unwraps the gzip envelope and calls inflate on the compressed payload.

use crate::error::ArchiveError;
use crate::zip::{crc32, inflate_raw};

const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];
const CM_DEFLATE: u8 = 8;

// Flag bits
const FHCRC: u8 = 1 << 1;
const FEXTRA: u8 = 1 << 2;
const FNAME: u8 = 1 << 3;
const FCOMMENT: u8 = 1 << 4;

/// Unwrap a gzip stream and return the raw uncompressed bytes.
pub fn unwrap_gzip(data: &[u8]) -> Result<Vec<u8>, ArchiveError> {
    if data.len() < 10 {
        return Err(ArchiveError::Truncated);
    }
    if data[0..2] != GZIP_MAGIC {
        return Err(ArchiveError::UnknownFormat);
    }
    if data[2] != CM_DEFLATE {
        return Err(ArchiveError::Truncated);
    }

    let flags = data[3];
    // bytes 4..8 = mtime, byte 8 = xfl, byte 9 = os — skip all
    let mut pos = 10usize;

    // FEXTRA: skip extra field
    if flags & FEXTRA != 0 {
        if pos + 2 > data.len() {
            return Err(ArchiveError::Truncated);
        }
        let xlen = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2 + xlen;
    }

    // FNAME: skip null-terminated filename
    if flags & FNAME != 0 {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        if pos >= data.len() {
            return Err(ArchiveError::Truncated);
        }
        pos += 1; // skip null terminator
    }

    // FCOMMENT: skip null-terminated comment
    if flags & FCOMMENT != 0 {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        if pos >= data.len() {
            return Err(ArchiveError::Truncated);
        }
        pos += 1;
    }

    // FHCRC: 2-byte header CRC
    if flags & FHCRC != 0 {
        if pos + 2 > data.len() {
            return Err(ArchiveError::Truncated);
        }
        pos += 2;
    }

    // Trailer: 4-byte CRC-32 + 4-byte ISIZE at the end
    if data.len() < pos + 8 {
        return Err(ArchiveError::Truncated);
    }
    let trailer_start = data.len() - 8;
    let expected_crc =
        u32::from_le_bytes(data[trailer_start..trailer_start + 4].try_into().unwrap());
    let expected_size = u32::from_le_bytes(
        data[trailer_start + 4..trailer_start + 8]
            .try_into()
            .unwrap(),
    );

    let compressed = &data[pos..trailer_start];

    let decompressed =
        inflate_raw(compressed, expected_size as usize).ok_or(ArchiveError::Truncated)?;

    // Verify CRC-32 and ISIZE
    if crc32(&decompressed) != expected_crc {
        return Err(ArchiveError::Truncated);
    }
    if (decompressed.len() as u32) != expected_size {
        return Err(ArchiveError::Truncated);
    }

    Ok(decompressed)
}
