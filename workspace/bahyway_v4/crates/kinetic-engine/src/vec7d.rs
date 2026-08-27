//! # Vec7D — The Sovereign 7-Dimensional Vector
//!
//! The fundamental mathematical type of BahyWay.Ecosystem.
//! Seven named dimensions corresponding exactly to the sealed
//! DMW Heptagon analysis levels (116 tests, v1.1.0).
//!
//! Each dimension is a coordinate axis in the sovereign analytical space:
//!
//! | Field | Sumerian | Analytical Meaning              |
//! |-------|----------|---------------------------------|
//! | me    | ME 𒈨    | Intent — Set Theory             |
//! | gu    | GU 𒄖    | Mass — Cardinality              |
//! | sag   | SAG 𒊕   | Gravity — Selectivity           |
//! | a     | A 𒀀     | Viscosity — IO Friction         |
//! | izi   | IZI 𒉈   | Heat — CPU Burn                 |
//! | ud    | UD 𒌓    | Volatility — Cache / Plan       |
//! | uru   | URU 𒌷   | Strength — Index Health         |

use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A vector in the sovereign 7-dimensional Heptagon space.
///
/// All BahyWay particles — SQL records, file particles, AkkadianAOL scripts —
/// are positioned in this space. Forces, velocities, and accelerations are
/// all `Vec7D` instances.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec7D {
    /// ME 𒈨 — Intent / Set Theory dimension
    pub me: f64,
    /// GU 𒄖 — Mass / Cardinality dimension
    pub gu: f64,
    /// SAG 𒊕 — Gravity / Selectivity dimension
    pub sag: f64,
    /// A 𒀀 — Viscosity / IO Friction dimension
    pub a: f64,
    /// IZI 𒉈 — Heat / CPU Burn dimension
    pub izi: f64,
    /// UD 𒌓 — Volatility / Cache dimension
    pub ud: f64,
    /// URU 𒌷 — Strength / Index Health dimension
    pub uru: f64,
}

impl Vec7D {
    // ── Constants ─────────────────────────────────────────────────────────

    /// The zero vector — no force, no velocity, no position
    pub const ZERO: Self = Self {
        me: 0.0,
        gu: 0.0,
        sag: 0.0,
        a: 0.0,
        izi: 0.0,
        ud: 0.0,
        uru: 0.0,
    };

    /// The unit vector — equal weight on all 7 dimensions
    pub const UNIT: Self = Self {
        me: 1.0,
        gu: 1.0,
        sag: 1.0,
        a: 1.0,
        izi: 1.0,
        ud: 1.0,
        uru: 1.0,
    };

    /// The healthy orbit center — a particle at this position
    /// is perfectly sovereign across all 7 dimensions.
    pub const HEALTHY_ORBIT: Self = Self {
        me: 1.0,
        gu: 1.0,
        sag: 1.0,
        a: 1.0,
        izi: 1.0,
        ud: 1.0,
        uru: 1.0,
    };

    // ── Constructors ───────────────────────────────────────────────────────

    /// Construct a Vec7D from named dimension values
    #[allow(clippy::too_many_arguments)]
    pub fn new(me: f64, gu: f64, sag: f64, a: f64, izi: f64, ud: f64, uru: f64) -> Self {
        Self {
            me,
            gu,
            sag,
            a,
            izi,
            ud,
            uru,
        }
    }

    /// Construct from a `[f64; 7]` array in Heptagon order
    /// `[me, gu, sag, a, izi, ud, uru]`
    pub fn from_array(arr: [f64; 7]) -> Self {
        Self {
            me: arr[0],
            gu: arr[1],
            sag: arr[2],
            a: arr[3],
            izi: arr[4],
            ud: arr[5],
            uru: arr[6],
        }
    }

    /// Export to `[f64; 7]` in Heptagon order
    pub fn to_array(self) -> [f64; 7] {
        [
            self.me, self.gu, self.sag, self.a, self.izi, self.ud, self.uru,
        ]
    }

    /// Construct a vector with one dimension set and all others zero
    pub fn on_dimension(dim: crate::force::HeptaDimension, value: f64) -> Self {
        use crate::force::HeptaDimension::*;
        let mut v = Self::ZERO;
        match dim {
            ME => v.me = value,
            GU => v.gu = value,
            SAG => v.sag = value,
            A => v.a = value,
            IZI => v.izi = value,
            UD => v.ud = value,
            URU => v.uru = value,
        }
        v
    }

    // ── Core Algebra ───────────────────────────────────────────────────────

    /// Sum of squares — avoids sqrt when comparison is sufficient
    pub fn magnitude_squared(self) -> f64 {
        self.me * self.me
            + self.gu * self.gu
            + self.sag * self.sag
            + self.a * self.a
            + self.izi * self.izi
            + self.ud * self.ud
            + self.uru * self.uru
    }

    /// Euclidean magnitude in 7D space
    pub fn magnitude(self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    /// Dot product — the inner product of two sovereign vectors
    pub fn dot(self, other: Vec7D) -> f64 {
        self.me * other.me
            + self.gu * other.gu
            + self.sag * other.sag
            + self.a * other.a
            + self.izi * other.izi
            + self.ud * other.ud
            + self.uru * other.uru
    }

    /// Normalize to unit magnitude — project onto the unit sovereign sphere
    /// Returns ZERO if magnitude is negligible (avoids division by zero)
    pub fn normalize(self) -> Vec7D {
        let mag = self.magnitude();
        if mag < f64::EPSILON {
            Vec7D::ZERO
        } else {
            self * (1.0 / mag)
        }
    }

    /// Inverse magnitude — for inverse-mass calculations.
    /// Returns 0.0 if magnitude is negligible (infinite mass — immovable)
    pub fn inverse_magnitude(self) -> f64 {
        let mag = self.magnitude();
        if mag < f64::EPSILON {
            0.0
        } else {
            1.0 / mag
        }
    }

    /// Component-wise clamp to `[min, max]`
    pub fn clamp(self, min: f64, max: f64) -> Vec7D {
        Vec7D {
            me: self.me.clamp(min, max),
            gu: self.gu.clamp(min, max),
            sag: self.sag.clamp(min, max),
            a: self.a.clamp(min, max),
            izi: self.izi.clamp(min, max),
            ud: self.ud.clamp(min, max),
            uru: self.uru.clamp(min, max),
        }
    }

    // ── Sovereign Analytical Operations ───────────────────────────────────

    /// **Alignment score** — cosine similarity in `[-1.0, 1.0]`
    ///
    /// - `+1.0` = perfectly aligned across all 7 dimensions (ideal JOIN)
    /// - ` 0.0` = orthogonal (no relationship)
    /// - `-1.0` = directly opposing (worst possible JOIN conflict)
    ///
    /// This is the mathematical basis of DMW Level 4 (A — Viscosity) JOIN analysis.
    pub fn alignment(self, other: Vec7D) -> f64 {
        let denom = self.magnitude() * other.magnitude();
        if denom < f64::EPSILON {
            0.0
        } else {
            (self.dot(other) / denom).clamp(-1.0, 1.0)
        }
    }

    /// **Orbital radius** from a center point — the distance in 7D space
    ///
    /// A particle's orbital radius from `HEALTHY_ORBIT` IS its health distance.
    /// Small radius → healthy orbit. Large radius → approaching dead zone.
    pub fn orbital_radius(self, center: Vec7D) -> f64 {
        (self - center).magnitude()
    }

    /// **Sigma (σ)** — uncertainty in `[0.0, 1.0]`
    ///
    /// The wavefunction collapse model:
    /// - `σ = 0.0` → perfectly sovereign (certain, aligned with reference)
    /// - `σ = 1.0` → raw chaos (maximally uncertain)
    ///
    /// As a particle passes through the 7 Heptagon levels, σ collapses
    /// from ~0.9 (L1 Raw) toward ~0.01 (L7 Sovereign).
    pub fn sigma(self, reference: Vec7D) -> f64 {
        // alignment=1.0  → sigma=0.0  (sovereign)
        // alignment=-1.0 → sigma=1.0  (chaotic)
        ((1.0 - self.alignment(reference)) / 2.0).clamp(0.0, 1.0)
    }

    /// **Health score** — distance-based, in `[0.0, 1.0]`
    ///
    /// Computed from orbital radius relative to the maximum possible
    /// distance in 7D unit space (√7 ≈ 2.646).
    pub fn health_score(self, healthy_orbit: Vec7D) -> f64 {
        let max_distance = 7.0_f64.sqrt(); // diagonal of unit 7D cube
        let radius = self.orbital_radius(healthy_orbit);
        (1.0 - radius / max_distance).clamp(0.0, 1.0)
    }
}

// ── Operator Overloading ────────────────────────────────────────────────────

impl Add for Vec7D {
    type Output = Vec7D;
    fn add(self, rhs: Vec7D) -> Vec7D {
        Vec7D {
            me: self.me + rhs.me,
            gu: self.gu + rhs.gu,
            sag: self.sag + rhs.sag,
            a: self.a + rhs.a,
            izi: self.izi + rhs.izi,
            ud: self.ud + rhs.ud,
            uru: self.uru + rhs.uru,
        }
    }
}

impl AddAssign for Vec7D {
    fn add_assign(&mut self, rhs: Vec7D) {
        *self = *self + rhs;
    }
}

impl Sub for Vec7D {
    type Output = Vec7D;
    fn sub(self, rhs: Vec7D) -> Vec7D {
        Vec7D {
            me: self.me - rhs.me,
            gu: self.gu - rhs.gu,
            sag: self.sag - rhs.sag,
            a: self.a - rhs.a,
            izi: self.izi - rhs.izi,
            ud: self.ud - rhs.ud,
            uru: self.uru - rhs.uru,
        }
    }
}

impl SubAssign for Vec7D {
    fn sub_assign(&mut self, rhs: Vec7D) {
        *self = *self - rhs;
    }
}

impl Mul<f64> for Vec7D {
    type Output = Vec7D;
    fn mul(self, scalar: f64) -> Vec7D {
        Vec7D {
            me: self.me * scalar,
            gu: self.gu * scalar,
            sag: self.sag * scalar,
            a: self.a * scalar,
            izi: self.izi * scalar,
            ud: self.ud * scalar,
            uru: self.uru * scalar,
        }
    }
}

impl MulAssign<f64> for Vec7D {
    fn mul_assign(&mut self, scalar: f64) {
        *self = *self * scalar;
    }
}

impl Neg for Vec7D {
    type Output = Vec7D;
    fn neg(self) -> Vec7D {
        self * -1.0
    }
}

impl Default for Vec7D {
    fn default() -> Self {
        Self::ZERO
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn zero_vector_all_dimensions_are_zero() {
        let v = Vec7D::ZERO;
        assert_eq!(v.to_array(), [0.0; 7]);
    }

    #[test]
    fn from_array_roundtrip() {
        let arr = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert_eq!(Vec7D::from_array(arr).to_array(), arr);
    }

    #[test]
    fn new_sets_all_named_dimensions() {
        let v = Vec7D::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
        assert_eq!(v.me, 1.0);
        assert_eq!(v.gu, 2.0);
        assert_eq!(v.sag, 3.0);
        assert_eq!(v.a, 4.0);
        assert_eq!(v.izi, 5.0);
        assert_eq!(v.ud, 6.0);
        assert_eq!(v.uru, 7.0);
    }

    #[test]
    fn magnitude_of_zero_is_zero() {
        assert!(approx(Vec7D::ZERO.magnitude(), 0.0));
    }

    #[test]
    fn magnitude_of_unit_axis_vector_is_one() {
        let v = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(v.magnitude(), 1.0));
    }

    #[test]
    fn magnitude_of_unit_vector_is_sqrt_7() {
        let expected = 7.0_f64.sqrt();
        assert!(approx(Vec7D::UNIT.magnitude(), expected));
    }

    #[test]
    fn magnitude_squared_avoids_sqrt() {
        let v = Vec7D::new(3.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(v.magnitude_squared(), 25.0));
        assert!(approx(v.magnitude(), 5.0));
    }

    #[test]
    fn dot_of_orthogonal_vectors_is_zero() {
        let a = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let b = Vec7D::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(a.dot(b), 0.0));
    }

    #[test]
    fn dot_of_vector_with_itself_is_magnitude_squared() {
        let v = Vec7D::new(1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(v.dot(v), v.magnitude_squared()));
    }

    #[test]
    fn dot_is_commutative() {
        let a = Vec7D::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
        let b = Vec7D::new(7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0);
        assert!(approx(a.dot(b), b.dot(a)));
    }

    #[test]
    fn normalization_produces_unit_magnitude() {
        let v = Vec7D::new(3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0);
        let n = v.normalize();
        assert!((n.magnitude() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn normalization_of_zero_returns_zero() {
        assert_eq!(Vec7D::ZERO.normalize(), Vec7D::ZERO);
    }

    #[test]
    fn alignment_of_identical_vectors_is_one() {
        let v = Vec7D::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
        assert!(approx(v.alignment(v), 1.0));
    }

    #[test]
    fn alignment_of_opposite_vectors_is_minus_one() {
        let v = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(v.alignment(-v), -1.0));
    }

    #[test]
    fn alignment_of_orthogonal_vectors_is_zero() {
        let a = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let b = Vec7D::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(a.alignment(b), 0.0));
    }

    #[test]
    fn alignment_of_zero_vector_is_zero() {
        let v = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(Vec7D::ZERO.alignment(v), 0.0));
    }

    #[test]
    fn orbital_radius_from_self_is_zero() {
        let v = Vec7D::UNIT;
        assert!(approx(v.orbital_radius(v), 0.0));
    }

    #[test]
    fn orbital_radius_from_origin() {
        let v = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(v.orbital_radius(Vec7D::ZERO), 1.0));
    }

    #[test]
    fn sigma_of_identical_vectors_is_zero() {
        let v = Vec7D::UNIT;
        assert!(approx(v.sigma(v), 0.0));
    }

    #[test]
    fn sigma_of_opposite_vectors_is_one() {
        let v = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(v.sigma(-v), 1.0));
    }

    #[test]
    fn sigma_of_orthogonal_vectors_is_half() {
        let a = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let b = Vec7D::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(approx(a.sigma(b), 0.5));
    }

    #[test]
    fn health_score_on_healthy_orbit_is_one() {
        assert!(approx(
            Vec7D::HEALTHY_ORBIT.health_score(Vec7D::HEALTHY_ORBIT),
            1.0
        ));
    }

    #[test]
    fn health_score_at_zero_is_minimum() {
        let score = Vec7D::ZERO.health_score(Vec7D::HEALTHY_ORBIT);
        assert!(score < 0.5, "score={score}");
    }

    #[test]
    fn add_operator() {
        let a = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let b = Vec7D::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let c = a + b;
        assert!(approx(c.me, 1.0));
        assert!(approx(c.gu, 1.0));
    }

    #[test]
    fn sub_operator() {
        let a = Vec7D::new(3.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let b = Vec7D::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let c = a - b;
        assert!(approx(c.me, 2.0));
        assert!(approx(c.gu, 1.0));
    }

    #[test]
    fn mul_scalar_operator() {
        let v = Vec7D::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
        let scaled = v * 2.0;
        assert!(approx(scaled.me, 2.0));
        assert!(approx(scaled.uru, 14.0));
    }

    #[test]
    fn neg_operator_reverses_all_dimensions() {
        let v = Vec7D::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
        let neg = -v;
        assert!(approx(neg.me, -1.0));
        assert!(approx(neg.uru, -7.0));
    }

    #[test]
    fn add_assign_operator() {
        let mut v = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        v += Vec7D::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        assert!(approx(v.me, 1.0));
        assert!(approx(v.uru, 1.0));
    }

    #[test]
    fn clamp_keeps_values_in_range() {
        let v = Vec7D::new(-2.0, 0.5, 1.5, 0.0, 3.0, -1.0, 0.8);
        let c = v.clamp(0.0, 1.0);
        for val in c.to_array() {
            assert!(val >= 0.0 && val <= 1.0, "value {val} out of [0,1]");
        }
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(Vec7D::default(), Vec7D::ZERO);
    }
}
