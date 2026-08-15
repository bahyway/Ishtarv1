// ── WpdError ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WpdError {
    SegmentNotFound(String),
    InvalidScan(String),
    ClassificationFailed(String),
    RouteNotFound(String),
    NoBands,
}

impl core::fmt::Display for WpdError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WpdError::SegmentNotFound(id)       => write!(f, "segment not found: {id}"),
            WpdError::InvalidScan(msg)           => write!(f, "invalid scan: {msg}"),
            WpdError::ClassificationFailed(msg)  => write!(f, "classification failed: {msg}"),
            WpdError::RouteNotFound(msg)         => write!(f, "route not found: {msg}"),
            WpdError::NoBands                    => write!(f, "no spectral bands provided"),
        }
    }
}

pub type WpdResult<T> = Result<T, WpdError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_segment_not_found() {
        let e = WpdError::SegmentNotFound("BGH-GZ-001".into());
        assert!(e.to_string().contains("BGH-GZ-001"));
    }

    #[test]
    fn display_no_bands() {
        let e = WpdError::NoBands;
        assert!(!e.to_string().is_empty());
    }
}
