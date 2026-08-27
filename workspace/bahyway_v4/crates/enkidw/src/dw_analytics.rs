//! DwAnalytics — analytical queries over a `PersistedDb` (§12.4 — DW layer).
//!
//! Provides OLAP-style aggregations without a separate query engine:
//!   - count particles by state (Golden / Fuzzy / Dead)
//!   - count events within an epoch range
//!   - top-N most active particles by event count
//!   - epoch range (min / max committed epoch)
//!   - shape signature: real persistent-homology TDA reading of this
//!     Tribe's particle cloud, via `lamassu-engine` (`report_with_shape`)
//!   - shape trajectory: how that reading changes across a sequence of
//!     epoch windows — EMERGING / STABLE / DISSOLVING (`shape_trajectory`)
//!
//! `ParticleState::Golden/Fuzzy/Dead` (this module's existing
//! `golden_count`/`fuzzy_count`/`dead_count`) and
//! `lamassu_engine::TopologicalSignature::Golden/Fuzzy/Dead` (the new
//! `shape_signature` field) share three words and NOTHING else — the
//! first is a per-PARTICLE evidence-lifecycle verdict (is this record
//! still valid), the second is a per-TRIBE geometric verdict over the
//! whole particle cloud's shape (does it form a persistent loop). Do not
//! read one as evidence for the other.

use bahyway_core::ParticleState;
use enkidb_persist::PersistedDb;
use lamassu_engine::{LamassuEngine, TopologicalSignature, TribeReading};

/// Per-particle statistics for the top-N report.
#[derive(Debug, Clone)]
pub struct ParticleStat {
    pub uuid_hash: u32,
    pub event_count: usize,
    pub state: ParticleState,
}

/// Full DW analytics report.
#[derive(Debug, Clone)]
pub struct DwReport {
    pub total_particles: usize,
    pub golden_count: usize,
    pub fuzzy_count: usize,
    pub dead_count: usize,
    pub total_events: usize,
    pub epoch_min: u32,
    pub epoch_max: u32,
    pub top_particles: Vec<ParticleStat>,
    /// Real persistent-homology reading of this Tribe's particle cloud —
    /// `None` unless produced via [`DwAnalytics::report_with_shape`]
    /// (`report` never populates this; it costs O(n^3) in particle
    /// count, so it is opt-in, not automatic). See this module's own
    /// doc comment for why this is NOT the same three words as
    /// `golden_count`/`fuzzy_count`/`dead_count`.
    pub shape_signature: Option<TribeReading>,
}

impl DwReport {
    /// Breakdown by state as an array for easy iteration.
    pub fn by_state(&self) -> [(ParticleState, usize); 3] {
        [
            (ParticleState::Golden, self.golden_count),
            (ParticleState::Fuzzy, self.fuzzy_count),
            (ParticleState::Dead, self.dead_count),
        ]
    }

    /// Health ratio: golden particles / total (0.0 if no particles).
    pub fn golden_ratio(&self) -> f32 {
        if self.total_particles == 0 {
            return 0.0;
        }
        self.golden_count as f32 / self.total_particles as f32
    }
}

/// Discrete three-tier proxy for LamassuEngine's `delta` (quality
/// distance δ = 1 − H(P), which determines a particle's orbital radius)
/// — NOT a continuous measured quality score. EnkiDW's mandatory EAV
/// schema doesn't yet carry one generically across every Tribe; this is
/// the honest stand-in, derived from the one per-particle lifecycle
/// signal every Tribe already has. Swap this for a real per-particle
/// continuous score the day one exists — nothing else here depends on
/// the values being exactly these three.
fn state_to_delta(state: ParticleState) -> f64 {
    match state {
        ParticleState::Golden => 0.15, // inner orbit: high quality
        ParticleState::Fuzzy => 0.50,  // mid orbit: ambiguous
        ParticleState::Dead => 0.90,   // outer orbit: decayed
    }
}

/// Ordinal for trend comparison only — Dead < Fuzzy < Golden. Not used
/// anywhere a topological verdict itself is computed; `lamassu-engine`
/// remains the sole source of the signature itself.
fn signature_rank(sig: TopologicalSignature) -> u8 {
    match sig {
        TopologicalSignature::Dead => 0,
        TopologicalSignature::Fuzzy => 1,
        TopologicalSignature::Golden => 2,
    }
}

/// Which direction a Tribe's shape reading is moving across a sequence
/// of [`ShapeTrajectory`] windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeTrend {
    /// Fewer than 2 windows produced a non-empty reading — no trend to report.
    Insufficient,
    /// The signature strengthened from the first non-empty reading to the last.
    Emerging,
    /// The signature held steady.
    Stable,
    /// The signature weakened from the first non-empty reading to the last.
    Dissolving,
}

/// One Tribe's shape reading across a sequence of epoch windows —
/// produced by [`DwAnalytics::shape_trajectory`].
#[derive(Debug, Clone)]
pub struct ShapeTrajectory {
    pub readings: Vec<((u64, u64), TribeReading)>,
}

impl ShapeTrajectory {
    /// Classify the overall direction: compares the first window with a
    /// non-empty sample against the last such window. Windows with zero
    /// particles (the Tribe didn't exist yet) are skipped rather than
    /// counted as DEAD, so a Tribe that simply hadn't been born yet
    /// doesn't read as "dissolving."
    pub fn trend(&self) -> ShapeTrend {
        let non_empty: Vec<&TribeReading> = self
            .readings
            .iter()
            .map(|(_, r)| r)
            .filter(|r| r.sample_size > 0)
            .collect();
        let (Some(first), Some(last)) = (non_empty.first(), non_empty.last()) else {
            return ShapeTrend::Insufficient;
        };
        if non_empty.len() < 2 {
            return ShapeTrend::Insufficient;
        }
        match signature_rank(last.signature).cmp(&signature_rank(first.signature)) {
            std::cmp::Ordering::Greater => ShapeTrend::Emerging,
            std::cmp::Ordering::Less => ShapeTrend::Dissolving,
            std::cmp::Ordering::Equal => ShapeTrend::Stable,
        }
    }
}

/// Analytics engine that operates on a shared `PersistedDb`.
pub struct DwAnalytics<'a> {
    pdb: &'a PersistedDb,
}

impl<'a> DwAnalytics<'a> {
    pub fn new(pdb: &'a PersistedDb) -> Self {
        DwAnalytics { pdb }
    }

    /// Generate a full analytics report.
    ///
    /// `top_n` controls how many particles appear in `top_particles`.
    pub fn report(&self, top_n: usize) -> DwReport {
        let db = self.pdb.db();
        let particles = db.journal().all_particles();
        let total_events = db.journal().entry_count();

        let mut epoch_min = u32::MAX;
        let mut epoch_max = 0u32;
        let mut golden = 0usize;
        let mut fuzzy = 0usize;
        let mut dead = 0usize;

        let mut stats: Vec<ParticleStat> = particles
            .iter()
            .map(|p| {
                let proj = db.project(p);
                let hist = db.journal().read_particle_history(p);
                let evs = hist.len();

                // Track epoch range across all particle histories
                for entry in &hist {
                    if entry.epoch < epoch_min {
                        epoch_min = entry.epoch;
                    }
                    if entry.epoch > epoch_max {
                        epoch_max = entry.epoch;
                    }
                }

                match proj.state {
                    ParticleState::Golden => golden += 1,
                    ParticleState::Fuzzy => fuzzy += 1,
                    ParticleState::Dead => dead += 1,
                }
                ParticleStat {
                    uuid_hash: p.uuid_hash(),
                    event_count: evs,
                    state: proj.state,
                }
            })
            .collect();

        if epoch_min == u32::MAX {
            epoch_min = 0;
        }

        // Sort by event_count descending, then uuid_hash for determinism
        stats.sort_by(|a, b| {
            b.event_count
                .cmp(&a.event_count)
                .then_with(|| a.uuid_hash.cmp(&b.uuid_hash))
        });
        let top_particles = stats.into_iter().take(top_n).collect();

        DwReport {
            total_particles: particles.len(),
            golden_count: golden,
            fuzzy_count: fuzzy,
            dead_count: dead,
            total_events,
            epoch_min,
            epoch_max,
            top_particles,
            shape_signature: None,
        }
    }

    /// Same as [`Self::report`], plus a real persistent-homology shape
    /// reading of this Tribe's CURRENT particle cloud (`report`'s
    /// `shape_signature` is always `None`; this is the opt-in path that
    /// actually samples and scans, since that costs O(n^3) in particle
    /// count — never done silently).
    ///
    /// `lamassu`'s `r_max`/`h_max`/`max_epsilon` must match whatever
    /// orbital ring layout this Tribe's data actually uses (same
    /// requirement `LamassuEngine` documents for every caller).
    pub fn report_with_shape(&self, top_n: usize, lamassu: &LamassuEngine) -> DwReport {
        let mut report = self.report(top_n);
        let db = self.pdb.db();
        let particles = db.journal().all_particles();
        let points: Vec<([u8; 16], f64)> = particles
            .iter()
            .map(|p| (*p.bytes(), state_to_delta(db.project(p).state)))
            .collect();
        let tribe_id = self.pdb.tribe_id().as_u16();
        report.shape_signature = Some(lamassu.scan_tribe(tribe_id, &points));
        report
    }

    /// How this Tribe's shape reading changes across a sequence of epoch
    /// windows — e.g. one window per daily partition. Each window
    /// samples every particle's state AS IT WAS at that window's end
    /// epoch (`enkidb_engine`'s time-travel projection, §3.4), so this
    /// is a genuine "shape trajectory," not just repeated present-day
    /// snapshots. See [`ShapeTrajectory::trend`] for the EMERGING/
    /// STABLE/DISSOLVING classification.
    pub fn shape_trajectory(
        &self,
        epoch_windows: &[(u64, u64)],
        lamassu: &LamassuEngine,
    ) -> ShapeTrajectory {
        let db = self.pdb.db();
        let all_particles = db.journal().all_particles();
        let tribe_id = self.pdb.tribe_id().as_u16();

        let readings: Vec<((u64, u64), TribeReading)> = epoch_windows
            .iter()
            .map(|&(from, to)| {
                // Only particles that already existed by this window's
                // end epoch (at least one event at or before `to`) --
                // a particle minted after this window is not part of
                // its shape.
                let points: Vec<([u8; 16], f64)> = all_particles
                    .iter()
                    .filter(|p| {
                        db.journal()
                            .read_particle_history(p)
                            .iter()
                            .any(|e| (e.epoch as u64) <= to)
                    })
                    .map(|p| (*p.bytes(), state_to_delta(db.project_at(p, to).state)))
                    .collect();
                ((from, to), lamassu.scan_tribe(tribe_id, &points))
            })
            .collect();

        ShapeTrajectory { readings }
    }

    /// Count events committed within `[from, to]` epoch range (inclusive).
    pub fn count_in_epoch_range(&self, from: u32, to: u32) -> usize {
        let db = self.pdb.db();
        let particles = db.journal().all_particles();
        particles
            .iter()
            .flat_map(|p| db.journal().read_particle_history(p))
            .filter(|e| e.epoch >= from && e.epoch <= to)
            .count()
    }

    /// Count particles whose current state matches `state`.
    pub fn count_by_state(&self, state: ParticleState) -> usize {
        let db = self.pdb.db();
        db.journal()
            .all_particles()
            .iter()
            .filter(|p| db.project(p).state == state)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_journal::entry::EavTriple;
    use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
    use enkidb_persist::PersistedDb;
    use enkidb_storage::FsyncPolicy;
    use std::fs;
    use std::path::PathBuf;
    use story_engine::projection::{encode_state, ATTR_STATE};

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("enkidw_anal_{}", tag));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn populated_pdb(dir: &PathBuf) -> PersistedDb {
        let tid = TribeId::from_u16(0x0001);
        let m = KakiMinter::new(tid);
        let mut pdb = PersistedDb::open(dir, tid, FsyncPolicy::Never).unwrap();
        let states = [
            (ParticleState::Golden, 1u32),
            (ParticleState::Golden, 2),
            (ParticleState::Fuzzy, 3),
            (ParticleState::Dead, 4),
        ];
        for (state, ep) in states {
            let p = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
            pdb.register_particle(&p).unwrap();
            pdb.commit(
                ek,
                p,
                ep,
                vec![EavTriple::new(ATTR_STATE, encode_state(state).to_vec())],
            )
            .unwrap();
        }
        pdb
    }

    #[test]
    fn report_counts_by_state() {
        let dir = tmp_dir("counts");
        let pdb = populated_pdb(&dir);
        let a = DwAnalytics::new(&pdb);
        let r = a.report(10);
        assert_eq!(r.total_particles, 4);
        assert_eq!(r.golden_count, 2);
        assert_eq!(r.fuzzy_count, 1);
        assert_eq!(r.dead_count, 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn report_epoch_range() {
        let dir = tmp_dir("epoch");
        let pdb = populated_pdb(&dir);
        let a = DwAnalytics::new(&pdb);
        let r = a.report(10);
        assert_eq!(r.epoch_min, 1);
        assert_eq!(r.epoch_max, 4);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn count_in_epoch_range() {
        let dir = tmp_dir("range");
        let pdb = populated_pdb(&dir);
        let a = DwAnalytics::new(&pdb);
        assert_eq!(a.count_in_epoch_range(1, 2), 2);
        assert_eq!(a.count_in_epoch_range(1, 4), 4);
        assert_eq!(a.count_in_epoch_range(5, 9), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn golden_ratio() {
        let dir = tmp_dir("ratio");
        let pdb = populated_pdb(&dir);
        let a = DwAnalytics::new(&pdb);
        let r = a.report(10);
        assert!((r.golden_ratio() - 0.5).abs() < f32::EPSILON);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn report_never_populates_shape_signature() {
        let dir = tmp_dir("no_shape");
        let pdb = populated_pdb(&dir);
        let a = DwAnalytics::new(&pdb);
        assert!(a.report(10).shape_signature.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn report_with_shape_populates_a_real_reading_sized_to_the_particle_count() {
        let dir = tmp_dir("with_shape");
        let pdb = populated_pdb(&dir);
        let a = DwAnalytics::new(&pdb);
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let r = a.report_with_shape(10, &lamassu);
        let shape = r
            .shape_signature
            .expect("report_with_shape must populate a reading");
        assert_eq!(shape.sample_size, r.total_particles);
        assert_eq!(shape.tribe_id, 0x0001);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn shape_trajectory_windows_only_see_particles_that_already_existed() {
        let dir = tmp_dir("trajectory_growth");
        let pdb = populated_pdb(&dir); // 4 particles, at epochs 1,2,3,4
        let a = DwAnalytics::new(&pdb);
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);

        let trajectory = a.shape_trajectory(&[(0, 1), (0, 2), (0, 4), (0, 100)], &lamassu);
        let sizes: Vec<usize> = trajectory
            .readings
            .iter()
            .map(|(_, r)| r.sample_size)
            .collect();
        // Monotonically non-decreasing as later windows' end-epoch admits
        // more of the 4 particles minted at epochs 1..4.
        assert_eq!(sizes, vec![1, 2, 4, 4], "sizes: {sizes:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn shape_trajectory_before_any_particle_existed_is_empty_not_dead() {
        let dir = tmp_dir("trajectory_before");
        let pdb = populated_pdb(&dir); // earliest epoch is 1
        let a = DwAnalytics::new(&pdb);
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);

        let trajectory = a.shape_trajectory(&[(0, 0)], &lamassu);
        assert_eq!(trajectory.readings[0].1.sample_size, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    // ---- ShapeTrend classification: pure logic, decoupled from a real
    // PersistedDb -- these build real TribeReadings via lamassu-engine's
    // own hand-built point clouds (same technique its own test suite
    // uses), not through KakiMinter, since controlling exact orbital
    // azimuth via a real minted KAKI isn't the point of this test.

    fn ring_points(n: usize) -> Vec<([u8; 16], f64)> {
        (0..n)
            .map(|i| {
                let mut k = [0u8; 16];
                k[12] = ((i * 128 / n) % 128) as u8;
                k[14] = 128;
                (k, 0.5)
            })
            .collect()
    }

    fn arc_points(n: u8) -> Vec<([u8; 16], f64)> {
        (0..n)
            .map(|i| {
                let mut k = [0u8; 16];
                k[12] = i;
                k[14] = 128;
                (k, 0.5)
            })
            .collect()
    }

    #[test]
    fn trend_is_insufficient_with_fewer_than_two_non_empty_readings() {
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let reading = lamassu.scan_tribe(1, &ring_points(16));
        let trajectory = ShapeTrajectory {
            readings: vec![((0, 1), reading)],
        };
        assert_eq!(trajectory.trend(), ShapeTrend::Insufficient);
    }

    #[test]
    fn trend_is_emerging_when_dead_becomes_golden() {
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let dead = lamassu.scan_tribe(1, &arc_points(5));
        let golden = lamassu.scan_tribe(1, &ring_points(16));
        assert_eq!(dead.signature, TopologicalSignature::Dead);
        assert_eq!(golden.signature, TopologicalSignature::Golden);
        let trajectory = ShapeTrajectory {
            readings: vec![((0, 1), dead), ((2, 3), golden)],
        };
        assert_eq!(trajectory.trend(), ShapeTrend::Emerging);
    }

    #[test]
    fn trend_is_dissolving_when_golden_becomes_dead() {
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let golden = lamassu.scan_tribe(1, &ring_points(16));
        let dead = lamassu.scan_tribe(1, &arc_points(5));
        let trajectory = ShapeTrajectory {
            readings: vec![((0, 1), golden), ((2, 3), dead)],
        };
        assert_eq!(trajectory.trend(), ShapeTrend::Dissolving);
    }

    #[test]
    fn trend_is_stable_when_golden_stays_golden() {
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let a = lamassu.scan_tribe(1, &ring_points(16));
        let b = lamassu.scan_tribe(1, &ring_points(16));
        let trajectory = ShapeTrajectory {
            readings: vec![((0, 1), a), ((2, 3), b)],
        };
        assert_eq!(trajectory.trend(), ShapeTrend::Stable);
    }

    #[test]
    fn trend_skips_leading_empty_windows_rather_than_reading_them_as_dead() {
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let empty = lamassu.scan_tribe(1, &[]);
        let dead = lamassu.scan_tribe(1, &arc_points(5));
        let golden = lamassu.scan_tribe(1, &ring_points(16));
        assert_eq!(empty.sample_size, 0);
        let trajectory = ShapeTrajectory {
            readings: vec![((0, 0), empty), ((1, 2), dead), ((3, 4), golden)],
        };
        // Must be Emerging (dead -> golden), not thrown off by the
        // leading zero-particle window.
        assert_eq!(trajectory.trend(), ShapeTrend::Emerging);
    }
}
