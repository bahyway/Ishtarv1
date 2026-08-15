//! MmapReader — read-only file view loaded into memory.
//!
//! Pure std replacement for memmap2::Mmap: loads file contents into a Vec<u8>
//! so the rest of the codebase can use the same slice-based API.

use std::fs;
use std::path::Path;

/// Read-only in-memory view of a cold block file.
pub struct MmapReader {
    data: Vec<u8>,
}

impl MmapReader {
    /// Load a file completely into memory (read-only).
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let data = fs::read(path)?;
        Ok(MmapReader { data })
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Read a fixed-size value at the given byte offset.
    pub fn read_at<const N: usize>(&self, offset: usize) -> Option<[u8; N]> {
        self.data.get(offset..offset + N)?.try_into().ok()
    }
}
