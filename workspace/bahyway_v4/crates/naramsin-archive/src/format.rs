#![forbid(unsafe_code)]

/// Archive format variants supported by NARAMSIN Stage 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// ZIP archive (including nested zip-of-zips).
    Zip,
    /// Gzip-compressed tar.
    TarGz,
    /// Bzip2-compressed tar.
    TarBz2,
    /// XZ-compressed tar.
    TarXz,
    /// 7-Zip archive.
    SevenZip,
    /// Single uncompressed file (no archive wrapper).
    None,
}

impl ArchiveFormat {
    /// Magic-byte signature for this format, used by Stage 0 detection.
    pub fn magic(&self) -> &'static [u8] {
        match self {
            Self::Zip => &[0x50, 0x4B, 0x03, 0x04],
            Self::TarGz => &[0x1F, 0x8B],
            Self::TarBz2 => &[0x42, 0x5A, 0x68],
            Self::TarXz => &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00],
            Self::SevenZip => &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C],
            Self::None => &[],
        }
    }

    /// Human-readable name for audit logging.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::TarGz => "tar.gz",
            Self::TarBz2 => "tar.bz2",
            Self::TarXz => "tar.xz",
            Self::SevenZip => "7z",
            Self::None => "none",
        }
    }
}
