//! Section B — journal projection cache. Source of truth is the append-only
//! journal on the write node; a `Segment` is a derived, read-side artifact.
//! Law: journal always wins — a segment whose watermark has fallen behind
//! the journal's current offset is condemned and must be rebuilt, never
//! trusted as-is.

/// One projected segment's header. The segment body (the actual derived EAV
/// rows) is out of scope for this crate — this type only tracks whether a
/// segment is still trustworthy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    /// The journal offset this segment was built from.
    pub journal_watermark_offset: u64,
    /// Wall-clock build time (unix millis).
    pub built_at_millis: u64,
}

impl Segment {
    pub fn new(journal_watermark_offset: u64, built_at_millis: u64) -> Self {
        Segment { journal_watermark_offset, built_at_millis }
    }

    /// Journal always wins: a segment is condemned the moment the journal's
    /// current offset has moved past what this segment was built from.
    pub fn is_condemned(&self, journal_current_offset: u64) -> bool {
        journal_current_offset > self.journal_watermark_offset
    }

    /// PROVE FRESH(watermark, delta_t_max): stale-beyond-SLA is a loud
    /// refusal, never silent stale history. `now_millis` and `sla_max_age_ms`
    /// come from the caller — this type performs no clock access itself.
    pub fn is_fresh(&self, now_millis: u64, sla_max_age_ms: u64) -> bool {
        now_millis.saturating_sub(self.built_at_millis) <= sla_max_age_ms
    }
}

/// Federation (Section D): a joined result's freshness is governed by the
/// STALEST participating segment — MIN(watermark) means MAX(age). Never
/// judge a federated join by its freshest engine alone. Returns `None` for
/// an empty slice (nothing to join).
pub fn federated_staleness_ms(segments: &[Segment], now_millis: u64) -> Option<u64> {
    segments.iter().map(|s| now_millis.saturating_sub(s.built_at_millis)).max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_condemned_once_journal_advances() {
        let seg = Segment::new(100, 1_000);
        assert!(!seg.is_condemned(100));
        assert!(seg.is_condemned(101));
    }

    #[test]
    fn freshness_respects_sla_window() {
        let seg = Segment::new(100, 1_000);
        assert!(seg.is_fresh(1_500, 1_000)); // 500ms old, SLA 1000ms
        assert!(!seg.is_fresh(3_000, 1_000)); // 2000ms old, SLA 1000ms
    }

    #[test]
    fn federated_staleness_is_governed_by_the_stalest_segment() {
        let segments = [Segment::new(1, 100), Segment::new(2, 500), Segment::new(3, 900)];
        // ages at now=1000: 900, 500, 100 -- the federated freshness is the
        // WORST (oldest) age, i.e. the maximum age value.
        assert_eq!(federated_staleness_ms(&segments, 1_000), Some(900));
    }

    #[test]
    fn empty_join_has_no_watermark() {
        assert_eq!(federated_staleness_ms(&[], 1_000), None);
    }
}
