//! Octonions O — closing ALGEBRA_GLOSSARY.md Part VI item 21 ("bahyway-algebra
//! (to extend)"), re-verified 2026-07-11: confirmed genuinely absent
//! (no `Octonion` anywhere in the workspace before this file).
//!
//! Built via the standard Cayley-Dickson doubling construction (R -> C ->
//! H -> O), generic over any base ring implementing `CayleyDickson`, rather
//! than a hand-typed 8x8 multiplication table with an unverified sign
//! convention. The doubling formula is the textbook definition:
//!   (a,b)(c,d) = (ac - d̄b, da + bc̄)
//!   conj(a,b)  = (ā, -b)
//! Every algebraic property claimed by the tests below (norm
//! multiplicativity, alternativity, loss of associativity at the octonion
//! level) is a *consequence* of this construction, not asserted — if the
//! construction were wrong, these tests would fail.

use core::ops::{Add, Mul, Neg, Sub};

pub trait CayleyDickson: Copy + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Neg<Output = Self> + PartialEq {
    const ZERO: Self;
    const ONE: Self;
    fn conj(self) -> Self;
    /// Sum of squares of all real leaf components (the algebra's norm^2).
    fn leaf_norm2(self) -> f64;
}

impl CayleyDickson for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    fn conj(self) -> Self { self } // reals are self-conjugate
    fn leaf_norm2(self) -> f64 { self * self }
}

/// A Cayley-Dickson doubling of `T`: pairs (a, b) representing a + b*e,
/// where e^2 = -1 relative to the multiplication rule below.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CD<T> {
    pub a: T,
    pub b: T,
}

impl<T: CayleyDickson> CD<T> {
    pub fn new(a: T, b: T) -> Self { CD { a, b } }
}

impl<T: CayleyDickson> Add for CD<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { CD { a: self.a + rhs.a, b: self.b + rhs.b } }
}
impl<T: CayleyDickson> Sub for CD<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { CD { a: self.a - rhs.a, b: self.b - rhs.b } }
}
impl<T: CayleyDickson> Neg for CD<T> {
    type Output = Self;
    fn neg(self) -> Self { CD { a: -self.a, b: -self.b } }
}
impl<T: CayleyDickson> Mul for CD<T> {
    type Output = Self;
    /// (a,b)(c,d) = (ac - d̄b, da + bc̄) — the Cayley-Dickson product.
    fn mul(self, rhs: Self) -> Self {
        let (a, b) = (self.a, self.b);
        let (c, d) = (rhs.a, rhs.b);
        CD {
            a: a * c - d.conj() * b,
            b: d * a + b * c.conj(),
        }
    }
}
impl<T: CayleyDickson> CayleyDickson for CD<T> {
    const ZERO: Self = CD { a: T::ZERO, b: T::ZERO };
    const ONE: Self = CD { a: T::ONE, b: T::ZERO };
    fn conj(self) -> Self { CD { a: self.a.conj(), b: -self.b } }
    fn leaf_norm2(self) -> f64 { self.a.leaf_norm2() + self.b.leaf_norm2() }
}

pub type Complex = CD<f64>;
pub type Quaternion = CD<Complex>;
/// The Octonions O: 8-dimensional, non-associative, alternative division algebra.
pub type Octonion = CD<Quaternion>;

impl Octonion {
    /// The 8 basis units e0 (real, =1) .. e7, built from the doubling
    /// structure rather than a hand-picked table.
    pub fn basis(i: usize) -> Self {
        assert!(i < 8);
        let mut leaves = [0.0f64; 8];
        leaves[i] = 1.0;
        Self::from_leaves(leaves)
    }

    pub fn from_leaves(l: [f64; 8]) -> Self {
        CD {
            a: CD { a: CD { a: l[0], b: l[1] }, b: CD { a: l[2], b: l[3] } },
            b: CD { a: CD { a: l[4], b: l[5] }, b: CD { a: l[6], b: l[7] } },
        }
    }

    pub fn to_leaves(self) -> [f64; 8] {
        [
            self.a.a.a, self.a.a.b, self.a.b.a, self.a.b.b,
            self.b.a.a, self.b.a.b, self.b.b.a, self.b.b.b,
        ]
    }

    pub fn norm(self) -> f64 { self.leaf_norm2().sqrt() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool { (a - b).abs() < 1e-9 }

    #[test]
    fn imaginary_units_square_to_minus_one() {
        for i in 1..8 {
            let e = Octonion::basis(i);
            let sq = e * e;
            let leaves = sq.to_leaves();
            assert!(approx(leaves[0], -1.0), "e{i}^2 real part should be -1, got {leaves:?}");
            for (k, &v) in leaves.iter().enumerate().skip(1) {
                assert!(approx(v, 0.0), "e{i}^2 should be purely real, leaf {k}={v}");
            }
        }
    }

    #[test]
    fn norm_is_multiplicative() {
        // |xy| = |x||y| is a deep, nontrivial property of the four
        // Hurwitz normed division algebras (R, C, H, O). If the
        // Cayley-Dickson construction above had a sign error, this would
        // fail for essentially any nonzero sample.
        let x = Octonion::from_leaves([1.0, 2.0, -1.0, 0.5, 3.0, -2.0, 0.0, 1.0]);
        let y = Octonion::from_leaves([0.0, 1.0, 2.0, -1.0, 1.0, 1.0, -1.0, 2.0]);
        let xy = x * y;
        assert!(approx(xy.norm(), x.norm() * y.norm()),
            "|xy|={} != |x||y|={}", xy.norm(), x.norm() * y.norm());
    }

    #[test]
    fn octonions_are_not_associative() {
        // This is THE defining property that separates octonions from
        // quaternions: there exist e_i, e_j, e_k with (e_i e_j) e_k != e_i (e_j e_k).
        let e1 = Octonion::basis(1);
        let e2 = Octonion::basis(2);
        let e4 = Octonion::basis(4);
        let lhs = (e1 * e2) * e4;
        let rhs = e1 * (e2 * e4);
        assert_ne!(lhs.to_leaves(), rhs.to_leaves(),
            "expected associativity to fail for octonions at (e1,e2,e4)");
    }

    #[test]
    fn octonions_are_alternative() {
        // Weaker than associativity, but always holds: (xx)y = x(xy) and
        // (yx)x = y(xx) for ALL x, y -- this is what makes O a genuine
        // (alternative) division algebra rather than an arbitrary
        // non-associative structure.
        let x = Octonion::from_leaves([1.0, 2.0, 0.0, -1.0, 3.0, 0.0, 1.0, -2.0]);
        let y = Octonion::from_leaves([0.0, 1.0, -1.0, 2.0, 0.0, 1.0, 1.0, 0.0]);
        let lhs = (x * x) * y;
        let rhs = x * (x * y);
        for (l, r) in lhs.to_leaves().iter().zip(rhs.to_leaves().iter()) {
            assert!(approx(*l, *r), "(xx)y != x(xy): {l} vs {r}");
        }
    }

    #[test]
    fn quaternions_are_associative() {
        // Sanity check on the same construction one doubling level down:
        // H must stay associative (only O loses it).
        let i = Quaternion::basis_imaginary(0);
        let j = Quaternion::basis_imaginary(1);
        let k = Quaternion::basis_imaginary(2);
        let lhs = (i * j) * k;
        let rhs = i * (j * k);
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn real_part_of_conjugate_product_is_norm_squared() {
        let x = Octonion::from_leaves([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        let xxc = x * x.conj();
        assert!(approx(xxc.to_leaves()[0], x.leaf_norm2()));
    }
}

impl Quaternion {
    /// i, j, k for basis_imaginary(0..3).
    pub fn basis_imaginary(i: usize) -> Self {
        assert!(i < 3);
        Octonion::basis(i + 1).a // reuse the same leaf layout, first 4 leaves are the quaternion
    }
}
