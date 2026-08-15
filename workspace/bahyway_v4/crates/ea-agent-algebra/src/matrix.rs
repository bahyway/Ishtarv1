//! matrix.rs — Sovereign pure Rust matrix algebra.
//! No nalgebra. No ndarray. No external deps.
#![forbid(unsafe_code)]

use ea_agent_core::constants::ABZU_PRECISION;

/// A sovereign N×N matrix of f64 values.
#[derive(Debug, Clone)]
pub struct SovereignMatrix {
    pub rows: usize,
    pub cols: usize,
    data: Vec<f64>,
}

/// An eigenvalue (may be complex — stored as real + imaginary parts).
#[derive(Debug, Clone, PartialEq)]
pub struct Eigenvalue {
    pub real: f64,
    pub imag: f64,
}

impl Eigenvalue {
    pub fn real(v: f64) -> Self { Self { real: v, imag: 0.0 } }
    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }
    pub fn is_real(&self) -> bool { self.imag.abs() < ABZU_PRECISION }
}

impl SovereignMatrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: vec![0.0; rows * cols] }
    }

    pub fn identity(n: usize) -> Self {
        let mut m = Self::new(n, n);
        for i in 0..n { m.set(i, i, 1.0); }
        m
    }

    pub fn from_vec(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        assert_eq!(data.len(), rows * cols);
        Self { rows, cols, data }
    }

    /// Build from flat row-major slice.
    pub fn from_slice(rows: usize, cols: usize, data: &[f64]) -> Self {
        assert_eq!(data.len(), rows * cols);
        Self { rows, cols, data: data.to_vec() }
    }

    pub fn get(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols + c]
    }

    pub fn set(&mut self, r: usize, c: usize, v: f64) {
        self.data[r * self.cols + c] = v;
    }

    /// Matrix trace (sum of diagonal).
    pub fn trace(&self) -> f64 {
        (0..self.rows.min(self.cols)).map(|i| self.get(i, i)).sum()
    }

    /// Matrix multiply: self (m×k) × other (k×n) → (m×n).
    pub fn mul(&self, other: &SovereignMatrix) -> SovereignMatrix {
        assert_eq!(self.cols, other.rows);
        let mut result = SovereignMatrix::new(self.rows, other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut s = 0.0f64;
                for k in 0..self.cols { s += self.get(i, k) * other.get(k, j); }
                result.set(i, j, s);
            }
        }
        result
    }

    /// Matrix-vector multiply: self (m×n) × v (n) → (m).
    pub fn mul_vec(&self, v: &[f64]) -> Vec<f64> {
        assert_eq!(self.cols, v.len());
        (0..self.rows).map(|i| {
            (0..self.cols).map(|j| self.get(i, j) * v[j]).sum()
        }).collect()
    }

    /// Frobenius norm: √Σᵢⱼ aᵢⱼ².
    pub fn frobenius_norm(&self) -> f64 {
        self.data.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    /// Spectral radius approximation via power iteration.
    /// Returns the largest eigenvalue magnitude.
    pub fn spectral_radius(&self, max_iter: usize) -> f64 {
        assert_eq!(self.rows, self.cols, "spectral_radius requires square matrix");
        let n = self.rows;
        let mut v: Vec<f64> = vec![1.0 / (n as f64).sqrt(); n];
        let mut lambda = 0.0f64;

        for _ in 0..max_iter {
            let w = self.mul_vec(&v);
            let norm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < ABZU_PRECISION { break; }
            lambda = norm;
            v = w.iter().map(|x| x / norm).collect();
        }
        lambda
    }

    /// Eigenvalues of a 2×2 matrix (exact closed form).
    pub fn eigenvalues_2x2(&self) -> [Eigenvalue; 2] {
        assert_eq!(self.rows, 2);
        assert_eq!(self.cols, 2);
        let a = self.get(0, 0);
        let b = self.get(0, 1);
        let c = self.get(1, 0);
        let d = self.get(1, 1);
        let trace = a + d;
        let det   = a * d - b * c;
        let disc  = trace * trace - 4.0 * det;

        if disc >= 0.0 {
            let sq = disc.sqrt();
            [Eigenvalue::real((trace + sq) / 2.0),
             Eigenvalue::real((trace - sq) / 2.0)]
        } else {
            let sq = (-disc).sqrt();
            [Eigenvalue { real: trace / 2.0, imag:  sq / 2.0 },
             Eigenvalue { real: trace / 2.0, imag: -sq / 2.0 }]
        }
    }

    /// Determinant via LU decomposition (for square matrices up to 7×7).
    pub fn determinant(&self) -> f64 {
        assert_eq!(self.rows, self.cols);
        let n = self.rows;
        let mut a = self.data.clone();
        let mut det = 1.0f64;

        for col in 0..n {
            // Find pivot
            let mut pivot = col;
            for row in col+1..n {
                if a[row*n+col].abs() > a[pivot*n+col].abs() { pivot = row; }
            }
            if pivot != col {
                for k in 0..n { a.swap(col*n+k, pivot*n+k); }
                det *= -1.0;
            }
            if a[col*n+col].abs() < ABZU_PRECISION { return 0.0; }
            det *= a[col*n+col];
            for row in col+1..n {
                let factor = a[row*n+col] / a[col*n+col];
                for k in col..n { a[row*n+k] -= factor * a[col*n+k]; }
            }
        }
        det
    }

    /// Check if matrix is symmetric (A = Aᵀ).
    pub fn is_symmetric(&self) -> bool {
        if self.rows != self.cols { return false; }
        for i in 0..self.rows {
            for j in 0..self.cols {
                if (self.get(i, j) - self.get(j, i)).abs() > ABZU_PRECISION {
                    return false;
                }
            }
        }
        true
    }

    /// Transpose.
    pub fn transpose(&self) -> SovereignMatrix {
        let mut m = SovereignMatrix::new(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols { m.set(j, i, self.get(i, j)); }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_trace_equals_n() {
        let m = SovereignMatrix::identity(5);
        assert!((m.trace() - 5.0).abs() < ABZU_PRECISION);
    }

    #[test]
    fn identity_determinant_is_one() {
        let m = SovereignMatrix::identity(3);
        assert!((m.determinant() - 1.0).abs() < ABZU_PRECISION);
    }

    #[test]
    fn matrix_mul_identity() {
        let a = SovereignMatrix::from_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let i = SovereignMatrix::identity(2);
        let r = a.mul(&i);
        assert!((r.get(0,0) - 1.0).abs() < ABZU_PRECISION);
        assert!((r.get(1,1) - 4.0).abs() < ABZU_PRECISION);
    }

    #[test]
    fn eigenvalues_2x2_identity() {
        let m = SovereignMatrix::identity(2);
        let ev = m.eigenvalues_2x2();
        assert!((ev[0].real - 1.0).abs() < ABZU_PRECISION);
        assert!((ev[1].real - 1.0).abs() < ABZU_PRECISION);
    }

    #[test]
    fn eigenvalues_2x2_complex() {
        // [[0, -1], [1, 0]] has eigenvalues ±i
        let m = SovereignMatrix::from_slice(2, 2, &[0.0, -1.0, 1.0, 0.0]);
        let ev = m.eigenvalues_2x2();
        assert!(ev[0].imag.abs() > 0.5);
    }

    #[test]
    fn spectral_radius_identity() {
        let m = SovereignMatrix::identity(4);
        let sr = m.spectral_radius(100);
        assert!((sr - 1.0).abs() < 0.01, "spectral radius of identity ≈ 1.0, got {sr}");
    }

    #[test]
    fn transpose_of_transpose_is_identity() {
        let m = SovereignMatrix::from_slice(2, 3, &[1.0,2.0,3.0,4.0,5.0,6.0]);
        let tt = m.transpose().transpose();
        for i in 0..2 {
            for j in 0..3 {
                assert!((m.get(i,j) - tt.get(i,j)).abs() < ABZU_PRECISION);
            }
        }
    }

    #[test]
    fn symmetric_detection() {
        let sym = SovereignMatrix::from_slice(2, 2, &[1.0, 2.0, 2.0, 3.0]);
        assert!(sym.is_symmetric());
        let asym = SovereignMatrix::from_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        assert!(!asym.is_symmetric());
    }
}
