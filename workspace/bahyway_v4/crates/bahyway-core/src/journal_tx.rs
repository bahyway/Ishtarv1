//! JournalTransaction — Ṭupšarrūtu Axiom A4 (Three-Tier Journal Law).
//!
//! Every state-changing operation MUST produce atomically:
//!   1. A crate WAL entry         (durability)
//!   2. A StoryWay event          (history)
//!   3. An EnkiWay update         (queryability)
//!
//! All three succeed together or all three fail. No torn writes.
//!
//! This module contains the trait kernel and the error type.  Concrete
//! implementations live in `enkidb-journal` (the operational layer).

/// Error variants for kernel-level journal operations.
///
/// Implementations may wrap these in richer error types but MUST be
/// able to map their errors to one of these variants for A4 enforcement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalError {
    /// A3 violated: attempt to transition to Gray without an Evidence event.
    A3Violation { sequence: u32 },
    /// A4 violated: WAL write succeeded but StoryWay or EnkiWay write failed
    /// (torn write — must be rolled back).
    TornWrite { tier: JournalTier },
    /// The underlying storage tier is full or unavailable.
    StorageFull { tier: JournalTier },
    /// The event sequence is out of order (A2: append-only).
    OutOfOrder { expected: u32, got: u32 },
    /// Generic failure — use only when no other variant applies.
    Internal,
}

/// Which of the three A4 tiers is affected by a JournalError.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalTier {
    /// Crate-level Write-Ahead Log (durability).
    Wal,
    /// StoryWay event stream (history).
    Story,
    /// EnkiWay queryable projection (queryability).
    Enki,
}

impl JournalTier {
    pub fn as_str(self) -> &'static str {
        match self {
            JournalTier::Wal   => "WAL",
            JournalTier::Story => "STORY",
            JournalTier::Enki  => "ENKI",
        }
    }
}

impl core::fmt::Display for JournalTier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::fmt::Display for JournalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            JournalError::A3Violation { sequence } =>
                write!(f, "A3: Gray lane at seq {sequence} has no Evidence"),
            JournalError::TornWrite { tier } =>
                write!(f, "A4: torn write — {tier} tier failed after WAL"),
            JournalError::StorageFull { tier } =>
                write!(f, "storage full on {tier} tier"),
            JournalError::OutOfOrder { expected, got } =>
                write!(f, "A2: out-of-order event (expected seq {expected}, got {got})"),
            JournalError::Internal =>
                write!(f, "internal journal error"),
        }
    }
}

/// The minimum trait surface required by Ṭupšarrūtu Axiom A4.
///
/// Any state-changing operation that touches particles MUST go through a
/// type that implements this trait — it is the only public API allowed for
/// writes.  Bypassing it violates A4.
///
/// `commit` writes WAL + Story + Enki atomically.  On any partial failure,
/// implementations MUST roll back or surface a `TornWrite` error.
/// `abort` discards the pending write — no tier is touched.
pub trait JournalTransaction: Sized {
    /// Write WAL + Story + Enki atomically, consuming the transaction.
    fn commit(self) -> Result<(), JournalError>;

    /// Discard this transaction without touching any storage tier.
    fn abort(self);
}

/// A no-op transaction used for kernel-level tests and placeholder
/// implementations that haven't yet been wired to real storage.
pub struct NoopTransaction {
    should_fail: bool,
}

impl NoopTransaction {
    pub fn new() -> Self { Self { should_fail: false } }

    /// Returns a transaction that always fails on commit (for negative tests).
    pub fn failing() -> Self { Self { should_fail: true } }
}

impl Default for NoopTransaction {
    fn default() -> Self { Self::new() }
}

impl JournalTransaction for NoopTransaction {
    fn commit(self) -> Result<(), JournalError> {
        if self.should_fail {
            Err(JournalError::Internal)
        } else {
            Ok(())
        }
    }

    fn abort(self) {
        // no-op — nothing to clean up
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_commit_succeeds() {
        let tx = NoopTransaction::new();
        assert!(tx.commit().is_ok());
    }

    #[test]
    fn noop_failing_commit_errors() {
        let tx = NoopTransaction::failing();
        assert_eq!(tx.commit(), Err(JournalError::Internal));
    }

    #[test]
    fn noop_abort_does_not_panic() {
        NoopTransaction::new().abort();
        NoopTransaction::failing().abort();
    }

    #[test]
    fn journal_error_display_a3() {
        let e = JournalError::A3Violation { sequence: 7 };
        assert!(e.to_string().contains("seq 7"));
        assert!(e.to_string().contains("A3"));
    }

    #[test]
    fn journal_error_display_torn_write() {
        let e = JournalError::TornWrite { tier: JournalTier::Story };
        assert!(e.to_string().contains("A4"));
        assert!(e.to_string().contains("STORY"));
    }

    #[test]
    fn journal_error_display_out_of_order() {
        let e = JournalError::OutOfOrder { expected: 3, got: 5 };
        assert!(e.to_string().contains("A2"));
        assert!(e.to_string().contains("expected seq 3"));
    }

    #[test]
    fn journal_tier_display() {
        assert_eq!(JournalTier::Wal.to_string(),   "WAL");
        assert_eq!(JournalTier::Story.to_string(), "STORY");
        assert_eq!(JournalTier::Enki.to_string(),  "ENKI");
    }

    #[test]
    fn noop_tx_default() {
        let tx: NoopTransaction = Default::default();
        assert!(tx.commit().is_ok());
    }
}
