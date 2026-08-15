#![forbid(unsafe_code)]
//! ConError — connection engine error variants (no thiserror).

use crate::roles::SovereignRole;

#[derive(Debug, Clone, PartialEq)]
pub enum ConError {
    /// CSR-01 Sargon gate: passport not valid.
    Forbidden(&'static str),
    /// CSR-02 insufficient role.
    InsufficientRole { needed: SovereignRole, got: SovereignRole },
    /// CSR-03 NĀRU WAL journal is full.
    AuditJournalFull,
    /// CSR-04 credential expired.
    CredentialExpired,
    /// CSR-05 Gilgamesh gate: cross-tribe write blocked.
    GilgameshBlocked(&'static str),
    /// CSR-06 Kibratu event emission failed.
    KibratuEventFailed,
    /// CSR-07 tribe isolation violation.
    TribeIsolationViolation { caller_tribe: u32, target_tribe: u32 },
    /// Connection pool exhausted.
    PoolExhausted,
    /// I/O error string.
    Io(String),
}

impl core::fmt::Display for ConError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Forbidden(msg)
                => write!(f, "CSR-01 Sargon gate FORBIDDEN: {msg}"),
            Self::InsufficientRole { needed, got }
                => write!(f, "CSR-02 insufficient role: need {needed:?}, got {got:?}"),
            Self::AuditJournalFull
                => write!(f, "CSR-03 NĀRU WAL journal is full"),
            Self::CredentialExpired
                => write!(f, "CSR-04 credential expired"),
            Self::GilgameshBlocked(msg)
                => write!(f, "CSR-05 Gilgamesh gate blocked: {msg}"),
            Self::KibratuEventFailed
                => write!(f, "CSR-06 Kibratu event emission failed"),
            Self::TribeIsolationViolation { caller_tribe, target_tribe }
                => write!(f, "CSR-07 tribe isolation: caller tribe {caller_tribe} may not access tribe {target_tribe}"),
            Self::PoolExhausted
                => write!(f, "connection pool exhausted"),
            Self::Io(msg)
                => write!(f, "I/O error: {msg}"),
        }
    }
}
