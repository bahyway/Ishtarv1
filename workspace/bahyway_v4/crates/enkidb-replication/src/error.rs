#![forbid(unsafe_code)]

/// Errors produced by the replication pipeline.
#[derive(Debug)]
pub enum ReplicationError {
    /// Sequence number gap — expected N, got M.
    SequenceGap { expected: u64, got: u64 },
    /// Chained digest does not match previous event's digest.
    ChainBroken,
    /// Ed25519 KANĀKU seal failed verification.
    SealInvalid,
    /// SHA3-256 payload digest does not match.
    DigestMismatch,
    /// Event epoch is older than REPL_MAX_SECS.
    EpochExpired { event_epoch: u32, now_epoch: u32 },
    /// Write Pod SargonPassport has expired (SATTATU_MAX).
    PassportExpired,
    /// Write Pod SargonPassport lacks "replication:emit" scope.
    PassportScopeInsufficient,
    /// HeptaSecSentinel blocked the Write Pod's KAKI.
    KakiBlocked,
    /// Frame magic bytes are wrong — corrupt or truncated log.
    InvalidFrame,
    /// I/O error reading or writing the log.
    Io(String),
    /// A verified event's `delta` bytes did not decode as the expected
    /// application payload (e.g. `JournalEntry::from_bytes` in
    /// `journal_bridge` — see that module for the real materialization
    /// wiring). The frame's own cryptographic integrity already passed;
    /// this is strictly an application-layer decode failure.
    DeltaDecodeFailed,
}

impl core::fmt::Display for ReplicationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ReplicationError::SequenceGap { expected, got } =>
                write!(f, "sequence gap: expected {expected}, got {got}"),
            ReplicationError::ChainBroken =>
                write!(f, "chain broken: prev_digest mismatch"),
            ReplicationError::SealInvalid =>
                write!(f, "KANĀKU seal invalid: Ed25519 verification failed"),
            ReplicationError::DigestMismatch =>
                write!(f, "ŠIPIR ŠARRI digest mismatch: payload corrupted"),
            ReplicationError::EpochExpired { event_epoch, now_epoch } =>
                write!(f, "epoch expired: event={event_epoch} now={now_epoch} delta={}s",
                       now_epoch.saturating_sub(*event_epoch)),
            ReplicationError::PassportExpired =>
                write!(f, "SargonPassport expired (SATTATU_MAX)"),
            ReplicationError::PassportScopeInsufficient =>
                write!(f, "SargonPassport lacks replication:emit scope"),
            ReplicationError::KakiBlocked =>
                write!(f, "HeptaSecSentinel blocked Write Pod KAKI"),
            ReplicationError::InvalidFrame =>
                write!(f, "invalid frame: bad magic or truncated log"),
            ReplicationError::Io(e) =>
                write!(f, "I/O error: {e}"),
            ReplicationError::DeltaDecodeFailed =>
                write!(f, "verified event's delta bytes did not decode as the expected payload"),
        }
    }
}

impl From<std::io::Error> for ReplicationError {
    fn from(e: std::io::Error) -> Self {
        ReplicationError::Io(e.to_string())
    }
}

pub type ReplicationResult<T> = Result<T, ReplicationError>;
