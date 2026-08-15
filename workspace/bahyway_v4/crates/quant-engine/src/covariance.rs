//! Cross-asset covariance and correlation matrix.
//!
//! Uses the sample covariance formula (divides by n-1 for unbiased estimate).
//! Pure sovereign Rust — no BLAS, no ndarray.

/// Covariance matrix for N return series.
///
/// `cov[i][j]` = sample covariance between series i and series j.
/// Diagonal `cov[i][i]` = sample variance of series i.
#[derive(Debug, Clone)]
pub struct CovMatrix {
    pub n_assets: usize,
    /// Flat row-major storage: data[i * n_assets + j] = cov(i, j).
    pub data:     Vec<f64>,
    pub asset_hashes: Vec<u32>,
}

impl CovMatrix {
    /// Build a covariance matrix from a slice of return series.
    ///
    /// Each inner Vec is one asset's return observations.
    /// All series must have the same length.
    pub fn from_return_series(series: &[(u32, Vec<f64>)]) -> Self {
        let n = series.len();
        let mut data = vec![0.0f64; n * n];
        let asset_hashes: Vec<u32> = series.iter().map(|(h, _)| *h).collect();

        for i in 0..n {
            for j in i..n {
                let cov = sample_covariance(&series[i].1, &series[j].1);
                data[i * n + j] = cov;
                data[j * n + i] = cov; // symmetric
            }
        }
        CovMatrix { n_assets: n, data, asset_hashes }
    }

    /// Covariance between asset at index `i` and asset at index `j`.
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.data[i * self.n_assets + j]
    }

    /// Variance of asset at index `i` (diagonal element).
    pub fn variance(&self, i: usize) -> f64 { self.get(i, i) }

    /// Standard deviation of asset at index `i`.
    pub fn std_dev(&self, i: usize) -> f64 { self.variance(i).sqrt() }

    /// Beta of asset `i` relative to benchmark asset `b`.
    /// β = cov(i, b) / var(b)
    pub fn beta(&self, i: usize, b: usize) -> f64 {
        let var_b = self.variance(b);
        if var_b.abs() < 1e-15 { return 0.0; }
        self.get(i, b) / var_b
    }
}

/// Correlation matrix derived from a covariance matrix.
///
/// `cor[i][j]` = cov(i,j) / (std_i × std_j), in [-1, +1].
#[derive(Debug, Clone)]
pub struct CorrelationMatrix {
    pub n_assets: usize,
    pub data:     Vec<f64>,
    pub asset_hashes: Vec<u32>,
}

impl CorrelationMatrix {
    /// Build from a covariance matrix.
    pub fn from_cov(cov: &CovMatrix) -> Self {
        let n = cov.n_assets;
        let mut data = vec![0.0f64; n * n];
        for i in 0..n {
            for j in 0..n {
                let std_i = cov.std_dev(i);
                let std_j = cov.std_dev(j);
                data[i * n + j] = if std_i * std_j < 1e-15 {
                    if i == j { 1.0 } else { 0.0 }
                } else {
                    cov.get(i, j) / (std_i * std_j)
                };
            }
        }
        CorrelationMatrix { n_assets: n, data, asset_hashes: cov.asset_hashes.clone() }
    }

    /// Correlation between asset at index `i` and asset at index `j`.
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.data[i * self.n_assets + j]
    }

    /// True if two assets are highly correlated (|cor| > threshold).
    pub fn is_correlated(&self, i: usize, j: usize, threshold: f64) -> bool {
        self.get(i, j).abs() > threshold
    }
}

/// Sample covariance: Σ((x_i − x̄)(y_i − ȳ)) / (n − 1).
pub fn sample_covariance(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 { return 0.0; }
    let xm = x[..n].iter().sum::<f64>() / n as f64;
    let ym = y[..n].iter().sum::<f64>() / n as f64;
    x[..n].iter().zip(y[..n].iter())
        .map(|(xi, yi)| (xi - xm) * (yi - ym))
        .sum::<f64>() / (n as f64 - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series_a() -> Vec<f64> { vec![0.01, 0.02, -0.01, 0.03, -0.02] }
    fn series_b() -> Vec<f64> { vec![0.02, 0.01, -0.02, 0.04, -0.01] }
    fn series_c() -> Vec<f64> { vec![-0.01, -0.02, 0.01, -0.03, 0.02] }

    #[test]
    fn sample_cov_self_equals_sample_variance() {
        let a = series_a();
        let cov_aa = sample_covariance(&a, &a);
        let n = a.len() as f64;
        let mean = a.iter().sum::<f64>() / n;
        let var = a.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        assert!((cov_aa - var).abs() < 1e-12, "cov_aa={cov_aa} var={var}");
    }

    #[test]
    fn covariance_symmetric() {
        let a = series_a();
        let b = series_b();
        assert!((sample_covariance(&a, &b) - sample_covariance(&b, &a)).abs() < 1e-12);
    }

    #[test]
    fn cov_matrix_diagonal_is_variance() {
        let input = vec![
            (0x01u32, series_a()),
            (0x02u32, series_b()),
        ];
        let cm = CovMatrix::from_return_series(&input);
        let var_a = sample_covariance(&series_a(), &series_a());
        assert!((cm.variance(0) - var_a).abs() < 1e-12);
    }

    #[test]
    fn cov_matrix_is_symmetric() {
        let input = vec![(0x01u32, series_a()), (0x02u32, series_b())];
        let cm = CovMatrix::from_return_series(&input);
        assert!((cm.get(0, 1) - cm.get(1, 0)).abs() < 1e-12);
    }

    #[test]
    fn correlation_self_is_one() {
        let input = vec![(0x01u32, series_a()), (0x02u32, series_b())];
        let cm  = CovMatrix::from_return_series(&input);
        let cor = CorrelationMatrix::from_cov(&cm);
        assert!((cor.get(0, 0) - 1.0).abs() < 1e-9);
        assert!((cor.get(1, 1) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn negative_correlation_detected() {
        // series_a and series_c are mirror images → correlation ≈ -1
        let input = vec![(0x01u32, series_a()), (0x03u32, series_c())];
        let cm  = CovMatrix::from_return_series(&input);
        let cor = CorrelationMatrix::from_cov(&cm);
        assert!(cor.get(0, 1) < -0.95, "cor={}", cor.get(0, 1));
    }

    #[test]
    fn beta_self_is_one() {
        let input = vec![(0x01u32, series_a()), (0x02u32, series_b())];
        let cm = CovMatrix::from_return_series(&input);
        let beta_aa = cm.beta(0, 0);
        assert!((beta_aa - 1.0).abs() < 1e-9, "beta_aa={beta_aa}");
    }

    #[test]
    fn single_asset_cov_matrix_is_scalar() {
        let input = vec![(0xABu32, series_a())];
        let cm = CovMatrix::from_return_series(&input);
        assert_eq!(cm.n_assets, 1);
        assert!(cm.variance(0) > 0.0);
    }
}
