//! BatchProbe — Layer 1 anchor-set deduplication + batched projection (§8.4).

use std::collections::HashMap;

use bahyway_core::ParticleState;
use enkidb_kaki::IdentityKaki;
use idu_prober::crosstribe::{compose_n_anchors, CrossTribeLinkState};
use idu_prober::IduProbe;
use enkidb_journal::Journal;
use enkidb_snapshot::SnapshotRecord;

/// A request to probe one CrossTribe-Kaki: its ID + resolved anchor particles.
pub struct LinkProbeRequest {
    /// Opaque identifier for this link (e.g., the CrossTribe-Kaki's uuid_hash).
    pub link_id:  u32,
    /// Resolved anchor IdentityKakis from the CrossTribe-Kaki's tribes_id EAV.
    pub anchors:  Vec<IdentityKaki>,
}

/// Result for one link in the batch.
pub struct LinkResult {
    pub link_id:    u32,
    pub link_state: CrossTribeLinkState,
}

/// Summary of one batch run.
pub struct BatchResult {
    pub results:            Vec<LinkResult>,
    pub links_probed:       usize,
    pub unique_anchors:     usize,
    pub projections_run:    usize,
}

/// Batched IDU probe engine.
pub struct BatchProbe<'a> {
    journal:   &'a Journal,
    snapshots: &'a [SnapshotRecord],
}

impl<'a> BatchProbe<'a> {
    pub fn new(journal: &'a Journal, snapshots: &'a [SnapshotRecord]) -> Self {
        BatchProbe { journal, snapshots }
    }

    /// Execute a batch probe over many CrossTribe-Kakis.
    ///
    /// Layer 1: collect unique anchor KAKIs, project each exactly once.
    /// In-batch projections are never persisted — discarded after this call.
    pub fn probe_batch(&self, requests: &[LinkProbeRequest]) -> BatchResult {
        let probe = IduProbe::new(self.journal, self.snapshots);

        // Collect unique anchors across all links (Layer 1 deduplication)
        let mut unique: HashMap<[u8; 16], IdentityKaki> = HashMap::new();
        for req in requests {
            for anchor in &req.anchors {
                unique.insert(*anchor.bytes(), anchor.clone());
            }
        }
        let unique_count = unique.len();

        // Project each unique anchor exactly once
        let anchor_states: HashMap<[u8; 16], ParticleState> = unique
            .into_iter()
            .map(|(key, anchor)| {
                let state = probe.probe(&[anchor]).anchor_states.into_iter().next()
                    .unwrap_or(ParticleState::Fuzzy);
                (key, state)
            })
            .collect();

        // Compose link states from the cached anchor projections
        let results: Vec<LinkResult> = requests.iter().map(|req| {
            let states: Vec<ParticleState> = req.anchors.iter()
                .map(|a| *anchor_states.get(a.bytes()).unwrap_or(&ParticleState::Fuzzy))
                .collect();
            let link_state = compose_n_anchors(&states);
            LinkResult { link_id: req.link_id, link_state }
        }).collect();

        BatchResult {
            links_probed:    results.len(),
            results,
            unique_anchors:  unique_count,
            projections_run: unique_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_kaki::{EventKaki, KakiRole, mint::KakiMinter};
    use bahyway_core::TribeId;
    use story_engine::projection::{encode_state, ATTR_STATE};

    fn push_state(jnl: &mut Journal, m: &KakiMinter, p: &IdentityKaki, state: ParticleState) {
        let ek  = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav = vec![EavTriple::new(ATTR_STATE, encode_state(state).to_vec())];
        jnl.append(JournalEntry::new(ek, p.clone(), 1, eav)).unwrap();
    }

    #[test]
    fn shared_anchor_projected_once() {
        let m  = KakiMinter::new(TribeId::from_u16(0x0001));
        let p1 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let p2 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);
        push_state(&mut jnl, &m, &p1, ParticleState::Golden);
        push_state(&mut jnl, &m, &p2, ParticleState::Golden);

        // Three links all sharing the same two anchors
        let requests: Vec<LinkProbeRequest> = (0..3).map(|i| {
            LinkProbeRequest { link_id: i, anchors: vec![p1.clone(), p2.clone()] }
        }).collect();

        let batch = BatchProbe::new(&jnl, &[]);
        let res   = batch.probe_batch(&requests);

        assert_eq!(res.links_probed,   3);
        assert_eq!(res.unique_anchors, 2); // p1 + p2, even though used 3 times
        assert_eq!(res.projections_run, 2);
        assert!(res.results.iter().all(|r| r.link_state == CrossTribeLinkState::Gold));
    }

    #[test]
    fn batch_result_reflects_dead_anchor() {
        let m  = KakiMinter::new(TribeId::from_u16(0x0002));
        let p1 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let p2 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);
        push_state(&mut jnl, &m, &p1, ParticleState::Golden);
        push_state(&mut jnl, &m, &p2, ParticleState::Dead);

        let batch = BatchProbe::new(&jnl, &[]);
        let res   = batch.probe_batch(&[
            LinkProbeRequest { link_id: 0, anchors: vec![p1, p2] }
        ]);
        assert_eq!(res.results[0].link_state, CrossTribeLinkState::Gray);
    }
}
