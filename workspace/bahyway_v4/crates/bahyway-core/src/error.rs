//! BahyWay ecosystem error types — pure std, no thiserror.

use std::fmt;

pub type Result<T> = core::result::Result<T, BahywayError>;

#[derive(Debug)]
pub enum BahywayError {
    // ── KAKI integrity ─────────────────────────────────────────────────
    ChecksumMismatch { expected: u16, actual: u16 },
    InvalidKakiType(u8),
    InvalidKakiRole(u8),

    // ── Immutability violations ─────────────────────────────────────────
    KakiMutationForbidden,
    KakiReassignmentForbidden,

    // ── Append-only violations ──────────────────────────────────────────
    UpdateForbidden,
    DeleteForbidden,

    // ── Storage ────────────────────────────────────────────────────────
    BlockMagicMismatch([u8; 8]),
    Io(std::io::Error),

    // ── Generic ────────────────────────────────────────────────────────
    Internal(String),
}

impl fmt::Display for BahywayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BahywayError::ChecksumMismatch { expected, actual } => write!(
                f,
                "KAKI checksum mismatch: expected {expected:#06x}, got {actual:#06x}"
            ),
            BahywayError::InvalidKakiType(b) => write!(f, "Invalid KAKI type byte: {b:#04x}"),
            BahywayError::InvalidKakiRole(b) => write!(f, "Invalid KAKI role byte: {b:#04x}"),
            BahywayError::KakiMutationForbidden => {
                write!(f, "Attempted mutation of an immutable KAKI — Rule I")
            }
            BahywayError::KakiReassignmentForbidden => {
                write!(f, "Attempted reassignment of KAKI — Rule II")
            }
            BahywayError::UpdateForbidden => write!(
                f,
                "UPDATE operations do not exist; append a new Event-Kaki instead"
            ),
            BahywayError::DeleteForbidden => write!(
                f,
                "DELETE operations do not exist; decay is a state transition to DEAD"
            ),
            BahywayError::BlockMagicMismatch(got) => write!(
                f,
                "Block magic mismatch — expected b\"ENKIv4.0\", got {got:?}"
            ),
            BahywayError::Io(e) => write!(f, "I/O error: {e}"),
            BahywayError::Internal(s) => write!(f, "Internal error: {s}"),
        }
    }
}

impl std::error::Error for BahywayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BahywayError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BahywayError {
    fn from(e: std::io::Error) -> Self {
        BahywayError::Io(e)
    }
}
