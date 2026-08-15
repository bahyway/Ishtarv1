//! ErrorTypeSpec / ErrorOccurrenceSpec — the Error Registry + Journal domain
//! types (EnkiMDB internal registries, 2026-07-24).
//!
//! Each ErrorType is a Particle: registered once as an Identity-Kaki
//! (role=Parzu — "logic, template, axiom, or rule", the same role this
//! ecosystem already uses for templates), via `EventCause::ErrorTypeRegistered`.
//! Every later firing of that error is a fresh Event-Kaki whose
//! `JournalEntry::target_kaki` is that same Identity-Kaki — the exact
//! "N events, one target" shape `enkidb_journal::Journal` already supports
//! (see `Journal::read_particle_history`), via `EventCause::ErrorOccurred`.
//!
//! This crate defines the domain types only. Minting the Kakis and
//! journaling them is `enkimdb::writenode::WriteNode`'s job (it already
//! owns a `KakiMinter` + `Journal`, the same pattern `ingest_artifact`
//! uses) — kept out of this crate to avoid enkidb-journal depending on
//! enkidb-particles/akkvalue for something only the write path needs.

/// Severity of an ErrorType — informational only, never itself a gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorSeverity {
    Info     = 0,
    Warning  = 1,
    Error    = 2,
    Critical = 3,
}

impl ErrorSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorSeverity::Info     => "INFO",
            ErrorSeverity::Warning  => "WARNING",
            ErrorSeverity::Error    => "ERROR",
            ErrorSeverity::Critical => "CRITICAL",
        }
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Info),
            1 => Some(Self::Warning),
            2 => Some(Self::Error),
            3 => Some(Self::Critical),
            _ => None,
        }
    }
}

/// A registered error type — the Registry side. One of these exists per
/// distinct error code, ever (low cardinality, rarely written).
#[derive(Debug, Clone)]
pub struct ErrorTypeSpec {
    /// Stable code, e.g. "LEMNI_CONSONANT_COUNT". This is the lookup key
    /// services use — NOT the Kaki itself, which is only known after
    /// registration.
    pub code: &'static str,
    /// Short, human-readable summary — what a status line shows.
    pub summary: &'static str,
    pub severity: ErrorSeverity,
    /// Which internal service/crate first defined this error type,
    /// e.g. "kupru".
    pub defined_by: &'static str,
}

impl ErrorTypeSpec {
    pub const fn new(
        code: &'static str,
        summary: &'static str,
        severity: ErrorSeverity,
        defined_by: &'static str,
    ) -> Self {
        ErrorTypeSpec { code, summary, severity, defined_by }
    }
}

/// One firing of an already-registered ErrorType — the Journal side.
/// High cardinality, append-heavy.
#[derive(Debug, Clone)]
pub struct ErrorOccurrenceSpec {
    /// Which internal service logged THIS firing, e.g.
    /// "sargon-passport-manager". Distinct from `ErrorTypeSpec::defined_by`
    /// -- a type can fire from a different service than the one that
    /// defined it.
    pub source: &'static str,
    /// The fuller, possibly-dynamic explanation for this specific firing
    /// (e.g. "1 of 3 shares gathered" — the exact numbers involved).
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_round_trips() {
        for s in [ErrorSeverity::Info, ErrorSeverity::Warning, ErrorSeverity::Error, ErrorSeverity::Critical] {
            assert_eq!(ErrorSeverity::from_byte(s.to_byte()), Some(s));
        }
    }

    #[test]
    fn unknown_severity_byte_returns_none() {
        assert_eq!(ErrorSeverity::from_byte(0xFF), None);
    }

    #[test]
    fn severity_as_str() {
        assert_eq!(ErrorSeverity::Critical.as_str(), "CRITICAL");
        assert_eq!(ErrorSeverity::Info.as_str(), "INFO");
    }

    #[test]
    fn error_type_spec_const_construction() {
        const SPEC: ErrorTypeSpec = ErrorTypeSpec::new(
            "LEMNI_CONSONANT_COUNT",
            "Identity phrase needs at least 3 consonants.",
            ErrorSeverity::Error,
            "kupru",
        );
        assert_eq!(SPEC.code, "LEMNI_CONSONANT_COUNT");
        assert_eq!(SPEC.severity, ErrorSeverity::Error);
    }
}
