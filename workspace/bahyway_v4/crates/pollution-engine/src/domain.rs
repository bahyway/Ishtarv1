//! # PollutionDomain — Domain classification
//! **PollutionEngine v4.0 | BahyWay.Ecosystem | DUB.SAR 𒁾**

/// The three monitored environmental domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollutionDomain {
    Air,
    Water,
    Oil,
}

impl std::fmt::Display for PollutionDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Air => write!(f, "AIR"),
            Self::Water => write!(f, "WATER"),
            Self::Oil => write!(f, "OIL"),
        }
    }
}

#[derive(Debug)]
pub enum PollutionError {
    InvalidDomain(String),
    InvalidCoordinates { lat: f64, lon: f64 },
}

impl std::fmt::Display for PollutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDomain(s) => write!(f, "invalid pollution domain: {}", s),
            Self::InvalidCoordinates { lat, lon } => {
                write!(f, "invalid coordinates: ({}, {})", lat, lon)
            }
        }
    }
}

impl std::error::Error for PollutionError {}
