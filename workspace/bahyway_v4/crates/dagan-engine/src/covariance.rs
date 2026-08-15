//! Streaming covariance (Welford's algorithm generalized to a full
//! covariance matrix) + power-iteration PCA with Hotelling deflation, and
//! the 7D->3D projection this feeds. `explained_variance_ratio` is exactly
//! the "projection fidelity" gauge from the original review: how honestly
//! the 3D stage represents 7D truth.

use crate::D;

pub type Mat7 = [[f64; D]; D];
pub type Vec7 = [f64; D];

#[derive(Debug, Clone, Copy)]
pub struct StreamingCovariance {
    count: u64,
    mean: Vec7,
    /// Running sum of outer-product deviations; sample covariance is this
    /// divided by (count - 1).
    c: Mat7,
}

impl Default for StreamingCovariance {
    fn default() -> Self {
        StreamingCovariance { count: 0, mean: [0.0; D], c: [[0.0; D]; D] }
    }
}

impl StreamingCovariance {
    pub fn new() -> Self {
        Self::default()
    }

    /// Online covariance update (Welford's algorithm generalized to the
    /// full matrix, not just the diagonal): delta uses the pre-update
    /// mean, delta2 uses the post-update mean, and every C[i][j] +=
    /// delta[i] * delta2[j] -- no special-casing the diagonal.
    pub fn update(&mut self, x: &Vec7) {
        self.count += 1;
        let n = self.count as f64;
        let mut delta = [0.0; D];
        for i in 0..D {
            delta[i] = x[i] - self.mean[i];
            self.mean[i] += delta[i] / n;
        }
        let mut delta2 = [0.0; D];
        for j in 0..D {
            delta2[j] = x[j] - self.mean[j];
        }
        for i in 0..D {
            for j in 0..D {
                self.c[i][j] += delta[i] * delta2[j];
            }
        }
    }

    pub fn mean(&self) -> Vec7 {
        self.mean
    }

    /// Sample covariance matrix (n-1 divisor). Zero matrix for count < 2.
    pub fn covariance(&self) -> Mat7 {
        let mut out = [[0.0; D]; D];
        if self.count < 2 {
            return out;
        }
        let denom = (self.count - 1) as f64;
        for i in 0..D {
            for j in 0..D {
                out[i][j] = self.c[i][j] / denom;
            }
        }
        out
    }
}

fn mat_vec(m: &Mat7, v: &Vec7) -> Vec7 {
    let mut out = [0.0; D];
    for i in 0..D {
        let mut s = 0.0;
        for j in 0..D {
            s += m[i][j] * v[j];
        }
        out[i] = s;
    }
    out
}

fn norm(v: &Vec7) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

fn normalize(v: &mut Vec7) {
    let n = norm(v);
    if n > 1e-12 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

fn dot(a: &Vec7, b: &Vec7) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Power iteration for the dominant eigenvector/eigenvalue of a symmetric
/// matrix. Returns (unit eigenvector, Rayleigh-quotient eigenvalue).
fn power_iteration(m: &Mat7, iters: usize) -> (Vec7, f64) {
    let mut v: Vec7 = [1.0; D];
    normalize(&mut v);
    for _ in 0..iters {
        let mut mv = mat_vec(m, &v);
        normalize(&mut mv);
        v = mv;
    }
    let mv = mat_vec(m, &v);
    let eigenvalue = dot(&v, &mv);
    (v, eigenvalue)
}

/// Hotelling deflation: removes the found eigenpair's contribution so the
/// next power iteration converges to the next-dominant eigenvector.
fn deflate(m: &Mat7, v: &Vec7, eigenvalue: f64) -> Mat7 {
    let mut out = *m;
    for i in 0..D {
        for j in 0..D {
            out[i][j] -= eigenvalue * v[i] * v[j];
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct PcaResult {
    /// Top-3 unit eigenvectors, dominant first.
    pub components: [Vec7; 3],
    pub eigenvalues: [f64; 3],
    /// (sum of top-3 eigenvalues) / trace(covariance) -- the projection
    /// fidelity gauge: how honestly a 3D stage represents 7D truth.
    pub explained_variance_ratio: f64,
}

/// Top-3 principal components of a symmetric covariance matrix via power
/// iteration + Hotelling deflation. `iters` controls convergence (30 is
/// generous for a 7x7 matrix).
pub fn pca_top3(cov: &Mat7, iters: usize) -> PcaResult {
    let trace: f64 = (0..D).map(|i| cov[i][i]).sum();

    let (v1, l1) = power_iteration(cov, iters);
    let m2 = deflate(cov, &v1, l1);
    let (v2, l2) = power_iteration(&m2, iters);
    let m3 = deflate(&m2, &v2, l2);
    let (v3, l3) = power_iteration(&m3, iters);

    let explained = if trace.abs() > 1e-12 { (l1 + l2 + l3) / trace } else { 0.0 };

    PcaResult { components: [v1, v2, v3], eigenvalues: [l1, l2, l3], explained_variance_ratio: explained }
}

/// Project a 7D point (relative to the fitted mean) onto the top-3
/// principal components.
pub fn project_3d(point: &Vec7, mean: &Vec7, pca: &PcaResult) -> [f64; 3] {
    let mut centered = [0.0; D];
    for i in 0..D {
        centered[i] = point[i] - mean[i];
    }
    [dot(&centered, &pca.components[0]), dot(&centered, &pca.components[1]), dot(&centered, &pca.components[2])]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covariance_matches_direct_calculation_for_two_correlated_dims() {
        // x and y perfectly correlated (y = 2x); rest of dims constant.
        let xs = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
        let mut sc = StreamingCovariance::new();
        for &x in &xs {
            let mut v = [0.0; D];
            v[0] = x;
            v[1] = 2.0 * x;
            sc.update(&v);
        }
        let cov = sc.covariance();
        let n = xs.len() as f64;
        let mx = xs.iter().sum::<f64>() / n;
        let my = xs.iter().map(|x| 2.0 * x).sum::<f64>() / n;
        let cov01_direct: f64 = xs.iter().map(|x| (x - mx) * (2.0 * x - my)).sum::<f64>() / (n - 1.0);
        assert!((cov[0][1] - cov01_direct).abs() < 1e-9);
        // Symmetric by construction.
        assert!((cov[0][1] - cov[1][0]).abs() < 1e-12);
    }

    #[test]
    fn pca_on_a_single_dominant_axis_recovers_it() {
        // All variance lives on dim 0 -- a diagonal covariance with a huge
        // dim-0 entry. The dominant eigenvector must point along dim 0.
        let mut cov: Mat7 = [[0.0; D]; D];
        cov[0][0] = 100.0;
        for i in 1..D {
            cov[i][i] = 0.01; // tiny, non-zero to keep the matrix well-posed
        }
        let pca = pca_top3(&cov, 50);
        // Dominant component should be (approximately) e0, up to sign.
        assert!(pca.components[0][0].abs() > 0.99, "v1={:?}", pca.components[0]);
        assert!((pca.eigenvalues[0] - 100.0).abs() < 1e-6);
        // Nearly all variance is explained by 3 components when only one
        // dimension actually carries meaningful variance.
        assert!(pca.explained_variance_ratio > 0.99);
    }

    #[test]
    fn projection_of_the_mean_itself_is_the_origin() {
        let mut cov: Mat7 = [[0.0; D]; D];
        for i in 0..D {
            cov[i][i] = 1.0;
        }
        let pca = pca_top3(&cov, 30);
        let mean: Vec7 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let projected = project_3d(&mean, &mean, &pca);
        for c in projected {
            assert!(c.abs() < 1e-9);
        }
    }
}
