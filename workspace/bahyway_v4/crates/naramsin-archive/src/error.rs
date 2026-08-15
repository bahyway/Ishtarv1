#![forbid(unsafe_code)]

/// Stage 0 archive error — maps directly to NRM_* integrity codes (EN-NARAMSIN-001 §5).
#[derive(Debug, Clone, PartialEq)]
pub enum ArchiveError {
    /// Archive truncated mid-transfer — missing end-of-central-directory or equivalent.
    /// Maps to NRM_TRUNCATED (reject, full, no partial extraction).
    Truncated,
    /// Archive or file extension/signature does not match any supported format.
    /// Maps to NRM_UNKNOWN_FORMAT (reject, surfaced for architect review).
    UnknownFormat,
    /// Input byte slice was empty.
    EmptyInput,
    /// Nested archive recursion exceeded sovereign depth limit (prevents zip-bomb).
    MaxRecursionDepth(u8),
    /// Format is recognised but the sovereign decompression module is not yet implemented.
    UnsupportedFormat(&'static str),
}

impl core::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Truncated              => write!(f, "NARAMSIN NRM_TRUNCATED: archive truncated mid-transfer"),
            Self::UnknownFormat          => write!(f, "NARAMSIN NRM_UNKNOWN_FORMAT: unsupported archive format"),
            Self::EmptyInput             => write!(f, "NARAMSIN: empty input byte slice"),
            Self::MaxRecursionDepth(d)   => write!(f, "NARAMSIN: max nested archive depth {d} exceeded"),
            Self::UnsupportedFormat(msg) => write!(f, "NARAMSIN NRM_UNSUPPORTED: {msg}"),
        }
    }
}
