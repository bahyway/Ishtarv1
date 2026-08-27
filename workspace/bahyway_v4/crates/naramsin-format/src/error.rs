#![forbid(unsafe_code)]

/// Stage 1 format parsing error — maps to NRM_* integrity codes.
#[derive(Debug, Clone, PartialEq)]
pub enum FormatError {
    /// CSV row has a different field count than the header row.
    /// Maps to NRM_MALFORMED.
    RaggedCsvRow { row: u32, expected: u32, got: u32 },
    /// JSON document fails to parse (trailing comma, unmatched bracket, etc.).
    /// Maps to NRM_MALFORMED.
    InvalidJson {
        byte_offset: usize,
        reason: &'static str,
    },
    /// XML document has an unclosed tag or otherwise fails to parse.
    /// Maps to NRM_MALFORMED.
    InvalidXml { line: u32, reason: &'static str },
    /// Column/field set differs from the last known schema for this source.
    /// Maps to NRM_SCHEMA_DRIFT (advisory, not rejection).
    SchemaDrift { source_id: u64 },
}

impl core::fmt::Display for FormatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::RaggedCsvRow { row, expected, got } => write!(
                f,
                "NARAMSIN NRM_MALFORMED: CSV row {row} has {got} fields, expected {expected}"
            ),
            Self::InvalidJson {
                byte_offset,
                reason,
            } => write!(
                f,
                "NARAMSIN NRM_MALFORMED: JSON parse error at byte {byte_offset}: {reason}"
            ),
            Self::InvalidXml { line, reason } => write!(
                f,
                "NARAMSIN NRM_MALFORMED: XML parse error at line {line}: {reason}"
            ),
            Self::SchemaDrift { source_id } => write!(
                f,
                "NARAMSIN NRM_SCHEMA_DRIFT: schema changed for source {source_id}"
            ),
        }
    }
}
