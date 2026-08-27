//! BUZU — Bivector Orbit Encoding (GL-VIZ-001, §1: the encoding law).
//!
//! The billion-particle rendering condition is met not by moving
//! positions but by moving LAWS OF MOTION: an orbit is encoded by its
//! bivector B (the orbital plane element of geometric algebra, reusing
//! `bahyway_algebra::clifford::Multivector`'s already-tested Cl(7,0)
//! geometric product rather than re-deriving one). The GPU forms the
//! rotor R = exp(-B*theta/2) and evaluates a particle's position as a
//! function of time (its phase) rather than receiving a new position
//! every frame. Per-frame CPU->GPU traffic collapses to a phase scalar.
//!
//! We do not draw a billion positions; we draw orbits, and particles
//! are phases upon them. Puhu reading: the bivector is the pattern
//! (shared by every particle on the orbit); the phase is the occupant.
//!
//! HONEST SCOPE (GL-VIZ-001 open items, NOT decided here, NOT guessed):
//!   D1. Bivector residence -- per-Tribe (shared plane) vs per-particle.
//!       A memory-layout decision; unaffected by the math below.
//!   D2. BUZU chunk byte layout for GPU upload. A wire-format decision;
//!       this module proves the LAW the format will encode, not the
//!       format itself.
//!   D3. FUZZY perturbation term's packed encoding. This module exposes
//!       the additive SEMANTICS (`orbit_position_perturbed`) since that
//!       part is uncontroversial; the compact packed representation is
//!       still open.
//! These three are Architect decisions per GL-VIZ-001 -- see
//! docs/07_file_formats/GL-VIZ-001.md. Nothing here defaults them. AS OF PB-256, D1-D3
//! ARE RATIFIED and implemented in the `chunk` module below -- this
//! module's own math and its own honest scope remain unchanged.

pub mod chunk;

use bahyway_algebra::clifford::Multivector;

/// e_i basis-vector blade indices in Cl(7,0), restricted to the 3
/// spatial dimensions BUZU orbits live in (indices 3..7 are unused
/// here -- reserved should a future law need higher-dimensional orbits).
const E0: usize = 1; // 1 << 0
const E1: usize = 2; // 1 << 1
const E2: usize = 4; // 1 << 2
const E01: usize = E0 | E1; // 3
const E02: usize = E0 | E2; // 5
const E12: usize = E1 | E2; // 6

fn vector_to_mv(v: [f64; 3]) -> Multivector {
    let mut m = Multivector::zero();
    m.c[E0] = v[0];
    m.c[E1] = v[1];
    m.c[E2] = v[2];
    m
}

fn mv_to_vector(m: &Multivector) -> [f64; 3] {
    [m.c[E0], m.c[E1], m.c[E2]]
}

/// A bivector: the orbital plane element. Compact triple (b01, b02,
/// b12) -- the coefficients of e0^e1, e0^e2, e1^e2 -- which is exactly
/// the payload a GPU-resident BUZU chunk would carry per orbit (D2's
/// eventual packed form still open, but this IS its logical content).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bivector {
    pub b01: f64,
    pub b02: f64,
    pub b12: f64,
}

impl Bivector {
    /// The plane spanned by two vectors, via the wedge product --
    /// reuses `Multivector::wedge`, already tested in bahyway-algebra.
    pub fn from_plane(u: [f64; 3], v: [f64; 3]) -> Self {
        let w = vector_to_mv(u).wedge(&vector_to_mv(v));
        Bivector {
            b01: w.c[E01],
            b02: w.c[E02],
            b12: w.c[E12],
        }
    }

    /// Magnitude of the bivector (area of the spanned parallelogram).
    pub fn magnitude(&self) -> f64 {
        (self.b01 * self.b01 + self.b02 * self.b02 + self.b12 * self.b12).sqrt()
    }

    /// Unit bivector -- pure plane direction, magnitude 1.
    pub fn unit(&self) -> Self {
        let m = self.magnitude();
        if m < 1e-15 {
            *self // degenerate plane; caller error, return as-is rather than divide by zero
        } else {
            Bivector {
                b01: self.b01 / m,
                b02: self.b02 / m,
                b12: self.b12 / m,
            }
        }
    }

    /// The plane's normal vector (Hodge dual in Cl(3,0)): a vector v
    /// lies in this plane iff `dot(v, normal) == 0`. Used to verify
    /// (not assume) that orbit points stay in-plane.
    pub fn normal(&self) -> [f64; 3] {
        [self.b12, -self.b02, self.b01]
    }

    /// A deterministic unit vector lying IN this plane -- the phase-0
    /// reference direction. Needed so a BUZU chunk (D1) can store
    /// only a per-particle phase + externally-supplied radius (the
    /// radius already comes from the existing KAKI-derived scheme in
    /// `bahyway_algebra::orbital`, not duplicated here) and still
    /// reconstruct a real 3D position -- no reference vector has to be
    /// stored per particle, or even per chunk.
    ///
    /// Built by Gram-Schmidt: pick whichever global axis (e0 or e1) is
    /// least aligned with the plane's normal, then project out the
    /// normal component and normalize what's left.
    pub fn canonical_reference(&self) -> [f64; 3] {
        let n = self.normal();
        let n_len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        let n_hat = if n_len < 1e-15 {
            [0.0, 0.0, 1.0]
        } else {
            [n[0] / n_len, n[1] / n_len, n[2] / n_len]
        };

        let e0 = [1.0, 0.0, 0.0];
        let e1 = [0.0, 1.0, 0.0];
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let candidate = if dot(e0, n_hat).abs() < 0.9 { e0 } else { e1 };

        let d = dot(candidate, n_hat);
        let proj = [
            candidate[0] - d * n_hat[0],
            candidate[1] - d * n_hat[1],
            candidate[2] - d * n_hat[2],
        ];
        let plen = (proj[0] * proj[0] + proj[1] * proj[1] + proj[2] * proj[2]).sqrt();
        [proj[0] / plen, proj[1] / plen, proj[2] / plen]
    }

    /// Exponentiate the bivector into a rotor: R = cos(theta/2) -
    /// sin(theta/2)*B_hat. This IS the GL-VIZ-001 §1 law -- the orbit's
    /// law of motion, uploaded once, evaluated at any phase `theta`.
    pub fn exp(&self, theta: f64) -> Rotor {
        let b = self.unit();
        let half = theta / 2.0;
        let (s, c) = half.sin_cos();
        Rotor {
            s: c,
            b01: -s * b.b01,
            b02: -s * b.b02,
            b12: -s * b.b12,
        }
    }
}

/// A rotor: scalar + bivector (grade 0 + grade 2 of a Cl(7,0)
/// multivector). Four floats -- the same shape a GPU uniform would
/// carry per orbit-evaluation call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotor {
    pub s: f64,
    pub b01: f64,
    pub b02: f64,
    pub b12: f64,
}

impl Rotor {
    fn to_mv(self) -> Multivector {
        let mut m = Multivector::zero();
        m.c[0] = self.s;
        m.c[E01] = self.b01;
        m.c[E02] = self.b02;
        m.c[E12] = self.b12;
        m
    }

    /// Reverse (R~): reversion of a grade-0+2 element negates the
    /// grade-2 part. R * R~ = 1 for a unit rotor -- the rotor's inverse.
    pub fn reverse(self) -> Rotor {
        Rotor {
            s: self.s,
            b01: -self.b01,
            b02: -self.b02,
            b12: -self.b12,
        }
    }

    /// Rotate a vector via the sandwich product v' = R v R~, built
    /// directly from the tested `Multivector::geometric` product --
    /// not a hand-derived closed form, so no independent sign-
    /// convention risk beyond what bahyway-algebra's own tests cover.
    pub fn apply(self, v: [f64; 3]) -> [f64; 3] {
        let r = self.to_mv();
        let r_rev = self.reverse().to_mv();
        let rotated = r.geometric(&vector_to_mv(v)).geometric(&r_rev);
        mv_to_vector(&rotated)
    }
}

/// GOLDEN evaluation: pure parametric position on the orbit. No
/// per-particle storage of position -- only `reference` (a fixed
/// vector in the plane, magnitude = orbit radius) and `theta` (the
/// particle's current phase) are needed; `plane` is shared per-orbit.
pub fn orbit_position(
    nucleus: [f64; 3],
    reference: [f64; 3],
    plane: &Bivector,
    theta: f64,
) -> [f64; 3] {
    let rotated = plane.exp(theta).apply(reference);
    [
        nucleus[0] + rotated[0],
        nucleus[1] + rotated[1],
        nucleus[2] + rotated[2],
    ]
}

/// FUZZY evaluation: the additive semantics of a particle deforming
/// off its shared orbit (GL-VIZ-001 §3). The compact packed encoding
/// of `perturbation` (D3) is now sealed by `chunk::BuzuChunk` -- this
/// function itself stays the plain additive semantics D3 is built on.
pub fn orbit_position_perturbed(
    nucleus: [f64; 3],
    reference: [f64; 3],
    plane: &Bivector,
    theta: f64,
    perturbation: [f64; 3],
) -> [f64; 3] {
    let p = orbit_position(nucleus, reference, plane, theta);
    [
        p[0] + perturbation[0],
        p[1] + perturbation[1],
        p[2] + perturbation[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        dot(d, d).sqrt()
    }

    /// Sanity-checks the Hodge dual convention used by `Bivector::normal`
    /// against the three coordinate planes, before trusting it in the
    /// orbit-circle test below.
    #[test]
    fn normal_matches_coordinate_planes() {
        let xy = Bivector::from_plane([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(dist(xy.normal(), [0.0, 0.0, 1.0]) < 1e-9);

        let yz = Bivector::from_plane([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(dist(yz.normal(), [1.0, 0.0, 0.0]) < 1e-9);

        let zx = Bivector::from_plane([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert!(dist(zx.normal(), [0.0, 1.0, 0.0]) < 1e-9);
    }

    #[test]
    fn canonical_reference_is_unit_and_in_plane() {
        let planes = [
            Bivector::from_plane([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            Bivector::from_plane([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
            Bivector::from_plane([1.0, 1.0, 0.0], [0.0, 1.0, 1.0]),
            Bivector::from_plane([0.3, 1.0, -0.4], [1.0, -0.2, 0.7]),
        ];
        for plane in planes {
            let r = plane.canonical_reference();
            assert!(
                (dot(r, r).sqrt() - 1.0).abs() < 1e-9,
                "reference must be unit length"
            );
            assert!(
                dot(r, plane.normal()).abs() < 1e-9,
                "reference must lie in the plane"
            );
        }
    }

    #[test]
    fn full_turn_returns_to_start() {
        let plane = Bivector::from_plane([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let start = [3.0, 0.0, 0.0];
        let after = plane.exp(std::f64::consts::TAU).apply(start);
        assert!(dist(start, after) < 1e-9);
    }

    #[test]
    fn quarter_turn_in_xy_plane() {
        let plane = Bivector::from_plane([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let start = [1.0, 0.0, 0.0];
        let after = plane.exp(std::f64::consts::FRAC_PI_2).apply(start);
        // A quarter turn in the e0^e1 plane moves e0 onto +/-e1 depending
        // on orientation -- the direction is a convention, but landing
        // exactly on the e1 axis (not drifting off-plane) is the claim.
        assert!(after[2].abs() < 1e-9, "must stay in the xy plane");
        assert!(
            (after[0].abs()) < 1e-9,
            "must land on the e1 axis, not partway"
        );
        assert!(
            (after[1].abs() - 1.0).abs() < 1e-9,
            "must preserve radius 1"
        );
    }

    #[test]
    fn rotor_preserves_length() {
        let plane = Bivector::from_plane([0.3, 1.0, -0.4], [1.0, -0.2, 0.7]);
        let start = [2.5, -1.1, 0.6];
        let r0 = dot(start, start).sqrt();
        for i in 0..16 {
            let theta = (i as f64) * std::f64::consts::TAU / 16.0;
            let p = plane.exp(theta).apply(start);
            assert!(
                (dot(p, p).sqrt() - r0).abs() < 1e-9,
                "orbit must preserve radius at every phase (rendering's core promise)"
            );
        }
    }

    #[test]
    fn rotor_composition_matches_direct_angle() {
        let plane = Bivector::from_plane([1.0, 0.2, -0.3], [-0.1, 1.0, 0.4]);
        let start = [1.7, 0.0, -2.2];
        let theta1 = 0.4;
        let theta2 = 1.1;

        // Two half-steps, composed via the geometric product.
        let step = plane
            .exp(theta1)
            .to_mv()
            .geometric(&plane.exp(theta2).to_mv());
        let composed = Rotor {
            s: step.c[0],
            b01: step.c[E01],
            b02: step.c[E02],
            b12: step.c[E12],
        };
        let via_composition = composed.apply(start);

        // One full step at the summed angle.
        let via_direct = plane.exp(theta1 + theta2).apply(start);

        assert!(
            dist(via_composition, via_direct) < 1e-9,
            "composing two rotors must equal one rotor at the summed angle"
        );
    }

    #[test]
    fn full_orbit_traces_a_circle_in_plane() {
        let nucleus = [10.0, -5.0, 2.0];
        let plane = Bivector::from_plane([1.0, 1.0, 0.0], [0.0, 1.0, 1.0]);
        let n = plane.normal();
        // A reference vector genuinely in the plane: any linear
        // combination of the two spanning vectors qualifies.
        let reference = [1.0, 1.0, 0.0];
        let radius = dot(reference, reference).sqrt();

        for i in 0..32 {
            let theta = (i as f64) * std::f64::consts::TAU / 32.0;
            let p = orbit_position(nucleus, reference, &plane, theta);
            let offset = [p[0] - nucleus[0], p[1] - nucleus[1], p[2] - nucleus[2]];
            assert!(
                (dot(offset, offset).sqrt() - radius).abs() < 1e-9,
                "every phase must sit at orbit radius from the nucleus"
            );
            assert!(
                dot(offset, n).abs() < 1e-9,
                "every phase must lie in the declared orbital plane"
            );
        }
    }

    #[test]
    fn fuzzy_perturbation_is_additive() {
        let nucleus = [0.0, 0.0, 0.0];
        let plane = Bivector::from_plane([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let reference = [1.0, 0.0, 0.0];
        let theta = 0.9;
        let delta = [0.05, -0.02, 0.11];

        let golden = orbit_position(nucleus, reference, &plane, theta);
        let fuzzy = orbit_position_perturbed(nucleus, reference, &plane, theta, delta);

        assert!(
            dist(
                [
                    fuzzy[0] - golden[0],
                    fuzzy[1] - golden[1],
                    fuzzy[2] - golden[2]
                ],
                delta
            ) < 1e-9
        );
    }
}
