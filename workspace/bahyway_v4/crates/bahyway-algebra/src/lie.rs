//! SU(7) — the TRANSITION law of BahyWay v4.0.
//! Distinct from E7 (the lattice law in hepta-shell-index).
//! Fundamental rep (7) = 7 Hepta Gates.
//! Adjoint rep (48) = 48 ETL transition types:
//!   21 symmetric (reversible promotions)
//!   21 antisymmetric (irreversible decay/exceptions)
//!    6 diagonal (PA-13 shell quantization)
//! Casimir C2(fundamental) = 24/7. No SQL. No KAKI minting here.
#![allow(clippy::needless_range_loop)]

/// Dimension of the fundamental representation.
pub const SU7_N: usize = 7;
/// Dimension of the adjoint representation: N^2 - 1.
pub const SU7_ADJOINT: usize = 48;
/// Number of diagonal (Cartan) generators: N - 1.
pub const SU7_CARTAN: usize = 6;
/// Quadratic Casimir of the fundamental rep = (N^2-1)/(2N) = 48/14 = 24/7.
pub const SU7_CASIMIR_FUND_NUM: u32 = 24;
pub const SU7_CASIMIR_FUND_DEN: u32 = 7;

/// A complex 7x7 matrix (row-major), split into real & imaginary parts.
#[derive(Debug, Clone, PartialEq)]
pub struct CMat7 {
    pub re: [[f64; 7]; 7],
    pub im: [[f64; 7]; 7],
}

impl CMat7 {
    pub fn zero() -> Self {
        CMat7 { re: [[0.0; 7]; 7], im: [[0.0; 7]; 7] }
    }

    /// Trace (real part) — for Casimir / normalization checks.
    pub fn trace_re(&self) -> f64 {
        (0..7).map(|i| self.re[i][i]).sum()
    }

    /// Is this matrix Hermitian? (generators of SU(N) are Hermitian.)
    pub fn is_hermitian(&self, eps: f64) -> bool {
        for i in 0..7 {
            for j in 0..7 {
                if (self.re[i][j] - self.re[j][i]).abs() > eps { return false; }
                if (self.im[i][j] + self.im[j][i]).abs() > eps { return false; }
            }
        }
        true
    }

    /// Is this matrix traceless? (SU(N) generators have trace 0.)
    pub fn is_traceless(&self, eps: f64) -> bool {
        self.trace_re().abs() < eps
            && (0..7).map(|i| self.im[i][i]).sum::<f64>().abs() < eps
    }
}

/// The generalized Gell-Mann basis for su(7).
/// Returns all 48 Hermitian traceless generators.
pub fn generate_su7_generators() -> Vec<CMat7> {
    let mut gens = Vec::with_capacity(SU7_ADJOINT);

    // (a) Symmetric off-diagonal: 21 generators
    //     T = E_ij + E_ji  (real, symmetric)
    for i in 0..7 {
        for j in (i + 1)..7 {
            let mut m = CMat7::zero();
            m.re[i][j] = 1.0;
            m.re[j][i] = 1.0;
            gens.push(m);
        }
    }

    // (b) Antisymmetric off-diagonal: 21 generators
    //     T = -i*E_ij + i*E_ji  (imaginary, antisymmetric)
    for i in 0..7 {
        for j in (i + 1)..7 {
            let mut m = CMat7::zero();
            m.im[i][j] = -1.0;
            m.im[j][i] = 1.0;
            gens.push(m);
        }
    }

    // (c) Diagonal (Cartan): 6 generators
    //     T_k = diag(1,...,1,-k,0,...) normalized by sqrt(2/(k(k+1)))
    for k in 1..7 {
        let mut m = CMat7::zero();
        let norm = (2.0 / (k as f64 * (k as f64 + 1.0))).sqrt();
        for d in 0..k { m.re[d][d] = norm; }
        m.re[k][k] = -(k as f64) * norm;
        gens.push(m);
    }

    gens
}

/// Casimir of the fundamental as an exact rational (24, 7).
pub fn casimir_fundamental() -> (u32, u32) {
    (SU7_CASIMIR_FUND_NUM, SU7_CASIMIR_FUND_DEN)
}

/// Classify a generator index into its transition family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionFamily {
    /// Reversible gate promotion (symmetric).
    Reversible,
    /// Irreversible decay / exception (antisymmetric).
    Irreversible,
    /// Shell quantization (diagonal / Cartan, PA-13).
    ShellQuantization,
}

pub fn family_of(index: usize) -> TransitionFamily {
    if index < 21 {
        TransitionFamily::Reversible
    } else if index < 42 {
        TransitionFamily::Irreversible
    } else {
        TransitionFamily::ShellQuantization
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjoint_dimension_is_48() {
        assert_eq!(generate_su7_generators().len(), SU7_ADJOINT);
        assert_eq!(SU7_N * SU7_N - 1, SU7_ADJOINT);
    }

    #[test]
    fn all_generators_hermitian_and_traceless() {
        for g in generate_su7_generators() {
            assert!(g.is_hermitian(1e-12), "generator must be Hermitian");
            assert!(g.is_traceless(1e-12), "generator must be traceless");
        }
    }

    #[test]
    fn family_partition_21_21_6() {
        let gens = generate_su7_generators();
        let mut rev = 0; let mut irr = 0; let mut shell = 0;
        for i in 0..gens.len() {
            match family_of(i) {
                TransitionFamily::Reversible => rev += 1,
                TransitionFamily::Irreversible => irr += 1,
                TransitionFamily::ShellQuantization => shell += 1,
            }
        }
        assert_eq!((rev, irr, shell), (21, 21, 6));
    }

    #[test]
    fn cartan_count_is_n_minus_1() {
        assert_eq!(SU7_CARTAN, SU7_N - 1);
    }

    #[test]
    fn casimir_is_24_over_7() {
        assert_eq!(casimir_fundamental(), (24, 7));
    }

    #[test]
    fn diagonal_generators_are_orthonormal_pairs() {
        // Two distinct Cartan generators are trace-orthogonal: Tr(T_a T_b)=0, a!=b.
        let gens = generate_su7_generators();
        let cartan: Vec<&CMat7> = gens[42..48].iter().collect();
        for a in 0..cartan.len() {
            for b in (a + 1)..cartan.len() {
                let mut tr = 0.0;
                for i in 0..7 { tr += cartan[a].re[i][i] * cartan[b].re[i][i]; }
                assert!(tr.abs() < 1e-12, "Cartan generators must be orthogonal");
            }
        }
    }
}
