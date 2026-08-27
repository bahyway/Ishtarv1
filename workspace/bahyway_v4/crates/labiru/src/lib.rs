//! labiru — GL-LBR-001 (The Labiru Doctrine): truth has a timestamp. The
//! mortality of GOLDEN, the write-once Origin Deposit, and the Kima
//! Labirisu confrontation rite. PB-338. Pure Rust, zero dependencies.
#![forbid(unsafe_code)]

/// The Origin Deposit (§2): sealed at initial ingestion, never updated.
/// content_hash is the ingestion-time byte hash; context_hash covers
/// tribe_id, the Mandatory-EAV spine snapshot, and the tribe's
/// Salmu/betti_signature at mint.
#[derive(Debug, Clone, PartialEq)]
pub struct OriginDeposit {
    pub tribe_id: u64,
    pub content_hash: u64,
    pub context_hash: u64,
    pub sealed_at: i64,
}

/// Append-only archive of Origin Deposits. A decree may deposit a NEW
/// labiru beside the old (a re-founding); the old deposit is unerasable
/// (§2 — "never updated... not by steward, not by migration, not by
/// decree"). This type has no update/remove method at all, only append.
#[derive(Debug, Default)]
pub struct OriginArchive {
    deposits: Vec<OriginDeposit>,
}

impl OriginArchive {
    pub fn new() -> Self {
        Self::default()
    }

    /// Write-once (L28): refuses a second deposit for the same tribe
    /// unless a decree is signed, in which case the new deposit is
    /// appended BESIDE the old one -- the old is never touched.
    pub fn deposit(&mut self, deposit: OriginDeposit, decree_signed: bool) -> Result<(), &'static str> {
        let exists = self.deposits.iter().any(|d| d.tribe_id == deposit.tribe_id);
        if exists && !decree_signed {
            return Err("origin deposit is write-once; a decree must sign a re-founding");
        }
        self.deposits.push(deposit);
        Ok(())
    }

    pub fn latest(&self, tribe_id: u64) -> Option<&OriginDeposit> {
        self.deposits.iter().filter(|d| d.tribe_id == tribe_id).last()
    }

    pub fn all_for(&self, tribe_id: u64) -> Vec<&OriginDeposit> {
        self.deposits.iter().filter(|d| d.tribe_id == tribe_id).collect()
    }
}

/// Divergence D(t), decomposed along Hepta axes (§3.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Divergence {
    pub integrity: f64, // hash distance
    pub quality: f64,   // EAV diff count
    pub temporal: f64,  // staleness vs world witnesses
    pub shape: f64,     // Salmu/betti drift
}

impl Divergence {
    pub fn total(&self) -> f64 {
        self.integrity + self.quality + self.temporal + self.shape
    }
}

/// Confront the living particle against its Origin Deposit (§3, steps
/// 1-2). L31: when content and context hashes are witness-identical, D
/// is exactly zero -- not approximately zero.
pub fn confront(
    origin: &OriginDeposit,
    living_content_hash: u64,
    living_context_hash: u64,
    eav_diff_count: u32,
    staleness: f64,
    betti_drift: f64,
) -> Divergence {
    let content_matches = living_content_hash == origin.content_hash;
    let context_matches = living_context_hash == origin.context_hash;
    if content_matches && context_matches {
        return Divergence { integrity: 0.0, quality: 0.0, temporal: 0.0, shape: 0.0 };
    }
    // The Integrity axis is hash distance -- both the content witness AND
    // the context witness are hash witnesses (§2). A context-hash mismatch
    // must register here even when the caller's independently-measured
    // quality/temporal/shape diffs happen to read zero: that mismatch with
    // zero measured cause is exactly the record-drift the doctrine names
    // in §1 -- "the record itself moves while claiming stillness" -- and
    // must never silently round to concord.
    Divergence {
        integrity: if content_matches && context_matches { 0.0 } else { 1.0 },
        quality: eav_diff_count as f64,
        temporal: staleness,
        shape: betti_drift,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Concord,
    LawfulEvolution,
    SilentDrift,
}

/// Does NARU account for every step of the divergence? (§3, step 2)
pub fn journal_fully_covers(steps: &[bool]) -> bool {
    !steps.is_empty() && steps.iter().all(|&explained| explained)
}

/// Route the verdict (§3, step 3). A SILENT DRIFT is a RIGMU by
/// definition (GL-NSR-001-A1 §3) -- the caller opens the escalation;
/// this function only names the finding.
pub fn route_verdict(divergence: &Divergence, eps: f64, journal_steps: &[bool]) -> Verdict {
    if divergence.total() <= eps {
        Verdict::Concord
    } else if journal_fully_covers(journal_steps) {
        Verdict::LawfulEvolution
    } else {
        Verdict::SilentDrift
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin() -> OriginDeposit {
        OriginDeposit { tribe_id: 42, content_hash: 0xABCD, context_hash: 0x1234, sealed_at: 1000 }
    }

    // L28 — the deposit is write-once (second write refused, decree
    // deposits beside, never over).
    #[test]
    fn l28_write_once_deposit() {
        let mut archive = OriginArchive::new();
        archive.deposit(origin(), false).unwrap();

        let refounding = OriginDeposit { content_hash: 0xFFFF, ..origin() };
        assert!(
            archive.deposit(refounding.clone(), false).is_err(),
            "un-decreed second deposit must be refused"
        );
        assert_eq!(archive.all_for(42).len(), 1);

        archive.deposit(refounding, true).unwrap();
        let both = archive.all_for(42);
        assert_eq!(both.len(), 2, "decree deposits BESIDE the old, never over it");
        assert_eq!(both[0].content_hash, 0xABCD, "old deposit is unerasable");
        assert_eq!(both[1].content_hash, 0xFFFF);
    }

    // L29 — fully journaled divergence -> LAWFUL EVOLUTION.
    #[test]
    fn l29_fully_journaled_is_lawful_evolution() {
        let d = confront(&origin(), 0xABCD, 0x9999, 3, 0.5, 0.1);
        assert!(d.total() > 0.0);
        let journal = [true, true, true];
        assert_eq!(route_verdict(&d, 0.01, &journal), Verdict::LawfulEvolution);
    }

    // L30 — one journal gap -> SILENT DRIFT -> Rigmu opened.
    #[test]
    fn l30_one_gap_is_silent_drift() {
        let d = confront(&origin(), 0xABCD, 0x9999, 3, 0.5, 0.1);
        let journal = [true, false, true]; // one gap
        assert_eq!(route_verdict(&d, 0.01, &journal), Verdict::SilentDrift);
    }

    // L31 — D = 0 exactly iff content and context are witness-identical.
    #[test]
    fn l31_concord_is_exact() {
        let exact = confront(&origin(), 0xABCD, 0x1234, 0, 0.0, 0.0);
        assert_eq!(exact.total(), 0.0);

        // Even with (implausibly) nonzero measured diffs passed in, exact
        // witness identity forces D to exactly zero -- concord is never
        // approximate.
        let exact_despite_noise = confront(&origin(), 0xABCD, 0x1234, 7, 4.0, 2.0);
        assert_eq!(exact_despite_noise.total(), 0.0);

        let near_miss = confront(&origin(), 0xABCD, 0x9999, 0, 0.0, 0.0);
        assert!(near_miss.total() > 0.0, "context mismatch must not silently round to concord");
    }
}
