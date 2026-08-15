//! lamassu-engine — LAMASSU: the TDA (Topological Data Analysis) sentinel.
//! 𒀭𒆗 — protector at the gate, reading the shape of the Orbits.
//!
//! LamassuEngine samples a Tribe's particles into a 3D point cloud (via
//! `bahyway_algebra::orbital`, PA-14) and asks `bahyway_algebra::persistence`
//! — GeoEngine, the sole math-truth source — for the real persistent
//! homology of that cloud. It never recomputes homology itself; it only
//! samples and classifies what GeoEngine returns.
//!
//! Three signatures, sealed in the Abiogenesis / BlackHoles-in-Orbits
//! design conversation:
//!   GOLDEN — one loud, long-lived H1 bar: a genuine orbit announcing itself.
//!   FUZZY  — H1 bars exist but are short-lived: a seam, not yet a pattern.
//!   DEAD   — no persistent H1 at all: dust, no orbit.
//!
//! Vietoris-Rips triangle enumeration is O(n^3) in cloud size (§persistence
//! module doc). Callers MUST downsample a Tribe to a landmark subset before
//! calling `scan_tribe` — this crate does not do that sampling itself.
#![forbid(unsafe_code)]

use bahyway_algebra::orbital::orbital_position;
use bahyway_algebra::persistence::{vietoris_rips_persistence, PersistenceDiagram, Point3};

/// The topological verdict LamassuEngine hands back for one Tribe scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologicalSignature {
    /// One loud, long-lived H1 class — a genuine, persistent orbit.
    Golden,
    /// H1 classes exist but none persist long relative to the scan radius.
    Fuzzy,
    /// No H1 class at all — no loop, just dust (H0 components only).
    Dead,
}

impl TopologicalSignature {
    pub fn label(&self) -> &'static str {
        match self {
            TopologicalSignature::Golden => "GOLDEN",
            TopologicalSignature::Fuzzy => "FUZZY",
            TopologicalSignature::Dead => "DEAD",
        }
    }
}

/// An H1 bar whose persistence exceeds this fraction of `max_epsilon`
/// counts as GOLDEN ("the un-circle announcing itself"); below it, any
/// H1 activity at all is only FUZZY.
pub const GOLDEN_PERSISTENCE_RATIO: f64 = 0.3;

/// Classify an already-computed diagram. Pure function over GeoEngine's
/// output — this is the only "interpretation" logic LamassuEngine owns;
/// the topology itself always comes from `bahyway_algebra::persistence`.
pub fn classify(diagram: &PersistenceDiagram, max_epsilon: f64) -> TopologicalSignature {
    if diagram.h1_count() == 0 {
        return TopologicalSignature::Dead;
    }
    if max_epsilon <= 0.0 {
        return TopologicalSignature::Fuzzy;
    }
    let ratio = diagram.max_h1_persistence() / max_epsilon;
    if ratio > GOLDEN_PERSISTENCE_RATIO {
        TopologicalSignature::Golden
    } else {
        TopologicalSignature::Fuzzy
    }
}

/// One Tribe's TDA reading: the signature plus the evidence behind it.
///
/// `void_count`/`max_h2_persistence` surface GeoEngine's H2 (void) reading
/// as evidence alongside the sealed GOLDEN/FUZZY/DEAD signature — they do
/// NOT extend that three-value contract (which is sealed in the
/// Abiogenesis / BlackHoles-in-Orbits design and reads H1 only). A
/// tribe's orbital sampling (a ring/torus-like cloud, per
/// `full_ring_of_particles_is_golden`'s own comment) is not shaped to
/// reliably enclose a genuine 3D void the way a deliberately-built
/// relationship complex can (see the octahedron-shell tests in
/// `bahyway_algebra::persistence`); `void_count` is reported here so a
/// caller CAN see it, not because a Tribe scan is expected to produce one.
#[derive(Debug, Clone)]
pub struct TribeReading {
    pub tribe_id: u16,
    pub signature: TopologicalSignature,
    pub diagram: PersistenceDiagram,
    pub component_count: usize,
    pub void_count: usize,
    pub max_h2_persistence: f64,
    pub sample_size: usize,
}

/// Two already-sealed [`TribeReading`]s compared — the buildable core of
/// "compare shape signatures across a BIGRING boundary without leaking
/// raw particles." `TribeReading` never carries raw KAKI bytes (only
/// `tribe_id`, the classified signature, the persistence diagram, and
/// two counts), so comparing two of them is structurally safe for a
/// cross-tenant boundary by construction, not merely by convention.
///
/// This is deliberately just the comparison primitive, not a federation
/// layer: no BIGRING transport, tenant registry, or cross-client
/// execution engine exists anywhere in this codebase, and none is
/// invented here. Whatever eventually moves `TribeReading`s across a
/// BIGRING boundary calls this on what it receives.
///
/// Component counts and sample sizes are reported for context but are
/// NOT compared numerically here — two Tribes scanned under different
/// `LamassuEngine` parameters (different `max_epsilon`/`r_max`/`h_max`,
/// as different tenants' orbital layouts may well use) are not
/// numerically comparable on those raw counts. The classified
/// `signature` is the one thing every `LamassuEngine` config already
/// normalizes to the same three-value scale, so it is the only field
/// this comparison treats as meaningfully cross-tenant-comparable.
#[derive(Debug, Clone, Copy)]
pub struct ShapeComparison {
    pub a_tribe_id: u16,
    pub b_tribe_id: u16,
    pub a_signature: TopologicalSignature,
    pub b_signature: TopologicalSignature,
    pub same_signature: bool,
}

/// Compare two already-sealed readings — see [`ShapeComparison`]'s own
/// doc comment for what this is (and deliberately is not).
pub fn compare_readings(a: &TribeReading, b: &TribeReading) -> ShapeComparison {
    ShapeComparison {
        a_tribe_id: a.tribe_id,
        b_tribe_id: b.tribe_id,
        a_signature: a.signature,
        b_signature: b.signature,
        same_signature: a.signature == b.signature,
    }
}

/// The TDA sentinel. Holds the sampling parameters (must match whatever
/// `r_max`/`h_max` the caller's orbital ring layout uses) and the
/// Vietoris-Rips scan radius.
#[derive(Debug, Clone, Copy)]
pub struct LamassuEngine {
    pub max_epsilon: f64,
    pub r_max: f64,
    pub h_max: f64,
}

impl LamassuEngine {
    pub fn new(max_epsilon: f64, r_max: f64, h_max: f64) -> Self {
        LamassuEngine { max_epsilon, r_max, h_max }
    }

    /// Sample a landmark subset of a Tribe's particles — `(kaki_bytes,
    /// quality_delta)` pairs — into a point cloud and read GeoEngine for
    /// its persistent homology. Downsample before calling: this is O(n^3).
    pub fn scan_tribe(&self, tribe_id: u16, particles: &[([u8; 16], f64)]) -> TribeReading {
        let points: Vec<Point3> = particles
            .iter()
            .map(|(kaki, delta)| {
                orbital_position(kaki, *delta, self.r_max, self.h_max).to_cartesian()
            })
            .collect();
        let diagram = vietoris_rips_persistence(&points, self.max_epsilon);
        let signature = classify(&diagram, self.max_epsilon);
        TribeReading {
            tribe_id,
            component_count: diagram.component_count(),
            void_count: diagram.void_count(),
            max_h2_persistence: diagram.max_h2_persistence(),
            sample_size: points.len(),
            signature,
            diagram,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_algebra::persistence::PersistencePair;

    fn ring_kaki_bytes(n: usize) -> Vec<[u8; 16]> {
        // orbital::to_cartesian parametrises x = R cos²θ, y = R cosθ sinθ,
        // i.e. a circle of radius R/2 traced via angle 2θ — so θ only needs
        // to span [0, π) (byte12 ∈ [0, 128)) for 2θ to sweep a full circle
        // without any two points landing on the same spot.
        (0..n)
            .map(|i| {
                let mut k = [0u8; 16];
                k[12] = ((i * 128 / n) % 128) as u8;
                k[14] = 128; // altitude ≈ 0, keep the ring planar
                k
            })
            .collect()
    }

    #[test]
    fn empty_tribe_is_dead() {
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let reading = lamassu.scan_tribe(0x0001, &[]);
        assert_eq!(reading.signature, TopologicalSignature::Dead);
        assert_eq!(reading.sample_size, 0);
        assert_eq!(reading.void_count, 0);
    }

    #[test]
    fn full_ring_of_particles_is_golden() {
        // Same delta (fixed radius) + evenly spread azimuth byte = a real
        // orbit ring in Cartesian space (see orbital::to_cartesian, which
        // traces a genuine circle from the azimuth parametrisation).
        let kakis = ring_kaki_bytes(16);
        let particles: Vec<([u8; 16], f64)> = kakis.into_iter().map(|k| (k, 0.5)).collect();
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let reading = lamassu.scan_tribe(0x0001, &particles);
        assert_eq!(reading.signature, TopologicalSignature::Golden, "{:?}", reading.diagram);
        assert_eq!(reading.component_count, 1);
        // A planar ring is a loop (H1), not a hollow 3D shell (H2): no void.
        assert_eq!(reading.void_count, 0);
    }

    #[test]
    fn tribe_reading_surfaces_a_directly_supplied_void() {
        // scan_tribe only ever samples the orbital ring/torus shape (see
        // full_ring_of_particles_is_golden's own comment), so it cannot
        // itself be coaxed into enclosing an H2 void — the octahedron-shell
        // tests in bahyway_algebra::persistence already prove the void math
        // itself works on a real hollow-shell point cloud. What this test
        // proves instead: TribeReading's void_count/max_h2_persistence
        // fields correctly read through whatever GeoEngine's diagram says,
        // for a diagram assembled the same direct way classify_golden_*
        // and classify_fuzzy_* already do above.
        let diag = PersistenceDiagram {
            pairs: vec![
                PersistencePair { dim: 0, birth: 0.0, death: f64::INFINITY },
                PersistencePair { dim: 2, birth: 0.4, death: f64::INFINITY },
            ],
        };
        let signature = classify(&diag, 1.0);
        let reading = TribeReading {
            tribe_id: 0x0009,
            component_count: diag.component_count(),
            void_count: diag.void_count(),
            max_h2_persistence: diag.max_h2_persistence(),
            sample_size: 0,
            signature,
            diagram: diag,
        };
        assert_eq!(reading.void_count, 1);
        assert!(reading.max_h2_persistence.is_infinite());
    }

    #[test]
    fn tight_arc_segment_is_dead() {
        // Only a small arc slice of azimuth values — never closes into a
        // loop, so no H1 class should persist: DEAD, not GOLDEN or FUZZY.
        let particles: Vec<([u8; 16], f64)> = (0u8..5)
            .map(|b| {
                let mut k = [0u8; 16];
                k[12] = b; // clustered azimuth, no wraparound
                k[14] = 128;
                (k, 0.5)
            })
            .collect();
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let reading = lamassu.scan_tribe(0x0002, &particles);
        assert_eq!(reading.signature, TopologicalSignature::Dead, "{:?}", reading.diagram);
    }

    #[test]
    fn classify_golden_requires_ratio_above_threshold() {
        let diag = PersistenceDiagram {
            pairs: vec![PersistencePair { dim: 1, birth: 0.1, death: 0.9 }],
        };
        assert_eq!(classify(&diag, 1.0), TopologicalSignature::Golden);
    }

    #[test]
    fn classify_fuzzy_below_threshold() {
        let diag = PersistenceDiagram {
            pairs: vec![PersistencePair { dim: 1, birth: 0.1, death: 0.15 }],
        };
        assert_eq!(classify(&diag, 1.0), TopologicalSignature::Fuzzy);
    }

    #[test]
    fn classify_dead_with_no_h1() {
        let diag = PersistenceDiagram {
            pairs: vec![PersistencePair { dim: 0, birth: 0.0, death: f64::INFINITY }],
        };
        assert_eq!(classify(&diag, 1.0), TopologicalSignature::Dead);
    }

    #[test]
    fn compare_readings_flags_matching_golden_signatures_across_tribes() {
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let a = lamassu.scan_tribe(0x0001, &ring_kaki_bytes(16).into_iter().map(|k| (k, 0.5)).collect::<Vec<_>>());
        let b = lamassu.scan_tribe(0x0002, &ring_kaki_bytes(16).into_iter().map(|k| (k, 0.5)).collect::<Vec<_>>());
        let cmp = compare_readings(&a, &b);
        assert!(cmp.same_signature);
        assert_eq!(cmp.a_signature, TopologicalSignature::Golden);
        assert_eq!(cmp.a_tribe_id, 0x0001);
        assert_eq!(cmp.b_tribe_id, 0x0002);
    }

    #[test]
    fn compare_readings_flags_a_divergent_pair() {
        let lamassu = LamassuEngine::new(1.2, 10.0, 5.0);
        let golden = lamassu.scan_tribe(0x0001, &ring_kaki_bytes(16).into_iter().map(|k| (k, 0.5)).collect::<Vec<_>>());
        let dead = lamassu.scan_tribe(0x0002, &[]);
        let cmp = compare_readings(&golden, &dead);
        assert!(!cmp.same_signature);
        assert_eq!(cmp.b_signature, TopologicalSignature::Dead);
    }
}
