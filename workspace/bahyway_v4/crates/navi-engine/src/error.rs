//! NaviError — all failure modes for the NaviEngine routing pipeline.

use core::fmt;

pub type NaviResult<T> = Result<T, NaviError>;

/// Every error that can arise in the NaviEngine pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum NaviError {
    /// A particle failed structural or sovereignty validation.
    InvalidParticle(String),
    /// No route exists between origin and destination.
    RouteNotFound,
    /// Graph was used before a map was loaded.
    MapNotLoaded,
    /// Anti-spoofing check detected a tampered NaviSignal.
    SpoofDetected,
    /// A tribe sovereignty rule was violated.
    SovereignViolation(String),
    /// The NaviMap source text could not be parsed.
    MapParseError(String),
    /// The graph has no nodes — routing is impossible.
    EmptyGraph,
}

impl fmt::Display for NaviError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NaviError::InvalidParticle(r) => write!(f, "invalid particle: {r}"),
            NaviError::RouteNotFound => write!(f, "route not found"),
            NaviError::MapNotLoaded => write!(f, "map not loaded"),
            NaviError::SpoofDetected => write!(f, "spoof detected"),
            NaviError::SovereignViolation(r) => write!(f, "sovereignty violation: {r}"),
            NaviError::MapParseError(r) => write!(f, "map parse error: {r}"),
            NaviError::EmptyGraph => write!(f, "empty graph"),
        }
    }
}

impl core::error::Error for NaviError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_not_found_display() {
        assert!(NaviError::RouteNotFound.to_string().contains("route"));
    }
    #[test]
    fn map_not_loaded_display() {
        assert!(NaviError::MapNotLoaded.to_string().contains("map"));
    }
    #[test]
    fn spoof_detected_display() {
        assert!(NaviError::SpoofDetected.to_string().contains("spoof"));
    }
    #[test]
    fn invalid_particle_display() {
        assert!(NaviError::InvalidParticle("bad crc".into())
            .to_string()
            .contains("bad crc"));
    }
    #[test]
    fn sovereign_violation_display() {
        assert!(NaviError::SovereignViolation("tribe 0x0099".into())
            .to_string()
            .contains("sovereignty"));
    }
    #[test]
    fn map_parse_error_display() {
        assert!(NaviError::MapParseError("line 7".into())
            .to_string()
            .contains("map parse"));
    }
    #[test]
    fn empty_graph_display() {
        assert!(NaviError::EmptyGraph.to_string().contains("empty"));
    }
    #[test]
    fn error_is_clone() {
        assert_eq!(NaviError::RouteNotFound.clone(), NaviError::RouteNotFound);
    }
    #[test]
    fn error_is_partial_eq() {
        assert_eq!(NaviError::MapNotLoaded, NaviError::MapNotLoaded);
        assert_ne!(NaviError::MapNotLoaded, NaviError::EmptyGraph);
    }
}
