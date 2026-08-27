//! NajafError — domain errors for the cemetery navigation engine.

use crate::sector::NajafSector;
use core::fmt;
use navi_engine::NaviError;

#[derive(Debug)]
pub enum NajafError {
    /// Grave ID not found in the registry.
    GraveNotFound(u32),
    /// The target sector is sealed or inaccessible.
    SectorBlocked(NajafSector),
    /// Underlying routing failure from NaviEngine.
    RouteError(NaviError),
    /// Invalid or malformed grave identifier.
    InvalidGraveId(String),
    /// Cemetery registry contains no graves.
    EmptyCemetery,
    /// Pilgrim destination has no accessible graves in that sector.
    NoAccessibleGravesInSector(NajafSector),
}

pub type NajafResult<T> = Result<T, NajafError>;

impl fmt::Display for NajafError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NajafError::GraveNotFound(id) => write!(f, "grave {id} not found"),
            NajafError::SectorBlocked(s) => write!(f, "sector {:?} is blocked", s),
            NajafError::RouteError(e) => write!(f, "route error: {e}"),
            NajafError::InvalidGraveId(msg) => write!(f, "invalid grave id: {msg}"),
            NajafError::EmptyCemetery => write!(f, "cemetery registry is empty"),
            NajafError::NoAccessibleGravesInSector(s) => {
                write!(f, "no accessible graves in {:?}", s)
            }
        }
    }
}

impl From<NaviError> for NajafError {
    fn from(e: NaviError) -> Self {
        NajafError::RouteError(e)
    }
}

impl core::error::Error for NajafError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grave_not_found_display() {
        let e = NajafError::GraveNotFound(42);
        assert!(e.to_string().contains("42"));
    }

    #[test]
    fn sector_blocked_display() {
        let e = NajafError::SectorBlocked(NajafSector::Shuhadaa);
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn route_error_wraps_navi_error() {
        let e: NajafError = NaviError::RouteNotFound.into();
        assert!(matches!(e, NajafError::RouteError(_)));
    }

    #[test]
    fn invalid_grave_id_display() {
        let e = NajafError::InvalidGraveId("bad_id".into());
        assert!(e.to_string().contains("bad_id"));
    }

    #[test]
    fn empty_cemetery_display() {
        let e = NajafError::EmptyCemetery;
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn no_accessible_graves_display() {
        let e = NajafError::NoAccessibleGravesInSector(NajafSector::Anbiya);
        assert!(!e.to_string().is_empty());
    }
}
