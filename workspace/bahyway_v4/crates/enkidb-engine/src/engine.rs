//! EnkiDb — the main BahyWay database handle (§3).
//!
//! EnkiDb is the single unified API surface for all pipeline stations.
//! It owns the Journal, snapshot list, and all native indexes. All writes
//! go through `append_event`; all reads go through the StoryEngine projection.

use bahyway_core::{Result, TribeId};
use enkidb_journal::{Journal, entry::{EavTriple, JournalEntry}};
use enkidb_kaki::{EventKaki, IdentityKaki};
use enkidb_snapshot::SnapshotRecord;
use enkidb_indexes::{
    identity::IdentityIndex,
    sovereignty::SovereigntyIndex,
    typerole::TypeRoleIndex,
    temporal::TemporalIndex,
    colorid::ColorIdIndex,
    eav::EavIndex,
    snapshot_idx::SnapshotIndex,
};
use story_engine::StoryEngine;
use story_engine::projected_state::ProjectedState;

/// The database — owns Journal + all seven native indexes.
pub struct EnkiDb {
    pub(crate) tribe_id:    TribeId,
    pub(crate) journal:     Journal,
    pub(crate) snapshots:   Vec<SnapshotRecord>,
    // Native indexes (§9.3)
    pub idx_identity:    IdentityIndex,
    pub idx_sovereignty: SovereigntyIndex,
    pub idx_typerole:    TypeRoleIndex,
    pub idx_temporal:    TemporalIndex,
    pub idx_colorid:     ColorIdIndex,
    pub idx_eav:         EavIndex,
    pub idx_snapshot:    SnapshotIndex,
}

impl EnkiDb {
    /// Create a new in-memory EnkiDb for a tribe (§2.1 Sovereignty).
    pub fn new(tribe_id: TribeId) -> Self {
        EnkiDb {
            tribe_id,
            journal:     Journal::new(64),
            snapshots:   Vec::new(),
            idx_identity:    IdentityIndex::new(tribe_id),
            idx_sovereignty: SovereigntyIndex::new(),
            idx_typerole:    TypeRoleIndex::new(),
            idx_temporal:    TemporalIndex::new(),
            idx_colorid:     ColorIdIndex::new(),
            idx_eav:         EavIndex::new(),
            idx_snapshot:    SnapshotIndex::new(),
        }
    }

    /// Register an IdentityKaki (creates its slot in identity + sovereignty indexes).
    pub fn register_particle(&mut self, particle: &IdentityKaki) -> Result<()> {
        // FileOffset=0 for in-memory operation; permanent-storage sets real offsets.
        self.idx_identity.insert(particle.uuid_hash(), 0);
        self.idx_sovereignty.insert(particle.tribe_id(), particle.uuid_hash());
        self.idx_typerole.insert(
            particle.tribe_id(),
            particle.kaki_type(),
            particle.kaki_role(),
            particle.uuid_hash(),
        );
        Ok(())
    }

    /// Append an Event-Kaki with its EAV payload.
    ///
    /// Indexes are updated in-place; the Journal is the source of truth.
    pub fn append_event(
        &mut self,
        event_kaki:  EventKaki,
        target:      IdentityKaki,
        epoch:       u32,
        eav:         Vec<EavTriple>,
    ) -> Result<()> {
        // TemporalIndex uses u16 timestamp (KAKI birth-time). Use epoch low 16 bits
        // as a temporal locality hint — Journal epoch is the authoritative timestamp.
        self.idx_temporal.insert(epoch as u16, target.uuid_hash());

        // Update EAV inverted index
        for triple in &eav {
            self.idx_eav.insert(
                self.tribe_id,
                triple.attr_hash,
                triple.value.clone(),
                target.uuid_hash(),
            );
        }

        self.journal.append(JournalEntry::new(event_kaki, target, epoch, eav))?;
        Ok(())
    }

    /// Project the current state of a particle (no time bound).
    pub fn project(&self, particle: &IdentityKaki) -> ProjectedState {
        let engine = StoryEngine::new(&self.journal, &self.snapshots);
        engine.project(particle)
    }

    /// Project the state of a particle at a specific epoch (time-travel §3.4).
    pub fn project_at(&self, particle: &IdentityKaki, at_epoch: u64) -> ProjectedState {
        let engine = StoryEngine::new(&self.journal, &self.snapshots);
        engine.project_at(particle, at_epoch)
    }

    /// How many events exist for this particle.
    pub fn event_count(&self, particle: &IdentityKaki) -> usize {
        let engine = StoryEngine::new(&self.journal, &self.snapshots);
        engine.event_count(particle)
    }

    pub fn tribe_id(&self) -> TribeId { self.tribe_id }

    /// Read-only access to the Journal (for query execution).
    pub fn journal(&self) -> &Journal { &self.journal }

    /// Read-only access to the snapshot list.
    pub fn snapshots(&self) -> &[SnapshotRecord] { &self.snapshots }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{KakiRole, mint::KakiMinter};
    use bahyway_core::{ParticleState, TribeId};
    use story_engine::projection::{encode_state, ATTR_STATE};

    fn setup() -> (KakiMinter, EnkiDb) {
        let tid = TribeId::from_u16(0x0001);
        let m   = KakiMinter::new(tid);
        let db  = EnkiDb::new(tid);
        (m, db)
    }

    #[test]
    fn register_and_project_empty() {
        let (m, mut db) = setup();
        let p = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        db.register_particle(&p).unwrap();
        let ps = db.project(&p);
        assert_eq!(ps.state, ParticleState::Fuzzy);
        assert_eq!(ps.events_seen, 0);
    }

    #[test]
    fn append_event_and_project() {
        let (m, mut db) = setup();
        let p  = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        db.register_particle(&p).unwrap();
        db.append_event(
            ek, p.clone(), 1,
            vec![EavTriple::new(ATTR_STATE, encode_state(ParticleState::Golden).to_vec())],
        ).unwrap();
        let ps = db.project(&p);
        assert_eq!(ps.state, ParticleState::Golden);
        assert_eq!(ps.events_seen, 1);
    }

    #[test]
    fn time_travel_via_engine() {
        let (m, mut db) = setup();
        let p  = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let ek1 = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let ek2 = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        db.register_particle(&p).unwrap();
        db.append_event(ek1, p.clone(), 1,
            vec![EavTriple::new(ATTR_STATE, encode_state(ParticleState::Golden).to_vec())]).unwrap();
        db.append_event(ek2, p.clone(), 5,
            vec![EavTriple::new(ATTR_STATE, encode_state(ParticleState::Dead).to_vec())]).unwrap();
        assert_eq!(db.project_at(&p, 3).state, ParticleState::Golden);
        assert_eq!(db.project_at(&p, 10).state, ParticleState::Dead);
    }

    #[test]
    fn event_count_tracks_appends() {
        let (m, mut db) = setup();
        let p  = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let ek1 = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let ek2 = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        db.register_particle(&p).unwrap();
        db.append_event(ek1, p.clone(), 1, vec![]).unwrap();
        db.append_event(ek2, p.clone(), 2, vec![]).unwrap();
        assert_eq!(db.event_count(&p), 2);
    }
}
