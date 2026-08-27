//! SdbStore — in-memory staging buffer for particles awaiting the 15-minute
//! validation sweep.
//!
//! Data flow: SdbPipeline stages particles here → ValidationSweep promotes or
//! quarantines them.  Promoted particles go to EnkiODB; quarantined to EnkiQDB.

use enkidb_journal::entry::EavTriple;

/// Lifecycle status of a particle inside the Stage Database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdbStatus {
    /// Arrived; awaiting the next validation sweep.
    Pending,
    /// Passed validation; forwarded to EnkiODB.
    Promoted,
    /// Failed validation or Musarû late-discovery; routed to EnkiQDB.
    Quarantined,
}

/// One particle held in staging.
#[derive(Debug, Clone)]
pub struct StagedParticle {
    /// Raw 16-byte KAKI identity bytes.
    pub kaki_bytes: [u8; 16],
    /// Tribe the particle belongs to.
    pub tribe_id: u16,
    /// Epoch at time of arrival.
    pub epoch: u32,
    /// EAV triples parsed from the source row.
    pub eav: Vec<EavTriple>,
    /// Mandatory Color_ID(RGB) captured at ingest time.
    pub color_rgb: [u8; 3],
    /// Current lifecycle status.
    pub status: SdbStatus,
    /// Tick counter value when this particle arrived.
    pub arrived_tick: u64,
    /// Set true if the pre-extraction Musarû byte scan raised a flag.
    pub malware_flag: bool,
}

/// Aggregate counts across the store.
#[derive(Debug, Default, Clone)]
pub struct SdbStats {
    pub total_staged: usize,
    pub total_promoted: usize,
    pub total_quarantined: usize,
}

impl SdbStats {
    pub fn pending_count(&self) -> usize {
        self.total_staged
            .saturating_sub(self.total_promoted + self.total_quarantined)
    }
}

/// In-memory staging store.  One per database tribe.
pub struct SdbStore {
    particles: Vec<StagedParticle>,
    stats: SdbStats,
}

impl SdbStore {
    pub fn new() -> Self {
        SdbStore {
            particles: Vec::new(),
            stats: SdbStats::default(),
        }
    }

    /// Stage a new particle.  Returns its store index.
    pub fn stage(&mut self, p: StagedParticle) -> usize {
        let idx = self.particles.len();
        self.particles.push(p);
        self.stats.total_staged += 1;
        idx
    }

    /// Promote the particle at `idx` (passed validation → EnkiODB).
    /// No-op if already promoted or quarantined.
    pub fn promote(&mut self, idx: usize) {
        if let Some(p) = self.particles.get_mut(idx) {
            if p.status == SdbStatus::Pending {
                p.status = SdbStatus::Promoted;
                self.stats.total_promoted += 1;
            }
        }
    }

    /// Quarantine the particle at `idx` (failed validation → EnkiQDB).
    pub fn quarantine(&mut self, idx: usize) {
        if let Some(p) = self.particles.get_mut(idx) {
            if p.status == SdbStatus::Pending {
                p.status = SdbStatus::Quarantined;
                self.stats.total_quarantined += 1;
            }
        }
    }

    /// Iterator over (index, particle) pairs for all Pending particles.
    pub fn pending_indices(&self) -> Vec<usize> {
        self.particles
            .iter()
            .enumerate()
            .filter(|(_, p)| p.status == SdbStatus::Pending)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn get(&self, idx: usize) -> Option<&StagedParticle> {
        self.particles.get(idx)
    }
    pub fn all(&self) -> &[StagedParticle] {
        &self.particles
    }
    pub fn stats(&self) -> &SdbStats {
        &self.stats
    }
    pub fn len(&self) -> usize {
        self.particles.len()
    }
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
}

impl Default for SdbStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_particle(tick: u64, malware: bool) -> StagedParticle {
        StagedParticle {
            kaki_bytes: [0u8; 16],
            tribe_id: 1,
            epoch: 1,
            eav: Vec::new(),
            color_rgb: [128, 200, 255],
            status: SdbStatus::Pending,
            arrived_tick: tick,
            malware_flag: malware,
        }
    }

    #[test]
    fn stage_and_promote() {
        let mut store = SdbStore::new();
        let idx = store.stage(make_particle(0, false));
        assert_eq!(store.stats().total_staged, 1);
        assert_eq!(store.stats().pending_count(), 1);

        store.promote(idx);
        assert_eq!(store.stats().total_promoted, 1);
        assert_eq!(store.stats().pending_count(), 0);
        assert_eq!(store.get(idx).unwrap().status, SdbStatus::Promoted);
    }

    #[test]
    fn stage_and_quarantine() {
        let mut store = SdbStore::new();
        let idx = store.stage(make_particle(0, true));
        store.quarantine(idx);
        assert_eq!(store.stats().total_quarantined, 1);
        assert_eq!(store.get(idx).unwrap().status, SdbStatus::Quarantined);
    }

    #[test]
    fn double_promote_is_noop() {
        let mut store = SdbStore::new();
        let idx = store.stage(make_particle(0, false));
        store.promote(idx);
        store.promote(idx); // idempotent
        assert_eq!(store.stats().total_promoted, 1);
    }

    #[test]
    fn pending_indices_only_returns_pending() {
        let mut store = SdbStore::new();
        let i0 = store.stage(make_particle(0, false));
        let i1 = store.stage(make_particle(1, false));
        let i2 = store.stage(make_particle(2, false));
        store.promote(i0);
        store.quarantine(i1);
        let pending = store.pending_indices();
        assert_eq!(pending, vec![i2]);
    }
}
