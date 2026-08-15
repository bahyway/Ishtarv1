//! DiagnosisEngine — watches color drift across all particles in the Journal
//! and emits diagnosis events when drift crosses sovereign thresholds.
//!
//! Per-particle decision:
//!   drift > critical_threshold → DiagnosisCritical  (EventCause 0x42)
//!   drift > warning_threshold  → DiagnosisWarning   (EventCause 0x41)
//!   drift > watch_threshold    → DiagnosisWatch     (EventCause 0x40)
//!   otherwise                  → no event
//!
//! The engine emits at most one event per particle per run (the highest
//! severity wins).

use std::collections::HashMap;

use bahyway_core::TribeId;
use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
use enkidb_journal::{Journal, JournalEntry, EventCause};
use crate::color_drift::{rgb_drift, drift_axis, DriftAxis};

/// Sovereign drift thresholds (Euclidean distance in RGB space).
#[derive(Debug, Clone)]
pub struct DriftThresholds {
    /// Drift at or above this level → DiagnosisWatch.
    pub watch:    f32,
    /// Drift at or above this level → DiagnosisWarning.
    pub warning:  f32,
    /// Drift at or above this level → DiagnosisCritical.
    pub critical: f32,
}

impl Default for DriftThresholds {
    fn default() -> Self {
        DriftThresholds { watch: 30.0, warning: 60.0, critical: 100.0 }
    }
}

/// Result of one diagnosis run.
#[derive(Debug, Default, Clone)]
pub struct DiagnosisResult {
    pub particles_examined: usize,
    pub watch_events:       usize,
    pub warning_events:     usize,
    pub critical_events:    usize,
    /// Particles with no color snapshot in the Journal (cannot diagnose).
    pub no_snapshot_count:  usize,
}

impl DiagnosisResult {
    pub fn total_events(&self) -> usize {
        self.watch_events + self.warning_events + self.critical_events
    }
}

/// The active diagnosis engine.  Registered tribe root colors define the
/// baseline against which particle color drift is measured.
pub struct DiagnosisEngine {
    /// Tribe root RGB colors: tribe_id → [R, G, B].
    tribe_root_colors: HashMap<u16, [u8; 3]>,
    pub thresholds:    DriftThresholds,
}

impl DiagnosisEngine {
    pub fn new() -> Self {
        DiagnosisEngine {
            tribe_root_colors: HashMap::new(),
            thresholds: DriftThresholds::default(),
        }
    }

    /// Register or update the root color for a tribe.
    pub fn set_tribe_root_color(&mut self, tribe_id: u16, rgb: [u8; 3]) {
        self.tribe_root_colors.insert(tribe_id, rgb);
    }

    /// Return the root color for a tribe (white = [255,255,255] if unknown).
    pub fn tribe_root_color(&self, tribe_id: u16) -> [u8; 3] {
        self.tribe_root_colors.get(&tribe_id).copied().unwrap_or([255, 255, 255])
    }

    /// Examine all particles in the journal.  For each one, read the most
    /// recent `color_id_snapshot` EAV, compute drift vs tribe root color, and
    /// emit an EventCause-annotated JournalEntry if a threshold is crossed.
    ///
    /// `minter` must belong to the same tribe as the journal.
    pub fn run(
        &mut self,
        journal:      &mut Journal,
        minter:       &KakiMinter,
        tribe_id:     TribeId,
        current_tick: u64,
    ) -> DiagnosisResult {
        let _ = current_tick;
        let root_color = self.tribe_root_color(tribe_id.as_u16());

        // Collect work without borrowing journal mutably
        struct Work {
            target:  IdentityKaki,
            snap:    [u8; 3],
            epoch:   u32,
        }

        let mut work_items: Vec<Work> = Vec::new();

        let particles = journal.all_particles();
        let mut result = DiagnosisResult { particles_examined: particles.len(), ..Default::default() };

        for target in &particles {
            let history = journal.read_particle_history(target);
            // Most recent color snapshot = last entry in history with ATTR_COLOR_ID_SNAPSHOT
            let snap_opt: Option<([u8; 3], u32)> = history.iter().rev().find_map(|e| {
                e.color_id_snapshot().map(|s| (s, e.epoch))
            });
            match snap_opt {
                None => { result.no_snapshot_count += 1; }
                Some((snap, epoch)) => {
                    work_items.push(Work { target: target.clone(), snap, epoch });
                }
            }
        }

        // Now mutably write new events
        for w in work_items {
            let drift = rgb_drift(w.snap, root_color);
            let cause = self.classify_drift(drift);
            if let Some(cause) = cause {
                match cause {
                    EventCause::DiagnosisWatch    => result.watch_events    += 1,
                    EventCause::DiagnosisWarning  => result.warning_events  += 1,
                    EventCause::DiagnosisCritical => result.critical_events += 1,
                    _ => {}
                }
                let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
                    .expect("minter always produces valid event KAKIs");

                // Encode drift and axis into the color snapshot for HeptaScript queries.
                let axis  = drift_axis(w.snap, root_color);
                let drift_rgb = axis_to_rgb(axis, drift);

                let entry = JournalEntry::new(event_kaki, w.target, w.epoch, Vec::new())
                    .with_color_snapshot(drift_rgb, drift)
                    .with_event_cause(cause);
                let _ = journal.append(entry);
            }
        }

        result
    }

    fn classify_drift(&self, drift: f32) -> Option<EventCause> {
        if      drift >= self.thresholds.critical { Some(EventCause::DiagnosisCritical) }
        else if drift >= self.thresholds.warning  { Some(EventCause::DiagnosisWarning)  }
        else if drift >= self.thresholds.watch    { Some(EventCause::DiagnosisWatch)    }
        else                                      { None }
    }
}

impl Default for DiagnosisEngine {
    fn default() -> Self { Self::new() }
}

/// Encode a DriftAxis into a diagnostic RGB snapshot stored in the Journal.
/// This lets HeptaScript filter by axis without re-computing drift.
fn axis_to_rgb(axis: DriftAxis, drift: f32) -> [u8; 3] {
    let level = drift.min(255.0) as u8;
    match axis {
        DriftAxis::Severity  => [level, 0, 0],  // R dominant → alert
        DriftAxis::Quality   => [0, level, 0],  // G dominant → quality
        DriftAxis::Freshness => [0, 0, level],  // B dominant → freshness
        DriftAxis::Balanced  => [level, level, level],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::{KakiMinter, KakiRole, EventKaki, IdentityKaki};
    use enkidb_journal::{Journal, JournalEntry, EventCause};

    fn setup(tid: TribeId) -> (DiagnosisEngine, KakiMinter, Journal) {
        let mut eng = DiagnosisEngine::new();
        eng.set_tribe_root_color(tid.as_u16(), [128, 200, 255]);
        let minter = KakiMinter::new(tid);
        let journal = Journal::new(64);
        (eng, minter, journal)
    }

    fn seed_particle_with_color(
        minter: &KakiMinter, journal: &mut Journal, rgb: [u8; 3], cause: EventCause,
    ) -> IdentityKaki {
        let target = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let ek     = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
        let entry  = JournalEntry::new(ek, target.clone(), 1, Vec::new())
            .with_color_snapshot(rgb, 0.0)
            .with_event_cause(cause);
        journal.append(entry).unwrap();
        target
    }

    #[test]
    fn no_events_for_particle_within_watch_threshold() {
        let tid = TribeId::from_u16(0x0001);
        let (mut eng, minter, mut jnl) = setup(tid);
        // Root = [128,200,255]; snapshot very close → drift << 30
        seed_particle_with_color(&minter, &mut jnl, [130, 202, 253], EventCause::KakiBorn);

        let initial_count = jnl.entry_count();
        let res = eng.run(&mut jnl, &minter, tid, 0);

        assert_eq!(res.total_events(), 0);
        assert_eq!(jnl.entry_count(), initial_count); // no new events written
    }

    #[test]
    fn watch_event_emitted_for_mild_drift() {
        let tid = TribeId::from_u16(0x0001);
        let (mut eng, minter, mut jnl) = setup(tid);
        // Root=[128,200,255]; snapshot drifted ~50 units (√(50²) along R)
        seed_particle_with_color(&minter, &mut jnl, [178, 200, 255], EventCause::KakiBorn);

        let res = eng.run(&mut jnl, &minter, tid, 0);

        assert_eq!(res.watch_events, 1);
        assert_eq!(res.warning_events, 0);
        assert_eq!(res.critical_events, 0);
    }

    #[test]
    fn warning_event_emitted() {
        let tid = TribeId::from_u16(0x0001);
        let (mut eng, minter, mut jnl) = setup(tid);
        // drift ~100 units along R channel → warning (60 ≤ drift < 100)
        seed_particle_with_color(&minter, &mut jnl, [200, 200, 255], EventCause::KakiBorn);

        let res = eng.run(&mut jnl, &minter, tid, 0);

        assert!(res.warning_events > 0 || res.critical_events > 0,
            "expected at least warning or critical for large drift");
    }

    #[test]
    fn critical_event_emitted() {
        let tid = TribeId::from_u16(0x0001);
        let (mut eng, minter, mut jnl) = setup(tid);
        // Root=[128,200,255]; snapshot completely opposite → large drift
        seed_particle_with_color(&minter, &mut jnl, [0, 0, 0], EventCause::KakiBorn);

        let res = eng.run(&mut jnl, &minter, tid, 0);

        assert_eq!(res.critical_events, 1);
    }

    #[test]
    fn particle_without_snapshot_counted_but_no_event() {
        let tid = TribeId::from_u16(0x0001);
        let (mut eng, minter, mut jnl) = setup(tid);

        // Particle with NO color_id_snapshot EAV
        let target = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let ek     = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
        let entry  = JournalEntry::new(ek, target, 1, Vec::new())
            .with_event_cause(EventCause::KakiBorn); // no color snapshot
        jnl.append(entry).unwrap();

        let res = eng.run(&mut jnl, &minter, tid, 0);

        assert_eq!(res.particles_examined, 1);
        assert_eq!(res.no_snapshot_count,  1);
        assert_eq!(res.total_events(),     0);
    }

    #[test]
    fn multiple_particles_each_diagnosed_independently() {
        let tid = TribeId::from_u16(0x0001);
        let (mut eng, minter, mut jnl) = setup(tid);

        // One healthy, one critical
        seed_particle_with_color(&minter, &mut jnl, [130, 202, 253], EventCause::KakiBorn);
        seed_particle_with_color(&minter, &mut jnl, [0, 0, 0],       EventCause::KakiBorn);

        let res = eng.run(&mut jnl, &minter, tid, 0);

        assert_eq!(res.particles_examined, 2);
        assert_eq!(res.critical_events, 1);
        assert_eq!(res.total_events(),   1);
    }
}
