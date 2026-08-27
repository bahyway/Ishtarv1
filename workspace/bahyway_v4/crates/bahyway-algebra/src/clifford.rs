//! Clifford algebra Cl(7,0) over the 7 Hepta dimensions.
//! A FACT is a multivector, never a row. Composition is the
//! geometric/outer product — never a JOIN. TRIPLE-O only.
#![allow(clippy::needless_range_loop)]

/// 2^7 = 128 basis blades indexed by a 7-bit mask.
pub const BLADES: usize = 128;
pub const DIMS: usize = 7;

/// A multivector over Cl(7,0): coefficient per blade (bitmask index).
#[derive(Debug, Clone, PartialEq)]
pub struct Multivector {
    pub c: [f64; BLADES],
}

impl Multivector {
    pub fn zero() -> Self {
        Multivector { c: [0.0; BLADES] }
    }

    /// A pure scalar (grade-0) — a measure.
    pub fn scalar(x: f64) -> Self {
        let mut m = Self::zero();
        m.c[0] = x;
        m
    }

    /// A base dimension e_i (grade-1), i in 0..7.
    pub fn basis(i: usize) -> Self {
        assert!(i < DIMS);
        let mut m = Self::zero();
        m.c[1 << i] = 1.0;
        m
    }

    pub fn add(&self, o: &Multivector) -> Multivector {
        let mut r = Self::zero();
        for k in 0..BLADES {
            r.c[k] = self.c[k] + o.c[k];
        }
        r
    }

    /// Grade (popcount) of a blade index.
    pub fn blade_grade(mask: usize) -> u32 {
        (mask as u32).count_ones()
    }

    /// Extract the grade-g part.
    pub fn grade(&self, g: u32) -> Multivector {
        let mut r = Self::zero();
        for k in 0..BLADES {
            if Self::blade_grade(k) == g {
                r.c[k] = self.c[k];
            }
        }
        r
    }

    /// Sign of the geometric product of two blades in Cl(7,0)
    /// (Euclidean: e_i^2 = +1). Counts swaps to sort concatenation.
    fn blade_product_sign(mut a: usize, b: usize) -> (usize, f64) {
        // count transpositions moving b's bits past a's higher bits
        let mut swaps = 0u32;
        for bit in 0..DIMS {
            if (b >> bit) & 1 == 1 {
                // number of set bits in `a` above `bit`
                let higher = a & !((1 << (bit + 1)) - 1);
                swaps += (higher as u32).count_ones();
            }
        }
        let result = a ^ b; // matching bits annihilate (e_i^2=+1)
        let common = a & b;
        // each common bit contributes e_i^2 = +1 (no sign) in Euclidean
        let _ = common;
        a = result;
        let sign = if swaps.is_multiple_of(2) { 1.0 } else { -1.0 };
        (a, sign)
    }

    /// Geometric product — the sovereign fact composition (replaces JOIN).
    pub fn geometric(&self, o: &Multivector) -> Multivector {
        let mut r = Self::zero();
        for i in 0..BLADES {
            if self.c[i] == 0.0 {
                continue;
            }
            for j in 0..BLADES {
                if o.c[j] == 0.0 {
                    continue;
                }
                let (blade, sign) = Self::blade_product_sign(i, j);
                r.c[blade] += sign * self.c[i] * o.c[j];
            }
        }
        r
    }

    /// Outer (wedge) product — natural dimensional combination.
    pub fn wedge(&self, o: &Multivector) -> Multivector {
        let mut r = Self::zero();
        for i in 0..BLADES {
            if self.c[i] == 0.0 {
                continue;
            }
            for j in 0..BLADES {
                if o.c[j] == 0.0 {
                    continue;
                }
                if i & j != 0 {
                    continue;
                } // shared factor => wedge is 0
                let (blade, sign) = Self::blade_product_sign(i, j);
                r.c[blade] += sign * self.c[i] * o.c[j];
            }
        }
        r
    }

    pub fn norm2(&self) -> f64 {
        self.c.iter().map(|x| x * x).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_and_basis_grades() {
        assert_eq!(Multivector::scalar(3.0).grade(0).c[0], 3.0);
        assert_eq!(Multivector::basis(2).grade(1).c[1 << 2], 1.0);
    }

    #[test]
    fn basis_squares_to_one_euclidean() {
        let e1 = Multivector::basis(1);
        let sq = e1.geometric(&e1);
        assert!((sq.c[0] - 1.0).abs() < 1e-12, "e_i^2 = +1");
    }

    #[test]
    fn wedge_is_antisymmetric() {
        let e0 = Multivector::basis(0);
        let e1 = Multivector::basis(1);
        let a = e0.wedge(&e1);
        let b = e1.wedge(&e0);
        // e0^e1 = -(e1^e0)
        for k in 0..BLADES {
            assert!((a.c[k] + b.c[k]).abs() < 1e-12);
        }
    }

    #[test]
    fn wedge_of_dependent_is_zero() {
        let e3 = Multivector::basis(3);
        assert!(e3.wedge(&e3).norm2() < 1e-12);
    }

    #[test]
    fn fact_composition_produces_bivector() {
        // A time-measure fact wedged with a product-measure fact yields
        // a grade-2 interaction term (time^product) — no JOIN needed.
        let time = Multivector::basis(0);
        let product = Multivector::basis(1);
        let interaction = time.wedge(&product);
        assert_eq!(interaction.grade(2).norm2() > 0.0, true);
        assert!(interaction.grade(1).norm2() < 1e-12);
    }
}
