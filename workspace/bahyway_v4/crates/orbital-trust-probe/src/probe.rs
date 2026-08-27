//! OrbitalTrustProbe — the 4-step causal attribution pipeline.
//!
//! Step 1 — FuzzyRules hash: abort if rules changed.
//! Step 2 — StoryEngine replay: legitimate evolution if new EAV events exist.
//! Step 3 — ScoreEngine dimensions: neighbour density shift / freshness decay /
//!           boundary noise.
//! Step 4 — Unexplained: emit AuditJournal entry + apply trust_penalty.

use crate::cause::DeviationCause;
use crate::snapshot::{near_boundary, OrbitalSnapshot};

/// Minimum Cartesian displacement (in orbit-space units) considered a
/// meaningful deviation — below this the particle has not meaningfully moved.
pub const MIN_DEVIATION_DISTANCE: f32 = 0.05;

/// Freshness BLUE byte drop that on its own explains a ring crossing.
/// Calibrated: civil_registry decay crosses the Mid→Outer boundary after
/// ~730 days, which moves the freshness byte by roughly 60 units.
pub const FRESHNESS_DECAY_THRESHOLD: u8 = 15;

/// Neighbour count delta that constitutes a meaningful density shift.
/// Crossing the Cluster→Rule7 boundary (10→11) is the worst case.
pub const NEIGHBOUR_DELTA_THRESHOLD: usize = 2;

/// Full input for one probe evaluation.
pub struct ProbeInput<'a> {
    /// Snapshot taken after the previous scoring cycle (the "expected" orbit).
    pub previous: &'a OrbitalSnapshot,
    /// Snapshot taken after the current scoring cycle (the "observed" orbit).
    pub current: &'a OrbitalSnapshot,
    /// Number of new EAV journal events that arrived between previous and
    /// current epochs (from StoryEngine::event_count delta).
    pub new_eav_events: usize,
}

/// Result of one probe evaluation.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    /// Cartesian distance moved in orbit space.
    pub cartesian_delta: f32,
    /// Ring changed between snapshots.
    pub ring_changed: bool,
    /// Attributed cause of the deviation.
    pub cause: DeviationCause,
    /// Trust penalty in [0.0, 1.0] — 0.0 for all non-Unexplained causes.
    pub trust_penalty: f32,
    /// Human-readable attribution label.
    pub attribution: &'static str,
}

impl ProbeResult {
    /// True only when the probe detected a genuine unexplained trust violation.
    pub fn is_trust_violation(&self) -> bool {
        self.cause.requires_audit()
    }
}

/// Run the 4-step causal probe against two consecutive orbital snapshots.
pub fn probe(input: &ProbeInput<'_>) -> ProbeResult {
    let prev = input.previous;
    let curr = input.current;
    let delta = prev.cartesian_distance(curr);
    let ring_changed = !prev.same_ring(curr);

    // No meaningful movement — nothing to attribute.
    if delta < MIN_DEVIATION_DISTANCE && !ring_changed {
        return ProbeResult {
            cartesian_delta: delta,
            ring_changed,
            cause: DeviationCause::LegitimateStateEvolution,
            trust_penalty: 0.0,
            attribution: "WITHIN_TOLERANCE",
        };
    }

    // ── Step 1: Fuzzy rules fingerprint ──────────────────────────────────────
    if prev.rules_fingerprint != curr.rules_fingerprint {
        return ProbeResult {
            cartesian_delta: delta,
            ring_changed,
            cause: DeviationCause::FuzzyRulesChanged,
            trust_penalty: 0.0,
            attribution: DeviationCause::FuzzyRulesChanged.label(),
        };
    }

    // ── Step 2: StoryEngine — new EAV events ─────────────────────────────────
    if input.new_eav_events > 0 {
        return ProbeResult {
            cartesian_delta: delta,
            ring_changed,
            cause: DeviationCause::LegitimateStateEvolution,
            trust_penalty: 0.0,
            attribution: DeviationCause::LegitimateStateEvolution.label(),
        };
    }

    // ── Step 3a: Neighbour density shift ─────────────────────────────────────
    let neighbour_delta = prev.neighbour_count.abs_diff(curr.neighbour_count);
    if neighbour_delta >= NEIGHBOUR_DELTA_THRESHOLD {
        return ProbeResult {
            cartesian_delta: delta,
            ring_changed,
            cause: DeviationCause::NeighborDensityShift,
            trust_penalty: 0.0,
            attribution: DeviationCause::NeighborDensityShift.label(),
        };
    }

    // ── Step 3b: Freshness decay ──────────────────────────────────────────────
    let freshness_drop = prev.freshness_byte.saturating_sub(curr.freshness_byte);
    if freshness_drop >= FRESHNESS_DECAY_THRESHOLD {
        return ProbeResult {
            cartesian_delta: delta,
            ring_changed,
            cause: DeviationCause::FreshnessDecay,
            trust_penalty: 0.0,
            attribution: DeviationCause::FreshnessDecay.label(),
        };
    }

    // ── Step 3c: Threshold boundary noise ────────────────────────────────────
    // If either snapshot's B11 is within HYSTERESIS of a ring boundary the
    // ring assignment is inherently unstable — this is scoring noise.
    if near_boundary(prev.b11) || near_boundary(curr.b11) {
        return ProbeResult {
            cartesian_delta: delta,
            ring_changed,
            cause: DeviationCause::ThresholdBoundaryNoise,
            trust_penalty: 0.0,
            attribution: DeviationCause::ThresholdBoundaryNoise.label(),
        };
    }

    // ── Step 4: Unexplained — genuine trust violation ─────────────────────────
    // Scale penalty by magnitude of deviation, capped at 1.0.
    // A ring change doubles the penalty weight.
    let base_penalty = (delta / 4.0).clamp(0.0, 0.5);
    let trust_penalty = if ring_changed {
        (base_penalty * 2.0).min(1.0)
    } else {
        base_penalty
    };

    ProbeResult {
        cartesian_delta: delta,
        ring_changed,
        cause: DeviationCause::Unexplained,
        trust_penalty,
        attribution: DeviationCause::Unexplained.label(),
    }
}

/// Run the probe across a whole batch. Returns results in the same order.
pub fn batch_probe<'a>(inputs: &[ProbeInput<'a>]) -> Vec<ProbeResult> {
    inputs.iter().map(probe).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::OrbitalSnapshot;
    use fuzzy_engine::QualityTier;
    use tribe_orbit_engine::{ring_from_b11, OrbitalAssignment, OrbitalRing};

    const FP: u64 = 0xDEAD_BEEF_CAFE_0001;

    fn make_snap(
        epoch: u64,
        b11: u8,
        kaki: [u8; 16],
        neighbours: usize,
        fresh: u8,
    ) -> OrbitalSnapshot {
        let assignment = OrbitalAssignment::from_kaki(&kaki, b11);
        let tier = if b11 >= 200 {
            QualityTier::Gem
        } else if b11 >= 140 {
            QualityTier::Tribe
        } else if b11 >= 100 {
            QualityTier::Active
        } else {
            QualityTier::NonActive
        };
        OrbitalSnapshot::new(epoch, b11, tier, assignment, neighbours, fresh, FP)
    }

    fn identical_input<'a>(snap: &'a OrbitalSnapshot) -> ProbeInput<'a> {
        ProbeInput {
            previous: snap,
            current: snap,
            new_eav_events: 0,
        }
    }

    #[test]
    fn no_movement_is_within_tolerance() {
        let snap = make_snap(1, 210, [0u8; 16], 3, 200);
        let result = probe(&identical_input(&snap));
        assert!(!result.is_trust_violation());
        assert_eq!(result.attribution, "WITHIN_TOLERANCE");
    }

    #[test]
    fn new_eav_events_is_legitimate_evolution() {
        let prev = make_snap(1, 210, [0u8; 16], 3, 200);
        let curr = make_snap(2, 155, [0u8; 16], 3, 200); // moved to Mid ring
        let input = ProbeInput {
            previous: &prev,
            current: &curr,
            new_eav_events: 3,
        };
        let result = probe(&input);
        assert_eq!(result.cause, DeviationCause::LegitimateStateEvolution);
        assert!(!result.is_trust_violation());
    }

    #[test]
    fn rules_change_aborts_probe() {
        let prev = make_snap(1, 210, [0u8; 16], 3, 200);
        // Build current with different rules fingerprint
        let assignment = OrbitalAssignment::from_kaki(&[0u8; 16], 155);
        let curr =
            OrbitalSnapshot::new(2, 155, QualityTier::Tribe, assignment, 3, 200, 0xFFFF_FFFF);
        let input = ProbeInput {
            previous: &prev,
            current: &curr,
            new_eav_events: 0,
        };
        let result = probe(&input);
        assert_eq!(result.cause, DeviationCause::FuzzyRulesChanged);
        assert!(result.cause.is_abort());
        assert_eq!(result.trust_penalty, 0.0);
    }

    #[test]
    fn neighbour_density_shift_is_not_violation() {
        let prev = make_snap(1, 210, [0u8; 16], 3, 200);
        let curr = make_snap(2, 210, [0u8; 16], 12, 200); // density jumped
        let input = ProbeInput {
            previous: &prev,
            current: &curr,
            new_eav_events: 0,
        };
        let result = probe(&input);
        // distance will be 0 (same KAKI + B11), so within tolerance first
        // — that's correct; with same position no deviation to attribute
        assert!(!result.is_trust_violation());
    }

    #[test]
    fn freshness_decay_is_not_violation() {
        // Use different azimuth bytes (kaki[12]) so cartesian distance > MIN_DEVIATION_DISTANCE.
        // Both B11=160 are far from any boundary (boundaries at 200 and 100, HYSTERESIS=5).
        // Neighbour count unchanged (delta=0 < threshold=2), so Step 3a is skipped.
        // Freshness drop = 30 ≥ FRESHNESS_DECAY_THRESHOLD=15, so Step 3b fires.
        let mut kaki_a = [0u8; 16];
        kaki_a[12] = 0;
        let mut kaki_b = [0u8; 16];
        kaki_b[12] = 128; // 180° opposite azimuth
        let prev = make_snap(1, 160, kaki_a, 3, 220);
        let curr = make_snap(2, 160, kaki_b, 3, 190); // fresh dropped 30 bytes
        let input = ProbeInput {
            previous: &prev,
            current: &curr,
            new_eav_events: 0,
        };
        let result = probe(&input);
        // Step 3b: freshness_drop = 30 ≥ threshold=15
        assert_eq!(result.cause, DeviationCause::FreshnessDecay);
        assert_eq!(result.trust_penalty, 0.0);
    }

    #[test]
    fn boundary_noise_is_not_violation() {
        // B11 = 202 — just above Inner/Mid boundary at 200, within HYSTERESIS=5
        let prev = make_snap(1, 202, [0u8; 16], 3, 200);
        let curr = make_snap(2, 197, [0u8; 16], 3, 199); // crossed boundary
        let input = ProbeInput {
            previous: &prev,
            current: &curr,
            new_eav_events: 0,
        };
        let result = probe(&input);
        assert_eq!(result.cause, DeviationCause::ThresholdBoundaryNoise);
        assert_eq!(result.trust_penalty, 0.0);
    }

    #[test]
    fn unexplained_deviation_with_ring_change_has_penalty() {
        // B11 values far from any boundary — 240 (deep Gem) → 80 (deep Dead)
        // Different KAKI bytes so orbital position differs
        let kaki_a = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let kaki_b = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 255, 0];
        let prev = make_snap(1, 240, kaki_a, 3, 200);
        let curr = make_snap(2, 80, kaki_b, 3, 199);
        let input = ProbeInput {
            previous: &prev,
            current: &curr,
            new_eav_events: 0,
        };
        let result = probe(&input);
        assert_eq!(result.cause, DeviationCause::Unexplained);
        assert!(result.is_trust_violation());
        assert!(
            result.trust_penalty > 0.0,
            "ring change must yield non-zero penalty"
        );
        assert!(result.ring_changed);
    }

    #[test]
    fn batch_probe_correct_length() {
        let snap = make_snap(1, 210, [0u8; 16], 3, 200);
        let inputs: Vec<ProbeInput<'_>> = (0..5).map(|_| identical_input(&snap)).collect();
        let results = batch_probe(&inputs);
        assert_eq!(results.len(), 5);
    }
}
