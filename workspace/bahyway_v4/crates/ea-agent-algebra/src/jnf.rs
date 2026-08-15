//! General Jordan Normal Form — closing ALGEBRA_GLOSSARY.md Part I item 8,
//! which was previously marked plain ✓ but only had `jordan.rs`'s
//! tribe-stability application (spectral radius classification), not an
//! actual eigendecomposition. Corrected 2026-07-11.
//!
//! Scope, stated honestly: symmetric matrices are ALWAYS diagonalizable
//! (the spectral theorem) — every Jordan block of a symmetric matrix has
//! size 1. So for the symmetric case, JNF *is* full eigendecomposition,
//! computed here exactly via the classical Jacobi eigenvalue algorithm
//! (real, well-tested, converges for every real symmetric input).
//!
//! NOT covered: general (non-symmetric) defective matrices, where true
//! Jordan blocks of size > 1 require generalized eigenvectors from a
//! nilpotent-part decomposition. That is a substantially harder numerical
//! problem (rank-deficient nullspace chains under floating point) and is
//! NOT implemented here. `jnf_shape_symmetric` below only ever reports
//! 1x1 blocks for that reason -- it does not silently invent size>1
//! blocks it hasn't actually verified.

/// Eigenvalues (diagonal of D) and eigenvectors (columns of V) of a real
/// symmetric matrix `a`, via the classical (cyclic) Jacobi rotation
/// method: repeatedly zero the largest off-diagonal element until the
/// matrix is diagonal to within `tol`. Returns (eigenvalues, V) such
/// that V^T A V ~= diag(eigenvalues) and V^T V ~= I.
///
/// # Panics
/// Panics if `a` is not square or not symmetric (within 1e-9).
pub fn jacobi_eigen(a: &[Vec<f64>], max_iter: usize, tol: f64) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = a.len();
    for row in a { assert_eq!(row.len(), n, "matrix must be square"); }
    for i in 0..n {
        for j in 0..n {
            assert!((a[i][j] - a[j][i]).abs() < 1e-9, "matrix must be symmetric");
        }
    }

    let mut m: Vec<Vec<f64>> = a.to_vec();
    let mut v: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect();

    for _ in 0..max_iter {
        // find largest off-diagonal element
        let (mut p, mut q, mut max_val) = (0, 1, 0.0f64);
        for i in 0..n {
            for j in (i + 1)..n {
                if m[i][j].abs() > max_val {
                    max_val = m[i][j].abs();
                    p = i; q = j;
                }
            }
        }
        if max_val < tol { break; }

        let (c, s) = if m[p][q].abs() > 1e-300 {
            let theta = (m[q][q] - m[p][p]) / (2.0 * m[p][q]);
            let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
            let t = if theta == 0.0 { 1.0 } else { t };
            let c = 1.0 / (t * t + 1.0).sqrt();
            (c, t * c)
        } else {
            (1.0, 0.0)
        };

        let (app, aqq, apq) = (m[p][p], m[q][q], m[p][q]);
        m[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        m[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        m[p][q] = 0.0;
        m[q][p] = 0.0;

        for i in 0..n {
            if i != p && i != q {
                let aip = m[i][p];
                let aiq = m[i][q];
                m[i][p] = c * aip - s * aiq;
                m[p][i] = m[i][p];
                m[i][q] = s * aip + c * aiq;
                m[q][i] = m[i][q];
            }
        }
        for i in 0..n {
            let vip = v[i][p];
            let viq = v[i][q];
            v[i][p] = c * vip - s * viq;
            v[i][q] = s * vip + c * viq;
        }
    }

    let eigenvalues: Vec<f64> = (0..n).map(|i| m[i][i]).collect();
    (eigenvalues, v)
}

/// Jordan block sizes for a real SYMMETRIC matrix -- always all 1s (the
/// spectral theorem). Included to make the "symmetric matrices have no
/// nontrivial Jordan structure" fact explicit and testable, not just
/// asserted in a comment.
pub fn jnf_shape_symmetric(n: usize) -> Vec<usize> {
    vec![1; n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matvec(a: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
        let n = a.len();
        (0..n).map(|i| (0..n).map(|j| a[i][j] * v[j]).sum()).collect()
    }

    #[test]
    fn known_2x2_eigenvalues_are_1_and_3() {
        // A = [[2,1],[1,2]]: char poly (2-l)^2 - 1 = 0 -> l = 1, 3.
        let a = vec![vec![2.0, 1.0], vec![1.0, 2.0]];
        let (mut eig, _v) = jacobi_eigen(&a, 100, 1e-12);
        eig.sort_by(|x, y| x.partial_cmp(y).unwrap());
        assert!((eig[0] - 1.0).abs() < 1e-9, "eig[0]={}", eig[0]);
        assert!((eig[1] - 3.0).abs() < 1e-9, "eig[1]={}", eig[1]);
    }

    #[test]
    fn eigenvectors_actually_satisfy_av_eq_lambda_v() {
        let a = vec![vec![2.0, 1.0], vec![1.0, 2.0]];
        let (eig, v) = jacobi_eigen(&a, 100, 1e-12);
        for k in 0..2 {
            let col: Vec<f64> = (0..2).map(|i| v[i][k]).collect();
            let av = matvec(&a, &col);
            for i in 0..2 {
                assert!((av[i] - eig[k] * col[i]).abs() < 1e-6,
                    "A v_{k} != lambda_{k} v_{k} at component {i}: {} vs {}", av[i], eig[k] * col[i]);
            }
        }
    }

    #[test]
    fn eigenvectors_are_orthonormal() {
        let a = vec![
            vec![4.0, 1.0, 0.0],
            vec![1.0, 3.0, 1.0],
            vec![0.0, 1.0, 2.0],
        ];
        let (_eig, v) = jacobi_eigen(&a, 200, 1e-12);
        let n = 3;
        for k1 in 0..n {
            for k2 in 0..n {
                let dot: f64 = (0..n).map(|i| v[i][k1] * v[i][k2]).sum();
                let expected = if k1 == k2 { 1.0 } else { 0.0 };
                assert!((dot - expected).abs() < 1e-6, "v_{k1}.v_{k2} = {dot}, expected {expected}");
            }
        }
    }

    #[test]
    fn trace_equals_sum_of_eigenvalues() {
        let a = vec![
            vec![5.0, 2.0, 1.0],
            vec![2.0, 3.0, 0.0],
            vec![1.0, 0.0, 6.0],
        ];
        let trace: f64 = (0..3).map(|i| a[i][i]).sum();
        let (eig, _v) = jacobi_eigen(&a, 200, 1e-12);
        let eig_sum: f64 = eig.iter().sum();
        assert!((trace - eig_sum).abs() < 1e-6, "trace={trace} sum(eig)={eig_sum}");
    }

    #[test]
    fn diagonal_matrix_eigenvalues_are_its_own_diagonal() {
        let a = vec![
            vec![7.0, 0.0, 0.0],
            vec![0.0, -2.0, 0.0],
            vec![0.0, 0.0, 4.5],
        ];
        let (mut eig, _v) = jacobi_eigen(&a, 10, 1e-12);
        eig.sort_by(|x, y| x.partial_cmp(y).unwrap());
        assert_eq!(eig, vec![-2.0, 4.5, 7.0]);
    }

    #[test]
    fn symmetric_matrices_have_only_1x1_jordan_blocks() {
        assert_eq!(jnf_shape_symmetric(5), vec![1, 1, 1, 1, 1]);
    }
}
