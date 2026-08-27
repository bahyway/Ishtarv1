//! ReturnSeries — log-return and simple-return computation from price series.
//!
//! Log returns: r_t = ln(P_t / P_{t-1})
//! Simple returns: r_t = (P_t - P_{t-1}) / P_{t-1}
//!
//! Prices are received as i64 fixed-point from enkidb-quantdb and converted
//! to f64 real prices before return computation.

/// Type of return calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnKind {
    /// r_t = ln(P_t / P_{t-1}) — additive, normally distributed (preferred)
    Log,
    /// r_t = (P_t - P_{t-1}) / P_{t-1} — simple percentage change
    Simple,
}

/// A computed return series derived from a price series.
#[derive(Debug, Clone)]
pub struct ReturnSeries {
    asset_hash: u32,
    kind: ReturnKind,
    returns: Vec<f64>,
    /// Mean return (pre-computed for efficiency).
    mean: f64,
    /// Standard deviation of returns (population).
    std_dev: f64,
}

impl ReturnSeries {
    /// Build a `ReturnSeries` from fixed-point prices.
    ///
    /// - `prices`      — raw fixed-point prices from `QuantDbStore`
    /// - `price_scale` — divisor to convert raw → real (e.g. 100 for cents)
    /// - Uses log returns by default.
    pub fn from_prices(prices: &[i64], price_scale: u32) -> Self {
        Self::from_prices_with_kind(0, prices, price_scale, ReturnKind::Log)
    }

    /// Build with a specific asset hash and return kind.
    pub fn from_prices_with_kind(
        asset_hash: u32,
        prices: &[i64],
        price_scale: u32,
        kind: ReturnKind,
    ) -> Self {
        let scale = price_scale as f64;
        let reals: Vec<f64> = prices.iter().map(|&p| p as f64 / scale).collect();
        let returns = compute_returns(&reals, kind);
        let (mean, std_dev) = mean_and_std(&returns);
        Self {
            asset_hash,
            kind,
            returns,
            mean,
            std_dev,
        }
    }

    /// Number of return observations.
    pub fn len(&self) -> usize {
        self.returns.len()
    }
    pub fn is_empty(&self) -> bool {
        self.returns.is_empty()
    }

    /// The raw return values.
    pub fn log_returns(&self) -> &[f64] {
        &self.returns
    }

    /// Mean return.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Population standard deviation of returns (volatility).
    pub fn std_dev(&self) -> f64 {
        self.std_dev
    }

    /// Annualised volatility assuming `periods_per_year` observations per year.
    pub fn annualised_volatility(&self, periods_per_year: f64) -> f64 {
        self.std_dev * periods_per_year.sqrt()
    }

    /// Cumulative return from first to last observation.
    ///
    /// For log returns: sum of all log returns = ln(P_last / P_first).
    /// For simple returns: product of (1 + r_t) − 1.
    pub fn cumulative_return(&self) -> f64 {
        match self.kind {
            ReturnKind::Log => self.returns.iter().sum(),
            ReturnKind::Simple => self.returns.iter().fold(1.0, |acc, &r| acc * (1.0 + r)) - 1.0,
        }
    }

    /// Maximum single-period return.
    pub fn max_return(&self) -> f64 {
        self.returns
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Minimum single-period return (worst single period).
    pub fn min_return(&self) -> f64 {
        self.returns.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// Positive return count.
    pub fn up_count(&self) -> usize {
        self.returns.iter().filter(|&&r| r > 0.0).count()
    }

    /// Negative return count.
    pub fn down_count(&self) -> usize {
        self.returns.iter().filter(|&&r| r < 0.0).count()
    }

    /// Win rate: fraction of periods with positive return.
    pub fn win_rate(&self) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }
        self.up_count() as f64 / self.returns.len() as f64
    }

    pub fn asset_hash(&self) -> u32 {
        self.asset_hash
    }
    pub fn kind(&self) -> ReturnKind {
        self.kind
    }
}

fn compute_returns(reals: &[f64], kind: ReturnKind) -> Vec<f64> {
    if reals.len() < 2 {
        return Vec::new();
    }
    reals
        .windows(2)
        .map(|w| {
            let (prev, curr) = (w[0], w[1]);
            match kind {
                ReturnKind::Log => {
                    if prev > 0.0 {
                        (curr / prev).ln()
                    } else {
                        0.0
                    }
                }
                ReturnKind::Simple => {
                    if prev > 0.0 {
                        (curr - prev) / prev
                    } else {
                        0.0
                    }
                }
            }
        })
        .collect()
}

pub fn mean_and_std(values: &[f64]) -> (f64, f64) {
    let n = values.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    (mean, variance.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prices() -> Vec<i64> {
        // 100.00, 102.00, 101.00, 104.00, 103.00  (scale=100)
        vec![10000, 10200, 10100, 10400, 10300]
    }

    #[test]
    fn return_count_is_n_minus_one() {
        let r = ReturnSeries::from_prices(&prices(), 100);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn log_returns_sum_to_cumulative() {
        let r = ReturnSeries::from_prices(&prices(), 100);
        let sum: f64 = r.log_returns().iter().sum();
        let expected = (103.0f64 / 100.0).ln();
        assert!(
            (sum - expected).abs() < 1e-9,
            "sum={sum} expected={expected}"
        );
    }

    #[test]
    fn simple_cumulative_return_correct() {
        let r = ReturnSeries::from_prices_with_kind(0, &prices(), 100, ReturnKind::Simple);
        let cum = r.cumulative_return();
        let expected = (103.0 - 100.0) / 100.0;
        assert!(
            (cum - expected).abs() < 1e-9,
            "cum={cum} expected={expected}"
        );
    }

    #[test]
    fn std_dev_is_nonnegative() {
        let r = ReturnSeries::from_prices(&prices(), 100);
        assert!(r.std_dev() >= 0.0);
    }

    #[test]
    fn flat_prices_give_zero_std_dev() {
        let flat = vec![10000i64; 10];
        let r = ReturnSeries::from_prices(&flat, 100);
        assert!(r.std_dev() < 1e-12);
        assert!((r.cumulative_return()).abs() < 1e-12);
    }

    #[test]
    fn win_rate_monotonic_prices() {
        let up: Vec<i64> = (1..=10).map(|i| i * 1000).collect();
        let r = ReturnSeries::from_prices(&up, 100);
        assert!((r.win_rate() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn max_min_return_correct() {
        let r = ReturnSeries::from_prices(&prices(), 100);
        assert!(r.max_return() > r.min_return());
    }

    #[test]
    fn annualised_volatility_scales_correctly() {
        let r = ReturnSeries::from_prices(&prices(), 100);
        let daily_std = r.std_dev();
        let annual_std = r.annualised_volatility(252.0);
        assert!((annual_std - daily_std * 252f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn empty_price_series_gives_empty_returns() {
        let r = ReturnSeries::from_prices(&[], 100);
        assert!(r.is_empty());
        assert_eq!(r.cumulative_return(), 0.0);
    }

    #[test]
    fn single_price_gives_empty_returns() {
        let r = ReturnSeries::from_prices(&[10000], 100);
        assert!(r.is_empty());
    }
}
