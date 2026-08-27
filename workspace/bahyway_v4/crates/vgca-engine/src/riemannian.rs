//! Riemannian differential geometry — ALGEBRA_GLOSSARY.md Part I/II,
//! items 9, 14, 15, 16 (Manifold, Geodesic, Covariant Derivative,
//! Riemannian Curvature). Confirmed genuinely absent before this file:
//! no hit for Manifold/Geodesic/Curvature/CovariantDerivative anywhere
//! in the workspace.
//!
//! Implemented generally, over any metric tensor field g_ij(x)
//! (numerically differentiated), not hardcoded to BahyWay's flat Nabu
//! metric g = diag(w1..w7) -- a hardcoded-flat implementation would make
//! "Riemannian Curvature" trivially zero everywhere and prove nothing.
//! Correctness is checked two ways in the tests below:
//!   1. On BahyWay's actual metric (constant diagonal): curvature must
//!      numerically vanish -- this is the real, documented case.
//!   2. On a round 2-sphere of radius r in (theta, phi) coordinates
//!      (g = diag(r^2, r^2 sin^2(theta))): Gaussian curvature must
//!      numerically match the known analytic result K = 1/r^2. This is
//!      the independent check that the general machinery is actually
//!      correct, not just "compiles and returns zero."
//!
//! Method: finite-difference metric derivatives -> Christoffel symbols
//! of the second kind -> geodesic ODE (RK4) / Riemann curvature tensor
//! -> Ricci tensor / scalar -> Gaussian curvature (2D case, K = R/2).

/// Signature of a metric tensor field g_ij(x): a point x maps to its n x n metric matrix.
type MetricFn = Box<dyn Fn(&[f64]) -> Vec<Vec<f64>>>;

/// A Riemannian manifold given by a metric tensor field g_ij(x), n x n,
/// symmetric positive-definite at every x in its domain.
pub struct RiemannianManifold {
    pub dim: usize,
    metric_fn: MetricFn,
    h: f64, // finite-difference step
}

impl RiemannianManifold {
    pub fn new(dim: usize, metric_fn: impl Fn(&[f64]) -> Vec<Vec<f64>> + 'static) -> Self {
        RiemannianManifold {
            dim,
            metric_fn: Box::new(metric_fn),
            h: 1e-4,
        }
    }

    pub fn metric(&self, x: &[f64]) -> Vec<Vec<f64>> {
        (self.metric_fn)(x)
    }

    /// Inverse of an n x n matrix via Gauss-Jordan elimination.
    fn invert(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = m.len();
        let mut a: Vec<Vec<f64>> = m.to_vec();
        let mut inv: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect();

        for col in 0..n {
            // partial pivot
            let mut pivot = col;
            for r in (col + 1)..n {
                if a[r][col].abs() > a[pivot][col].abs() {
                    pivot = r;
                }
            }
            a.swap(col, pivot);
            inv.swap(col, pivot);

            let d = a[col][col];
            for j in 0..n {
                a[col][j] /= d;
                inv[col][j] /= d;
            }

            for r in 0..n {
                if r == col {
                    continue;
                }
                let factor = a[r][col];
                for j in 0..n {
                    a[r][j] -= factor * a[col][j];
                    inv[r][j] -= factor * inv[col][j];
                }
            }
        }
        inv
    }

    /// dg_ij/dx_k at point x, via central finite differences.
    fn metric_derivative(&self, x: &[f64], k: usize) -> Vec<Vec<f64>> {
        let mut xp = x.to_vec();
        let mut xm = x.to_vec();
        xp[k] += self.h;
        xm[k] -= self.h;
        let gp = self.metric(&xp);
        let gm = self.metric(&xm);
        let n = self.dim;
        let mut d = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                d[i][j] = (gp[i][j] - gm[i][j]) / (2.0 * self.h);
            }
        }
        d
    }

    /// Christoffel symbols of the second kind Gamma^k_ij at point x.
    /// Gamma^k_ij = 1/2 g^kl (d_i g_jl + d_j g_il - d_l g_ij)
    pub fn christoffel(&self, x: &[f64]) -> Vec<Vec<Vec<f64>>> {
        let n = self.dim;
        let g_inv = Self::invert(&self.metric(x));
        let dg: Vec<Vec<Vec<f64>>> = (0..n).map(|k| self.metric_derivative(x, k)).collect();

        let mut gamma = vec![vec![vec![0.0; n]; n]; n]; // gamma[k][i][j]
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    let mut sum = 0.0;
                    for l in 0..n {
                        sum += g_inv[k][l] * (dg[i][j][l] + dg[j][i][l] - dg[l][i][j]);
                    }
                    gamma[k][i][j] = 0.5 * sum;
                }
            }
        }
        gamma
    }

    /// Covariant derivative of a vector field V (given componentwise as
    /// closures) at point x, in direction index `along`:
    /// (nabla_along V)^k = d_along V^k + Gamma^k_{along j} V^j(x)
    pub fn covariant_derivative(
        &self,
        v: &dyn Fn(&[f64]) -> Vec<f64>,
        x: &[f64],
        along: usize,
    ) -> Vec<f64> {
        let n = self.dim;
        let v0 = v(x);
        let mut xp = x.to_vec();
        let mut xm = x.to_vec();
        xp[along] += self.h;
        xm[along] -= self.h;
        let vp = v(&xp);
        let vm = v(&xm);
        let gamma = self.christoffel(x);

        (0..n)
            .map(|k| {
                let dvk = (vp[k] - vm[k]) / (2.0 * self.h);
                let mut correction = 0.0;
                for j in 0..n {
                    correction += gamma[k][along][j] * v0[j];
                }
                dvk + correction
            })
            .collect()
    }

    /// Geodesic ODE: d^2x^k/dt^2 = -Gamma^k_ij dx^i/dt dx^j/dt.
    /// Integrates via RK4, `steps` steps of size `dt`, from (x0, v0).
    /// Returns the trajectory (positions only).
    pub fn geodesic(&self, x0: &[f64], v0: &[f64], dt: f64, steps: usize) -> Vec<Vec<f64>> {
        let n = self.dim;
        let mut x = x0.to_vec();
        let mut v = v0.to_vec();
        let mut traj = vec![x.clone()];

        let accel = |x: &[f64], v: &[f64]| -> Vec<f64> {
            let gamma = self.christoffel(x);
            (0..n)
                .map(|k| {
                    let mut a = 0.0;
                    for i in 0..n {
                        for j in 0..n {
                            a -= gamma[k][i][j] * v[i] * v[j];
                        }
                    }
                    a
                })
                .collect()
        };

        for _ in 0..steps {
            // RK4 on the state (x, v)
            let k1v = accel(&x, &v);
            let k1x = v.clone();

            let x2: Vec<f64> = (0..n).map(|i| x[i] + 0.5 * dt * k1x[i]).collect();
            let v2: Vec<f64> = (0..n).map(|i| v[i] + 0.5 * dt * k1v[i]).collect();
            let k2v = accel(&x2, &v2);
            let k2x = v2.clone();

            let x3: Vec<f64> = (0..n).map(|i| x[i] + 0.5 * dt * k2x[i]).collect();
            let v3: Vec<f64> = (0..n).map(|i| v[i] + 0.5 * dt * k2v[i]).collect();
            let k3v = accel(&x3, &v3);
            let k3x = v3.clone();

            let x4: Vec<f64> = (0..n).map(|i| x[i] + dt * k3x[i]).collect();
            let v4: Vec<f64> = (0..n).map(|i| v[i] + dt * k3v[i]).collect();
            let k4v = accel(&x4, &v4);
            let k4x = v4.clone();

            for i in 0..n {
                x[i] += dt / 6.0 * (k1x[i] + 2.0 * k2x[i] + 2.0 * k3x[i] + k4x[i]);
                v[i] += dt / 6.0 * (k1v[i] + 2.0 * k2v[i] + 2.0 * k3v[i] + k4v[i]);
            }
            traj.push(x.clone());
        }
        traj
    }

    /// Riemann curvature tensor R^l_{ijk} at point x, via finite
    /// differences of the Christoffel symbols:
    /// R^l_{ijk} = d_i Gamma^l_{jk} - d_j Gamma^l_{ik}
    ///           + Gamma^l_{im} Gamma^m_{jk} - Gamma^l_{jm} Gamma^m_{ik}
    fn riemann_tensor(&self, x: &[f64]) -> Vec<Vec<Vec<Vec<f64>>>> {
        let n = self.dim;
        let gamma = self.christoffel(x);

        let d_gamma = |dim_idx: usize| -> Vec<Vec<Vec<f64>>> {
            let mut xp = x.to_vec();
            let mut xm = x.to_vec();
            xp[dim_idx] += self.h;
            xm[dim_idx] -= self.h;
            let gp = self.christoffel(&xp);
            let gm = self.christoffel(&xm);
            let mut d = vec![vec![vec![0.0; n]; n]; n];
            for l in 0..n {
                for a in 0..n {
                    for b in 0..n {
                        d[l][a][b] = (gp[l][a][b] - gm[l][a][b]) / (2.0 * self.h);
                    }
                }
            }
            d
        };
        let dgamma: Vec<Vec<Vec<Vec<f64>>>> = (0..n).map(d_gamma).collect();

        let mut r = vec![vec![vec![vec![0.0; n]; n]; n]; n]; // r[l][i][j][k]
        for l in 0..n {
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        // Sign fixed empirically against the known-analytic
                        // round-sphere case (K = 1/r^2): the raw textbook
                        // formula R^l_ijk = d_i G^l_jk - d_j G^l_ik + GG - GG
                        // produced the correct magnitude but opposite sign
                        // under this contraction/index convention, so the
                        // tensor is negated here to match the verified case.
                        let mut val = dgamma[i][l][j][k] - dgamma[j][l][i][k];
                        // m indexes gamma at two different tensor slots in the
                        // same expression (contracted index), not a simple
                        // parallel walk -- an enumerate()-based rewrite would
                        // only cover one slot and risk a silent indexing bug
                        // in this contraction. Loop kept explicit.
                        #[allow(clippy::needless_range_loop)]
                        for m in 0..n {
                            val += gamma[l][i][m] * gamma[m][j][k];
                            val -= gamma[l][j][m] * gamma[m][i][k];
                        }
                        r[l][i][j][k] = -val;
                    }
                }
            }
        }
        r
    }

    /// Ricci tensor R_ij = R^k_{ikj} (contraction of the Riemann tensor).
    pub fn ricci_tensor(&self, x: &[f64]) -> Vec<Vec<f64>> {
        let n = self.dim;
        let riem = self.riemann_tensor(x);
        let mut ric = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                // k indexes riem at two different tensor slots (the Ricci
                // contraction R_ij = R^k_ikj) -- same reasoning as
                // riemann_tensor's own m-loop above: kept explicit rather
                // than an enumerate() that would only cover one slot.
                #[allow(clippy::needless_range_loop)]
                for k in 0..n {
                    sum += riem[k][i][k][j];
                }
                ric[i][j] = sum;
            }
        }
        ric
    }

    /// Ricci scalar R = g^ij R_ij.
    pub fn ricci_scalar(&self, x: &[f64]) -> f64 {
        let g_inv = Self::invert(&self.metric(x));
        let ric = self.ricci_tensor(x);
        let n = self.dim;
        let mut r = 0.0;
        for i in 0..n {
            for j in 0..n {
                r += g_inv[i][j] * ric[i][j];
            }
        }
        r
    }

    /// Gaussian curvature K, valid only for dim == 2 (K = R/2).
    pub fn gaussian_curvature_2d(&self, x: &[f64]) -> f64 {
        assert_eq!(self.dim, 2, "Gaussian curvature is a 2D-only concept");
        self.ricci_scalar(x) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BahyWay's actual, documented metric: g = diag(w1..w7), the Nabu
    /// metric (GL-MRD-002). Constant -> flat -> zero curvature everywhere.
    fn nabu_metric_7d(_x: &[f64]) -> Vec<Vec<f64>> {
        let w = [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05];
        let mut g = vec![vec![0.0; 7]; 7];
        for i in 0..7 {
            g[i][i] = w[i];
        }
        g
    }

    #[test]
    fn bahyway_nabu_metric_has_zero_christoffel_symbols() {
        let m = RiemannianManifold::new(7, nabu_metric_7d);
        let x = vec![0.5; 7];
        let gamma = m.christoffel(&x);
        for k in 0..7 {
            for i in 0..7 {
                for j in 0..7 {
                    assert!(
                        gamma[k][i][j].abs() < 1e-6,
                        "constant diagonal metric must have Gamma=0, got {}",
                        gamma[k][i][j]
                    );
                }
            }
        }
    }

    #[test]
    fn bahyway_nabu_metric_geodesics_are_straight_lines() {
        // Flat metric -> no curving -> the geodesic must match x0 + v*t exactly.
        let m = RiemannianManifold::new(7, nabu_metric_7d);
        let x0 = vec![0.2; 7];
        let v0 = vec![0.1; 7];
        let traj = m.geodesic(&x0, &v0, 0.1, 5);
        let t = 5.0 * 0.1;
        for i in 0..7 {
            let expected = x0[i] + v0[i] * t;
            assert!(
                (traj.last().unwrap()[i] - expected).abs() < 1e-3,
                "flat-metric geodesic should be a straight line: got {}, expected {}",
                traj.last().unwrap()[i],
                expected
            );
        }
    }

    /// Round 2-sphere of radius r in (theta, phi): ds^2 = r^2 dtheta^2 +
    /// r^2 sin^2(theta) dphi^2. Known analytic Gaussian curvature: K = 1/r^2
    /// everywhere. This is the independent check that the general
    /// machinery (not just the trivially-flat BahyWay case) is correct.
    fn sphere_metric(r: f64) -> impl Fn(&[f64]) -> Vec<Vec<f64>> {
        move |x: &[f64]| {
            let theta = x[0];
            vec![
                vec![r * r, 0.0],
                vec![0.0, r * r * theta.sin() * theta.sin()],
            ]
        }
    }

    #[test]
    fn sphere_gaussian_curvature_matches_known_analytic_value() {
        let r = 2.0;
        let m = RiemannianManifold::new(2, sphere_metric(r));
        // Away from the poles (theta = 0 or pi) where the coordinate
        // system is singular.
        let x = vec![std::f64::consts::FRAC_PI_2, 0.7];
        let k = m.gaussian_curvature_2d(&x);
        let expected = 1.0 / (r * r);
        assert!(
            (k - expected).abs() < 1e-2,
            "sphere of radius {r} should have K=1/r^2={expected}, got {k}"
        );
    }

    #[test]
    fn sphere_curvature_is_positive_everywhere_tested() {
        let r = 1.5;
        let m = RiemannianManifold::new(2, sphere_metric(r));
        for theta_frac in [0.3, 0.5, 0.7, 1.0, 1.5] {
            let x = vec![theta_frac, 0.2];
            let k = m.gaussian_curvature_2d(&x);
            assert!(
                k > 0.0,
                "sphere curvature must be positive, got {k} at theta={theta_frac}"
            );
        }
    }

    #[test]
    fn covariant_derivative_of_constant_field_on_flat_metric_is_zero() {
        let m = RiemannianManifold::new(7, nabu_metric_7d);
        let x = vec![0.3; 7];
        let constant_field = |_x: &[f64]| vec![1.0; 7];
        for along in 0..7 {
            let d = m.covariant_derivative(&constant_field, &x, along);
            for &c in &d {
                assert!(
                    c.abs() < 1e-5,
                    "constant field on flat metric has zero covariant derivative, got {c}"
                );
            }
        }
    }
}
