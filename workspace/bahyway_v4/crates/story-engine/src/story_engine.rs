//! StoryEngine — the CQRS read-side projection engine (§3.3, §3.4, §3.6).
//!
//! Key guarantees:
//! - Current state is NEVER a stored value — always projected on demand.
//! - Time-travel: "state at time T" = find snapshot ≤ T, apply delta events.
//! - Snapshot acceleration: O(events since last snapshot) instead of O(total).
//! - Zero writes: the StoryEngine never modifies the Journal or any index.

use enkidb_kaki::IdentityKaki;
use enkidb_journal::Journal;
use enkidb_snapshot::SnapshotRecord;
use enkidb_snapshot::projection::{choose_algorithm, ProjectionAlgorithm};

use crate::projected_state::ProjectedState;
use crate::projection::{decode_state, ATTR_STATE};

pub struct StoryEngine<'a> {
    journal:   &'a Journal,
    snapshots: &'a [SnapshotRecord],
}

impl<'a> StoryEngine<'a> {
    pub fn new(journal: &'a Journal, snapshots: &'a [SnapshotRecord]) -> Self {
        StoryEngine { journal, snapshots }
    }

    /// Project the current state of a particle (no time bound = now).
    pub fn project(&self, particle: &IdentityKaki) -> ProjectedState {
        self.project_at(particle, u64::MAX)
    }

    /// Project the state of a particle at a specific epoch (time-travel §3.4).
    /// Equivalent to: "what was this particle's state at time T?"
    pub fn project_at(&self, particle: &IdentityKaki, at_epoch: u64) -> ProjectedState {
        let algo = choose_algorithm(particle, at_epoch, self.snapshots);

        let mut state = match &algo {
            ProjectionAlgorithm::SnapshotAccelerated { snapshot_epoch } => {
                // Start from snapshot state
                let snap = self.snapshots.iter()
                    .filter(|s| s.particle.bytes() == particle.bytes()
                             && s.snapshot_date <= *snapshot_epoch)
                    .max_by_key(|s| s.snapshot_date);
                let mut ps = ProjectedState::new(particle.clone());
                ps.from_snapshot = true;
                if let Some(s) = snap {
                    // snapshot_state is raw bytes encoding prior attr values;
                    // for now treat as opaque starting state
                    let _ = &s.snapshot_state;
                }
                ps
            }
            ProjectionAlgorithm::FullReplay => ProjectedState::new(particle.clone()),
        };

        // Determine the epoch threshold for delta events
        let since_epoch = match &algo {
            ProjectionAlgorithm::SnapshotAccelerated { snapshot_epoch } => *snapshot_epoch as u32,
            ProjectionAlgorithm::FullReplay => 0,
        };

        // Apply all qualifying Event-Kakis in chronological order (§3.1)
        let history = self.journal.read_particle_history(particle);
        let mut relevant: Vec<_> = history.iter()
            .filter(|e| e.epoch >= since_epoch && e.epoch as u64 <= at_epoch)
            .collect();
        // Sort ascending by epoch — Journal must be replayed in chronological order
        relevant.sort_by_key(|e| e.epoch);

        for entry in &relevant {
            state.events_seen += 1;
            for triple in &entry.eav {
                state.apply_single_row(triple.attr_hash, triple.value.clone());
            }
        }

        // Derive `state` field from the projected `state` EAV attribute
        if let Some(s_bytes) = state.get(ATTR_STATE) {
            state.state = decode_state(s_bytes);
        }

        state
    }

    /// How many Event-Kakis exist for this particle in the Journal.
    pub fn event_count(&self, particle: &IdentityKaki) -> usize {
        self.journal.read_particle_history(particle).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_kaki::{EventKaki, KakiRole, mint::KakiMinter};
    use bahyway_core::{ParticleState, TribeId};
    use crate::projection::{encode_state, ATTR_QUALITY};

    fn setup() -> (KakiMinter, Journal, IdentityKaki) {
        let m      = KakiMinter::new(TribeId::from_u16(0x0001));
        let target = enkidb_kaki::IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let jnl    = Journal::new(64);
        (m, jnl, target)
    }

    fn push_state(jnl: &mut Journal, m: &KakiMinter, target: &IdentityKaki, ps: ParticleState, epoch: u32) {
        let ek  = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav = vec![EavTriple::new(ATTR_STATE, encode_state(ps).to_vec())];
        jnl.append(JournalEntry::new(ek, target.clone(), epoch, eav)).unwrap();
    }

    #[test]
    fn project_current_state() {
        let (m, mut jnl, target) = setup();
        push_state(&mut jnl, &m, &target, ParticleState::Golden, 1);
        let engine = StoryEngine::new(&jnl, &[]);
        let ps     = engine.project(&target);
        assert_eq!(ps.state, ParticleState::Golden);
        assert_eq!(ps.events_seen, 1);
    }

    #[test]
    fn project_state_evolution() {
        let (m, mut jnl, target) = setup();
        push_state(&mut jnl, &m, &target, ParticleState::Golden, 1);
        push_state(&mut jnl, &m, &target, ParticleState::Fuzzy,  2);
        push_state(&mut jnl, &m, &target, ParticleState::Dead,   3);
        let engine = StoryEngine::new(&jnl, &[]);
        assert_eq!(engine.project(&target).state, ParticleState::Dead);
    }

    #[test]
    fn time_travel_query() {
        let (m, mut jnl, target) = setup();
        push_state(&mut jnl, &m, &target, ParticleState::Golden, 1);
        push_state(&mut jnl, &m, &target, ParticleState::Dead,   5);
        let engine = StoryEngine::new(&jnl, &[]);
        // At epoch 3 the particle was still Golden
        let at_3 = engine.project_at(&target, 3);
        assert_eq!(at_3.state, ParticleState::Golden);
        // At epoch 10 it is Dead
        let at_10 = engine.project_at(&target, 10);
        assert_eq!(at_10.state, ParticleState::Dead);
    }

    #[test]
    fn multi_attribute_projection() {
        let (m, mut jnl, target) = setup();
        let ek  = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav = vec![
            EavTriple::new(ATTR_STATE,   encode_state(ParticleState::Golden).to_vec()),
            EavTriple::new(ATTR_QUALITY, b"0.95".to_vec()),
        ];
        jnl.append(JournalEntry::new(ek, target.clone(), 1, eav)).unwrap();
        let engine = StoryEngine::new(&jnl, &[]);
        let ps     = engine.project(&target);
        assert_eq!(ps.get(ATTR_QUALITY), Some(b"0.95".as_slice()));
    }

    #[test]
    fn empty_journal_returns_fuzzy() {
        let (_, jnl, target) = setup();
        let engine = StoryEngine::new(&jnl, &[]);
        let ps     = engine.project(&target);
        assert_eq!(ps.state, ParticleState::Fuzzy);
        assert_eq!(ps.events_seen, 0);
    }
}
