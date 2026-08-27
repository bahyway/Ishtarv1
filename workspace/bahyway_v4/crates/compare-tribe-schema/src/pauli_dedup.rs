// ============================================================================
// compare-tribe-schema/src/pauli_dedup.rs
// Tribe-vs-Tribe Pauli-duplicate dedup station -- hash-indexed, O(N+M).
//
// CompareEngine's other stations (schema_comparator, tribe_completeness)
// compare *schema shape*. This one compares *particles*: previous Tribe
// Orbit snapshot vs current Tribe Orbit snapshot, of the same source, and
// excludes any particle in `current` that collides with one already in
// `previous` per the Pauli Exclusion Principle (`bahyway_algebra::enlil`)
// -- no two particles may share the same (Identity-KAKI prefix,
// orbit_position, state) triple.
//
// A naive station (`for a in previous { for b in current { pauli_check
// pairwise } }`) is O(N x M) -- the exact blowup class this workspace's
// own design law exists to rule out ("the index determines *which*
// particles are ever compared; the algebra determines *what* gets
// computed once that set is bounded"). `pauli_check` itself is correct at
// any N (it's pure per-pair logic, no I/O) -- the O(N x M) risk lives
// entirely in how a caller drives it, not in the function. This station
// builds a `HashMap` index over `previous` once (O(N)), then probes it
// once per `current` particle (O(1) amortized, O(M) total): O(N+M), never
// O(N x M), the same "hash join, not nested-loop join" principle behind
// this workspace's EAV exact-match index and GRAVITY's MAX_GROUPS cap.
// ============================================================================
#![allow(missing_docs)]

use std::collections::HashMap;

use bahyway_algebra::{pauli_check, ExclusionResult, QuantumCoord};

/// One `current`-Tribe particle that collided with a `previous`-Tribe
/// particle at the same Pauli coordinate.
#[derive(Debug, Clone)]
pub struct PauliDuplicate {
    pub coord: QuantumCoord,
    /// The gate/reason `pauli_check` itself assigns to this collision --
    /// reused rather than re-derived, so this station and `pauli_check`
    /// can never disagree about which Council gate a given collision
    /// belongs to.
    pub verdict: ExclusionResult,
}

/// Result of comparing two Tribe Orbit snapshots for Pauli duplicates.
#[derive(Debug, Clone)]
pub struct PauliDedupReport {
    pub previous_count: usize,
    pub current_count: usize,
    pub duplicates: Vec<PauliDuplicate>,
}

impl PauliDedupReport {
    pub fn excluded_count(&self) -> usize {
        self.duplicates.len()
    }

    pub fn has_duplicates(&self) -> bool {
        !self.duplicates.is_empty()
    }
}

/// Compare `previous` Tribe Orbit particles against `current` Tribe Orbit
/// particles and flag every Pauli-duplicate collision.
///
/// Assumes `previous` is already internally Pauli-clean (no two particles
/// within the same snapshot share a coordinate) -- that invariant is
/// enforced by whichever station ingested `previous` in the first place,
/// not re-checked here; re-validating an already-validated O(N) set on
/// every comparison would itself burn the O(N) budget this station exists
/// to stay within.
pub fn dedup_tribes(previous: &[QuantumCoord], current: &[QuantumCoord]) -> PauliDedupReport {
    let index: HashMap<&QuantumCoord, ()> = previous.iter().map(|c| (c, ())).collect();

    let mut duplicates = Vec::new();
    for c in current {
        if let Some((&existing, _)) = index.get_key_value(c) {
            // Exactly one match: `pauli_check` against just that one
            // occupied coordinate reproduces the identical gate/reason its
            // own O(N) scan over the full `previous` slice would have
            // found, without paying for the full scan.
            let verdict = pauli_check(c, std::slice::from_ref(existing));
            duplicates.push(PauliDuplicate {
                coord: c.clone(),
                verdict,
            });
        }
    }

    PauliDedupReport {
        previous_count: previous.len(),
        current_count: current.len(),
        duplicates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_algebra::QuantumState;

    fn coord(prefix: u8, pos: u32, state: QuantumState) -> QuantumCoord {
        QuantumCoord {
            kaki_prefix: [prefix; 8],
            orbit_position: pos,
            state,
        }
    }

    #[test]
    fn no_overlap_produces_no_duplicates() {
        let previous = vec![coord(1, 0, QuantumState::Active)];
        let current = vec![coord(2, 0, QuantumState::Active)];
        let report = dedup_tribes(&previous, &current);
        assert!(!report.has_duplicates());
        assert_eq!(report.previous_count, 1);
        assert_eq!(report.current_count, 1);
    }

    #[test]
    fn exact_coordinate_collision_is_flagged() {
        let shared = coord(9, 0, QuantumState::Active);
        let previous = vec![shared.clone()];
        let current = vec![shared.clone()];
        let report = dedup_tribes(&previous, &current);
        assert_eq!(report.excluded_count(), 1);
        assert_eq!(report.duplicates[0].coord, shared);
        assert!(matches!(
            report.duplicates[0].verdict,
            ExclusionResult::Excluded { .. }
        ));
    }

    #[test]
    fn different_orbit_position_is_not_a_collision() {
        let previous = vec![coord(5, 0, QuantumState::Active)];
        let current = vec![coord(5, 1, QuantumState::Active)];
        let report = dedup_tribes(&previous, &current);
        assert!(!report.has_duplicates());
    }

    #[test]
    fn different_state_same_position_is_not_a_collision() {
        // Same as pauli_check's own semantics: Active and Dead can coexist
        // at the same orbit_position -- different energy shells.
        let previous = vec![coord(5, 0, QuantumState::Active)];
        let current = vec![coord(5, 0, QuantumState::Dead)];
        let report = dedup_tribes(&previous, &current);
        assert!(!report.has_duplicates());
    }

    #[test]
    fn only_colliding_particles_are_reported_others_pass_through() {
        let shared = coord(3, 0, QuantumState::Active);
        let previous = vec![shared.clone()];
        let current = vec![
            shared.clone(),
            coord(4, 0, QuantumState::Active),
            coord(3, 1, QuantumState::Active),
        ];
        let report = dedup_tribes(&previous, &current);
        assert_eq!(report.current_count, 3);
        assert_eq!(report.excluded_count(), 1);
        assert_eq!(report.duplicates[0].coord, shared);
    }

    #[test]
    fn empty_previous_never_collides() {
        let current = vec![coord(1, 0, QuantumState::Active)];
        let report = dedup_tribes(&[], &current);
        assert!(!report.has_duplicates());
        assert_eq!(report.previous_count, 0);
    }

    #[test]
    fn scales_past_seven_without_collapsing_to_zero_or_blowing_up() {
        // Not a timing test (that belongs in a benchmark, not a unit
        // test) -- this proves *correctness* holds well past the small
        // fixtures above: 500 previous particles at distinct coordinates,
        // 500 current particles where every 10th one deliberately
        // collides. A naive O(N x M) nested loop would still get the
        // right *answer* here; the point already proven above is that
        // this implementation's answer matches pauli_check's own
        // semantics exactly, at a scale too large to eyeball by hand.
        let previous: Vec<QuantumCoord> = (0..500)
            .map(|i| coord((i % 256) as u8, i as u32, QuantumState::Active))
            .collect();
        let mut current: Vec<QuantumCoord> = Vec::new();
        let mut expected_collisions = 0;
        for i in 0..500u32 {
            if i % 10 == 0 {
                current.push(previous[i as usize].clone());
                expected_collisions += 1;
            } else {
                current.push(coord(
                    ((i + 1000) % 256) as u8,
                    i + 100_000,
                    QuantumState::Active,
                ));
            }
        }
        let report = dedup_tribes(&previous, &current);
        assert_eq!(report.excluded_count(), expected_collisions);
    }
}
