//! LineageLedger — every data hop recorded, every transformation auditable.
//!
//! This is the answer to "Where does this data even come from?" (see the image).
//! Every record that passes through the Fabric carries a `LineageChain`:
//! an ordered list of `LineageHop` events, one per stage.  The chain is
//! immutable once written — appended to, never modified.

use crate::connector::SourceId;

/// Identifies a processing stage inside a pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StageId(pub &'static str);

/// The quality state of a particle at a single stage boundary.
#[derive(Debug, Clone, Copy)]
pub struct QualitySnapshot {
    /// H(P) × 240 at entry to this stage.
    pub b11_in:  u8,
    /// H(P) × 240 at exit from this stage.
    pub b11_out: u8,
}

/// One hop in a particle's journey from source to target.
#[derive(Debug, Clone)]
pub struct LineageHop {
    pub stage:        StageId,
    pub source_id:    Option<SourceId>,
    /// FNV-1a of the input record bytes — proof of what entered the stage.
    pub input_hash:   u64,
    /// FNV-1a of the output record bytes — proof of what left the stage.
    pub output_hash:  u64,
    pub quality:      QualitySnapshot,
    /// Monotonic epoch when this hop was recorded.
    pub epoch:        u32,
    /// Optional human-readable annotation (e.g. rule that fired).
    pub annotation:   Option<String>,
}

impl LineageHop {
    pub fn new(
        stage: StageId,
        source_id: Option<SourceId>,
        input_hash: u64,
        output_hash: u64,
        quality: QualitySnapshot,
        epoch: u32,
    ) -> Self {
        LineageHop { stage, source_id, input_hash, output_hash, quality, epoch, annotation: None }
    }

    pub fn with_annotation(mut self, note: impl Into<String>) -> Self {
        self.annotation = Some(note.into());
        self
    }

    /// True when this stage improved or held quality.
    pub fn quality_improved(&self) -> bool {
        self.quality.b11_out >= self.quality.b11_in
    }
}

/// The complete lineage chain for a single particle.
/// Answers: "Where did this datum come from, what touched it, and in what order?"
#[derive(Debug, Clone, Default)]
pub struct LineageChain {
    pub hops: Vec<LineageHop>,
}

impl LineageChain {
    pub fn new() -> Self { LineageChain { hops: Vec::new() } }

    /// Append a hop — always append, never modify existing hops.
    pub fn push(&mut self, hop: LineageHop) {
        self.hops.push(hop);
    }

    /// The source that originally produced this particle.
    pub fn origin(&self) -> Option<&SourceId> {
        self.hops.first().and_then(|h| h.source_id.as_ref())
    }

    /// Total number of stages this particle passed through.
    pub fn depth(&self) -> usize { self.hops.len() }

    /// Final quality score after all stages.
    pub fn final_b11(&self) -> Option<u8> {
        self.hops.last().map(|h| h.quality.b11_out)
    }

    /// True if every stage held or improved quality — a "clean pass".
    pub fn is_clean_pass(&self) -> bool {
        self.hops.iter().all(|h| h.quality_improved())
    }

    /// Produce a human-readable lineage report (shown in Dubsar).
    pub fn report(&self) -> String {
        let mut out = String::new();
        for (i, hop) in self.hops.iter().enumerate() {
            out.push_str(&format!(
                "  [{i:02}] stage={} epoch={} B11:{}->{} hash:{:x}->{:x}",
                hop.stage.0, hop.epoch,
                hop.quality.b11_in, hop.quality.b11_out,
                hop.input_hash, hop.output_hash,
            ));
            if let Some(note) = &hop.annotation {
                out.push_str(&format!(" ({note})"));
            }
            out.push('\n');
        }
        out
    }
}

// ── FNV-1a helper (no external deps) ─────────────────────────────────────────

pub fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn hash_record(record: &[(u32, Vec<u8>)]) -> u64 {
    let mut combined: Vec<u8> = Vec::new();
    for (attr, val) in record {
        combined.extend_from_slice(&attr.to_le_bytes());
        combined.extend_from_slice(val);
    }
    fnv1a_hash(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::SourceId;

    fn make_hop(stage: &'static str, b11_in: u8, b11_out: u8, epoch: u32) -> LineageHop {
        LineageHop::new(
            StageId(stage),
            Some(SourceId("erp.test")),
            0xABCD,
            0xEF01,
            QualitySnapshot { b11_in, b11_out },
            epoch,
        )
    }

    #[test]
    fn empty_chain_has_zero_depth() {
        assert_eq!(LineageChain::new().depth(), 0);
    }

    #[test]
    fn push_increments_depth() {
        let mut chain = LineageChain::new();
        chain.push(make_hop("validate", 180, 200, 1));
        chain.push(make_hop("enrich",   200, 210, 2));
        assert_eq!(chain.depth(), 2);
    }

    #[test]
    fn final_b11_is_last_hop_out() {
        let mut chain = LineageChain::new();
        chain.push(make_hop("validate", 150, 170, 1));
        chain.push(make_hop("enrich",   170, 205, 2));
        assert_eq!(chain.final_b11(), Some(205));
    }

    #[test]
    fn clean_pass_when_quality_never_drops() {
        let mut chain = LineageChain::new();
        chain.push(make_hop("a", 150, 160, 1));
        chain.push(make_hop("b", 160, 160, 2));
        assert!(chain.is_clean_pass());
    }

    #[test]
    fn not_clean_pass_when_quality_drops() {
        let mut chain = LineageChain::new();
        chain.push(make_hop("a", 160, 150, 1));
        assert!(!chain.is_clean_pass());
    }

    #[test]
    fn origin_is_first_hop_source() {
        let mut chain = LineageChain::new();
        chain.push(make_hop("extraction", 100, 120, 1));
        assert_eq!(chain.origin(), Some(&SourceId("erp.test")));
    }

    #[test]
    fn fnv1a_is_deterministic() {
        let a = fnv1a_hash(b"sovereign");
        let b = fnv1a_hash(b"sovereign");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_record_changes_with_content() {
        let r1 = vec![(0x01u32, b"alpha".to_vec())];
        let r2 = vec![(0x01u32, b"beta".to_vec())];
        assert_ne!(hash_record(&r1), hash_record(&r2));
    }

    #[test]
    fn report_contains_stage_names() {
        let mut chain = LineageChain::new();
        chain.push(make_hop("validate", 150, 180, 1));
        chain.push(make_hop("enrich", 180, 210, 2));
        let report = chain.report();
        assert!(report.contains("validate"));
        assert!(report.contains("enrich"));
    }
}
