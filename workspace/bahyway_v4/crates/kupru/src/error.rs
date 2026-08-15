//! KupruError — sovereign error taxonomy.

/// Sovereign error variants for the KUPRU crypto layer.
#[derive(Debug, Clone)]
pub enum KupruError {
    InvalidInput(String),
    SealBroken(String),
    Io(String),
    Crypto(String),
    /// Credential has exceeded ŠATTĀTU_MAX (54h) or clock-skew limit.
    Expired(String),
    /// Operation denied by sovereign policy (DŪRU).
    AccessDenied(String),
}

impl std::fmt::Display for KupruError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(m)  => write!(f, "LEMNĪ (invalid input): {m}"),
            Self::SealBroken(m)    => write!(f, "NAKRU (seal broken): {m}"),
            Self::Io(m)            => write!(f, "KUPRU IO: {m}"),
            Self::Crypto(m)        => write!(f, "KUPRU crypto: {m}"),
            Self::Expired(m)       => write!(f, "ŠANÛ ŠATTĀTI (expired): {m}"),
            Self::AccessDenied(m)  => write!(f, "DŪRU (access denied): {m}"),
        }
    }
}

impl std::error::Error for KupruError {}

pub type KupruResult<T> = Result<T, KupruError>;
