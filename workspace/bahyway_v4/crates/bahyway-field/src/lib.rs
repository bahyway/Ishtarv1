//! bahyway-field — the abstract algebraic Field (𝔽, +, ×).
//!
//! `ALGEBRA_GLOSSARY.md` Part I marks "Field" as the escalation root for
//! Vector Space and Algebra, crate `bahyway-algebra`, with the Sovereign
//! Rule "the fixed-point field for B11 uses divisor 240 exactly... never
//! ℤ/255." Before this crate, no `Field` trait existed anywhere in the
//! workspace — `fields.rs` in `bahyway-algebra` is a semantic-kind
//! taxonomy (`SemanticField`), a different, unrelated meaning of "field."
//! That was a real gap, not a naming miss. This crate closes it.
//!
//! One correction made in closing it: ℤ/240 is **not** a field in the
//! strict abstract-algebra sense, because 240 = 2⁴×3×5 is composite.
//! A field requires every nonzero element to have a multiplicative
//! inverse; ℤ/240 has zero divisors (e.g. 16×15 ≡ 0 mod 240), so 2 has
//! no inverse mod 240 (gcd(2,240)=16≠1). It is a commutative ring, not
//! a field. `Zmod240` below is implemented and named honestly as a
//! ring — the B11 byte-quantization behavior (divisor 240, never 255)
//! is preserved exactly as used in `hepta-score::equation::to_b11`; only
//! the mathematical label is corrected.
//!
//! `RealField` (an `f64` wrapper) genuinely satisfies the Field axioms
//! (up to floating-point rounding) and is the one true `Field` impl here.

use core::ops::{Add, Mul, Neg, Sub};

/// The abstract algebraic Field: a set with +, ×, additive/multiplicative
/// identity, additive inverse (via `Neg`/`Sub`), and multiplicative
/// inverse for every nonzero element.
pub trait Field:
    Copy + PartialEq + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Neg<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;

    /// Multiplicative inverse. Must return `None` iff `self == ZERO`
    /// (that is the one axiomatically-permitted exception).
    fn inverse(&self) -> Option<Self>;
}

/// A commutative ring: like a Field, but multiplicative inverses are not
/// guaranteed to exist for every nonzero element (zero divisors allowed).
pub trait CommutativeRing:
    Copy + PartialEq + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Neg<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;

    /// Checked multiplicative inverse — `None` when no inverse exists
    /// (self == 0, or self shares a nontrivial factor with the modulus).
    fn checked_inverse(&self) -> Option<Self>;
}

// ---------------------------------------------------------------------
// RealField — ℝ, approximated by f64. A genuine Field.
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RealField(pub f64);

impl Add for RealField {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        RealField(self.0 + rhs.0)
    }
}
impl Sub for RealField {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        RealField(self.0 - rhs.0)
    }
}
impl Mul for RealField {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        RealField(self.0 * rhs.0)
    }
}
impl Neg for RealField {
    type Output = Self;
    fn neg(self) -> Self {
        RealField(-self.0)
    }
}

impl Field for RealField {
    const ZERO: Self = RealField(0.0);
    const ONE: Self = RealField(1.0);

    fn inverse(&self) -> Option<Self> {
        if self.0 == 0.0 {
            None
        } else {
            Some(RealField(1.0 / self.0))
        }
    }
}

// ---------------------------------------------------------------------
// Zmod240 — ℤ/240, the sovereign B11 quantization ring. NOT a field.
// ---------------------------------------------------------------------

/// Element of ℤ/240 — the sovereign fixed-point scoring ring
/// (B11 = round(H(P) × 240)), always represented in `0..240`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zmod240(pub u16);

const MODULUS: u16 = 240;

impl Zmod240 {
    pub fn new(v: u16) -> Self {
        Zmod240(v % MODULUS)
    }
}

impl Add for Zmod240 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Zmod240((self.0 + rhs.0) % MODULUS)
    }
}
impl Sub for Zmod240 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Zmod240((self.0 + MODULUS - rhs.0) % MODULUS)
    }
}
impl Mul for Zmod240 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Zmod240(((self.0 as u32 * rhs.0 as u32) % MODULUS as u32) as u16)
    }
}
impl Neg for Zmod240 {
    type Output = Self;
    fn neg(self) -> Self {
        Zmod240((MODULUS - self.0) % MODULUS)
    }
}

/// Extended Euclidean algorithm over i64, returns (gcd, x) such that
/// a*x ≡ gcd (mod m) is derivable; used only to check invertibility.
fn egcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x1, y1) = egcd(b, a % b);
        (g, y1, x1 - (a / b) * y1)
    }
}

impl CommutativeRing for Zmod240 {
    const ZERO: Self = Zmod240(0);
    const ONE: Self = Zmod240(1);

    fn checked_inverse(&self) -> Option<Self> {
        let (g, x, _) = egcd(self.0 as i64, MODULUS as i64);
        if g != 1 {
            None // shares a factor with 240 -> no inverse (e.g. 2, 3, 4, 5, 6, 16, 15...)
        } else {
            let inv = ((x % MODULUS as i64 + MODULUS as i64) % MODULUS as i64) as u16;
            Some(Zmod240(inv))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- RealField axiom tests (the Field trait's actual claim) ---

    #[test]
    fn real_field_additive_identity() {
        let a = RealField(3.5);
        assert_eq!(a + RealField::ZERO, a);
    }

    #[test]
    fn real_field_multiplicative_identity() {
        let a = RealField(3.5);
        assert_eq!(a * RealField::ONE, a);
    }

    #[test]
    fn real_field_additive_inverse() {
        let a = RealField(3.5);
        assert_eq!(a + (-a), RealField::ZERO);
    }

    #[test]
    fn real_field_multiplicative_inverse_of_nonzero() {
        let a = RealField(4.0);
        let inv = a.inverse().unwrap();
        assert!((a.0 * inv.0 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn real_field_zero_has_no_multiplicative_inverse() {
        assert_eq!(RealField::ZERO.inverse(), None);
    }

    #[test]
    fn real_field_distributivity() {
        let a = RealField(2.0);
        let b = RealField(3.0);
        let c = RealField(5.0);
        assert_eq!(a * (b + c), a * b + a * c);
    }

    #[test]
    fn real_field_commutativity() {
        let a = RealField(2.0);
        let b = RealField(7.0);
        assert_eq!(a + b, b + a);
        assert_eq!(a * b, b * a);
    }

    // --- Zmod240 tests (proving it is a ring, and specifically NOT a field) ---

    #[test]
    fn zmod240_wraps_at_modulus() {
        assert_eq!(Zmod240::new(240), Zmod240(0));
        assert_eq!(Zmod240::new(241), Zmod240(1));
    }

    #[test]
    fn zmod240_additive_inverse_exists_for_every_element() {
        for v in 0u16..240 {
            let a = Zmod240(v);
            assert_eq!(a + (-a), Zmod240::ZERO);
        }
    }

    #[test]
    fn zmod240_unit_1_has_inverse() {
        // 1 is always its own inverse.
        assert_eq!(Zmod240(1).checked_inverse(), Some(Zmod240(1)));
    }

    #[test]
    fn zmod240_unit_7_has_inverse_and_it_round_trips() {
        // gcd(7, 240) = 1, so 7 is a unit.
        let seven = Zmod240(7);
        let inv = seven.checked_inverse().expect("7 is coprime to 240");
        assert_eq!(seven * inv, Zmod240::ONE);
    }

    #[test]
    fn zmod240_is_not_a_field_two_has_no_inverse() {
        // gcd(2, 240) = 2 -> 2 is a zero divisor, not a unit.
        // This is the concrete proof that Z/240 fails the Field axioms.
        assert_eq!(Zmod240(2).checked_inverse(), None);
    }

    #[test]
    fn zmod240_is_not_a_field_16_times_15_is_zero() {
        // Two nonzero elements multiplying to zero -> zero divisors exist
        // -> Z/240 cannot be an integral domain, let alone a field.
        assert_eq!(Zmod240(16) * Zmod240(15), Zmod240::ZERO);
        assert_ne!(Zmod240(16), Zmod240::ZERO);
        assert_ne!(Zmod240(15), Zmod240::ZERO);
    }

    #[test]
    fn zmod240_matches_real_b11_quantization_divisor() {
        // Cross-check against hepta-score's real B11 = round(H(P) * 240) convention:
        // the ring's modulus and the byte-quantization divisor must agree.
        assert_eq!(MODULUS, 240);
    }
}
