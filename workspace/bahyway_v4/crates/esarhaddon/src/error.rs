#![forbid(unsafe_code)]

/// Sovereign error type for ESARHADDON earthquake rescue intelligence.
#[derive(Debug, Clone, PartialEq)]
pub enum EsarhaddonError {
    /// Seismic particle missing required EAV field.
    MissingEavField(&'static str),
    /// SMI calculation requires at least one sensor reading.
    InsufficientSensorData,
    /// HeptaMap cell reference out of valid E7 lattice range.
    InvalidHeptaCell(u16),
    /// Signal fusion requires at least two corroborating sources.
    InsufficientSignalSources,
    /// Gas dispersion model received invalid concentration value.
    InvalidConcentration(f32),
}

impl core::fmt::Display for EsarhaddonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingEavField(field) => write!(f, "ESARHADDON: missing EAV field '{field}'"),
            Self::InsufficientSensorData => write!(f, "ESARHADDON: insufficient sensor data for SMI"),
            Self::InvalidHeptaCell(id) => write!(f, "ESARHADDON: invalid HeptaMap cell {id}"),
            Self::InsufficientSignalSources => write!(f, "ESARHADDON: signal fusion requires ≥2 sources"),
            Self::InvalidConcentration(v) => write!(f, "ESARHADDON: invalid gas concentration {v}"),
        }
    }
}
