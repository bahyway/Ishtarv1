//! ARS — Aged Recognition Service (the sovereign cron replacement).
//! Partitioning and snapshots in EnkiDW are AGED-DRIVEN, not
//! time-driven. ARS scans particles, scores "agedness" from
//! internal signals, and decides partition/snapshot actions.
//! Runs as a StoryEngine actor. TRIPLE-O only.

/// A particle's agedness signals (all derived from EAV/orbit, not clocks).
#[derive(Debug, Clone, Copy)]
pub struct AgednessSignals {
    /// Fraction of orbit that is downstream/superseded (0..1).
    pub superseded_ratio: f64,
    /// Access recency score: 1.0 = hot, 0.0 = never queried.
    pub access_heat: f64,
    /// Rotor-angle accumulation since last snapshot (radians).
    pub rotor_accum: f64,
    /// Number of newer golden versions that supersede this one.
    pub supersede_depth: u32,
}

/// The ARS decision for a particle or partition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgedDecision {
    /// Keep hot in the active partition.
    KeepActive,
    /// Mark Aged — move to aged partition on next sweep.
    MarkAged,
    /// Snapshot now (rotor accumulation crossed the band).
    SnapshotNow,
}

/// Sovereign thresholds (no wall-clock anywhere).
#[derive(Debug, Clone, Copy)]
pub struct ArsThresholds {
    pub aged_score: f64,        // agedness score above which -> MarkAged
    pub rotor_band: f64,        // rotor radians triggering a snapshot
    pub cold_access: f64,       // access_heat below which counts as cold
}

impl Default for ArsThresholds {
    fn default() -> Self {
        ArsThresholds { aged_score: 0.6, rotor_band: 1.0, cold_access: 0.2 }
    }
}

/// Compute an agedness score in [0,1] from signals.
/// Weighted: supersession dominates, then coldness, then depth.
pub fn agedness_score(s: &AgednessSignals) -> f64 {
    let coldness = 1.0 - s.access_heat;
    let depth_norm = (s.supersede_depth as f64 / 8.0).min(1.0);
    let raw = 0.45 * s.superseded_ratio
            + 0.35 * coldness
            + 0.20 * depth_norm;
    raw.clamp(0.0, 1.0)
}

/// The ARS decision function — the cron replacement's core.
pub fn decide(s: &AgednessSignals, t: &ArsThresholds) -> AgedDecision {
    // Snapshot takes priority: accumulated change must be journaled.
    if s.rotor_accum >= t.rotor_band {
        return AgedDecision::SnapshotNow;
    }
    if agedness_score(s) >= t.aged_score {
        return AgedDecision::MarkAged;
    }
    AgedDecision::KeepActive
}

/// Batch sweep: returns (index, decision) for a slice of particles.
pub fn sweep(signals: &[AgednessSignals], t: &ArsThresholds) -> Vec<(usize, AgedDecision)> {
    signals.iter().enumerate().map(|(i, s)| (i, decide(s, t))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hot() -> AgednessSignals {
        AgednessSignals { superseded_ratio: 0.0, access_heat: 0.9, rotor_accum: 0.1, supersede_depth: 0 }
    }
    fn cold_superseded() -> AgednessSignals {
        AgednessSignals { superseded_ratio: 0.8, access_heat: 0.1, rotor_accum: 0.2, supersede_depth: 4 }
    }
    fn churning() -> AgednessSignals {
        AgednessSignals { superseded_ratio: 0.1, access_heat: 0.9, rotor_accum: 1.5, supersede_depth: 0 }
    }

    #[test]
    fn hot_particle_stays_active() {
        assert_eq!(decide(&hot(), &ArsThresholds::default()), AgedDecision::KeepActive);
    }

    #[test]
    fn cold_superseded_is_marked_aged() {
        assert_eq!(decide(&cold_superseded(), &ArsThresholds::default()), AgedDecision::MarkAged);
    }

    #[test]
    fn churning_particle_triggers_snapshot() {
        assert_eq!(decide(&churning(), &ArsThresholds::default()), AgedDecision::SnapshotNow);
    }

    #[test]
    fn snapshot_takes_priority_over_aging() {
        // Even a cold+superseded particle snapshots first if rotor high.
        let mut s = cold_superseded();
        s.rotor_accum = 2.0;
        assert_eq!(decide(&s, &ArsThresholds::default()), AgedDecision::SnapshotNow);
    }

    #[test]
    fn agedness_score_is_bounded() {
        for s in [hot(), cold_superseded(), churning()] {
            let sc = agedness_score(&s);
            assert!((0.0..=1.0).contains(&sc));
        }
    }

    #[test]
    fn sweep_covers_all_particles_no_clock() {
        let batch = vec![hot(), cold_superseded(), churning()];
        let out = sweep(&batch, &ArsThresholds::default());
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].1, AgedDecision::KeepActive);
        assert_eq!(out[1].1, AgedDecision::MarkAged);
        assert_eq!(out[2].1, AgedDecision::SnapshotNow);
    }
}
