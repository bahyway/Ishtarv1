#![forbid(unsafe_code)]

/// Sovereign error type for ASHNAN climate-agriculture-livestock intelligence.
#[derive(Debug, Clone, PartialEq)]
pub enum AshnanError {
    /// Index calculation received an out-of-range factor.
    InvalidFactor { factor: &'static str, value: f32 },
    /// External ingestion particle missing mandatory provenance EAV field.
    MissingProvenance(&'static str),
    /// Pest trap data insufficient for IRI calculation.
    InsufficientTrapData,
    /// Livestock herd sensor data below minimum threshold for LMI.
    InsufficientHerdData,
}

impl core::fmt::Display for AshnanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFactor { factor, value } => {
                write!(f, "ASHNAN: invalid factor '{factor}' = {value}")
            }
            Self::MissingProvenance(field) => {
                write!(f, "ASHNAN: missing provenance EAV field '{field}'")
            }
            Self::InsufficientTrapData => write!(f, "ASHNAN: insufficient pest trap data for IRI"),
            Self::InsufficientHerdData => {
                write!(f, "ASHNAN: insufficient herd sensor data for LMI")
            }
        }
    }
}
