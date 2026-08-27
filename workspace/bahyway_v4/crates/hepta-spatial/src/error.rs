#![forbid(unsafe_code)]

/// All errors produced by the hepta parser and spatial layer.
#[derive(Debug, Clone, PartialEq)]
pub enum HeptaError {
    ParseError { line: usize, message: String },
    UnknownCoordSystem(String),
    InvalidChordStep(u8),
    SectorOutOfRange(u8),
    MaxZoomReached,
    LongitudeOutOfRange(f64),
    LatitudeOutOfRange(f64),
    DensityOutOfRange(u8),
    MalformedHex(String),
    MissingDirective(&'static str),
    DuplicateLabel(String),
    InvalidNodeRef(String),
}

impl core::fmt::Display for HeptaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HeptaError::ParseError { line, message } => {
                write!(f, "hepta parse error at line {}: {}", line, message)
            }
            HeptaError::UnknownCoordSystem(s) => write!(f, "unknown coordinate system: '{}'", s),
            HeptaError::InvalidChordStep(s) => {
                write!(f, "invalid chord step '{}': must be 2 or 3", s)
            }
            HeptaError::SectorOutOfRange(n) => {
                write!(f, "sector index {} out of range 0\u{2013}7", n)
            }
            HeptaError::MaxZoomReached => write!(f, "maximum Hobble-Zoom depth reached (2 levels)"),
            HeptaError::LongitudeOutOfRange(v) => {
                write!(f, "longitude {} out of range \u{2212}180..+180", v)
            }
            HeptaError::LatitudeOutOfRange(v) => {
                write!(f, "latitude {} out of range \u{2212}90..+90", v)
            }
            HeptaError::DensityOutOfRange(v) => write!(f, "density {} out of range 0\u{2013}63", v),
            HeptaError::MalformedHex(s) => write!(f, "malformed hex literal: '{}'", s),
            HeptaError::MissingDirective(d) => write!(f, "missing required directive: {}", d),
            HeptaError::DuplicateLabel(s) => write!(f, "duplicate block label: '{}'", s),
            HeptaError::InvalidNodeRef(s) => write!(f, "invalid node ref: '{}'", s),
        }
    }
}
