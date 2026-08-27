//! IduProbe — the single-link IDU probe operation (§8.3).
//!
//! Shamash (the indexer) calls `IduProbe::probe()` when rendering a CrossTribe-Kaki.
//! The probe is read-only: zero writes occur, nothing is cached beyond the call.

#[cfg(test)]
use bahyway_core::LinkState;
use bahyway_core::ParticleState;
use enkidb_journal::Journal;
use enkidb_kaki::IdentityKaki;
use enkidb_snapshot::SnapshotRecord;
use story_engine::StoryEngine;

use crate::crosstribe::{compose_n_anchors, CrossTribeLinkState};

/// Result of one IDU probe.
pub struct IduProbeResult {
    pub link_state: CrossTribeLinkState,
    pub anchor_states: Vec<ParticleState>,
    /// Number of StoryEngine projections executed.
    pub projections_run: usize,
}

/// Performs the IDU Probe for a CrossTribe-Kaki.
pub struct IduProbe<'a> {
    journal: &'a Journal,
    snapshots: &'a [SnapshotRecord],
}

impl<'a> IduProbe<'a> {
    pub fn new(journal: &'a Journal, snapshots: &'a [SnapshotRecord]) -> Self {
        IduProbe { journal, snapshots }
    }

    /// Probe the current effective state of a link given its anchor particles.
    /// `anchors` = the resolved IdentityKakis from the CrossTribe-Kaki's tribes_id EAV.
    pub fn probe(&self, anchors: &[IdentityKaki]) -> IduProbeResult {
        let engine = StoryEngine::new(self.journal, self.snapshots);
        let anchor_states: Vec<_> = anchors.iter().map(|a| engine.project(a).state).collect();
        let projections_run = anchor_states.len();
        let link_state = compose_n_anchors(&anchor_states);
        IduProbeResult {
            link_state,
            anchor_states,
            projections_run,
        }
    }

    /// Time-travel probe: effective state at a specific epoch (§8.3 time-travel).
    pub fn probe_at(&self, anchors: &[IdentityKaki], at_epoch: u64) -> IduProbeResult {
        let engine = StoryEngine::new(self.journal, self.snapshots);
        let anchor_states: Vec<_> = anchors
            .iter()
            .map(|a| engine.project_at(a, at_epoch).state)
            .collect();
        let projections_run = anchor_states.len();
        let link_state = compose_n_anchors(&anchor_states);
        IduProbeResult {
            link_state,
            anchor_states,
            projections_run,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_kaki::{mint::KakiMinter, EventKaki, KakiRole};
    use story_engine::projection::{encode_state, ATTR_STATE};

    fn push_event(
        jnl: &mut Journal,
        m: &KakiMinter,
        target: &IdentityKaki,
        state: ParticleState,
        epoch: u32,
    ) {
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav = vec![EavTriple::new(ATTR_STATE, encode_state(state).to_vec())];
        jnl.append(JournalEntry::new(ek, target.clone(), epoch, eav))
            .unwrap();
    }

    #[test]
    fn golden_golden_gives_gold() {
        let m1 = KakiMinter::new(TribeId::from_u16(0x0001));
        let m2 = KakiMinter::new(TribeId::from_u16(0x0002));
        let p1 = IdentityKaki::try_from_kaki(m1.identity(KakiRole::Zikru)).unwrap();
        let p2 = IdentityKaki::try_from_kaki(m2.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);
        push_event(&mut jnl, &m1, &p1, ParticleState::Golden, 1);
        push_event(&mut jnl, &m2, &p2, ParticleState::Golden, 1);
        let probe = IduProbe::new(&jnl, &[]);
        let res = probe.probe(&[p1, p2]);
        assert_eq!(res.link_state, LinkState::Gold);
        assert_eq!(res.projections_run, 2);
    }

    #[test]
    fn dead_anchor_gives_gray() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let p1 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let p2 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);
        push_event(&mut jnl, &m, &p1, ParticleState::Golden, 1);
        push_event(&mut jnl, &m, &p2, ParticleState::Dead, 1);
        let probe = IduProbe::new(&jnl, &[]);
        assert_eq!(probe.probe(&[p1, p2]).link_state, LinkState::Gray);
    }

    #[test]
    fn time_travel_probe() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let p1 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let p2 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);
        push_event(&mut jnl, &m, &p1, ParticleState::Golden, 1);
        push_event(&mut jnl, &m, &p2, ParticleState::Golden, 1);
        push_event(&mut jnl, &m, &p2, ParticleState::Dead, 5);
        let probe = IduProbe::new(&jnl, &[]);
        // At epoch 3: both Golden → Gold
        assert_eq!(
            probe.probe_at(&[p1.clone(), p2.clone()], 3).link_state,
            LinkState::Gold
        );
        // At epoch 10: p2 is Dead → Gray
        assert_eq!(probe.probe_at(&[p1, p2], 10).link_state, LinkState::Gray);
    }
}
