//! Rolling window analytics — SMA, EMA, Bollinger Bands, rolling stats.
//!
//! All computations are sovereign: no external crates.
//! Window sizes are in number of observations (not epochs).

/// Rolling window statistics over a slice of f64 values.
pub struct RollingWindow {
    values: Vec<f64>,
    period: usize,
}

impl RollingWindow {
    pub fn new(values: Vec<f64>, period: usize) -> Self {
        assert!(period >= 1, "period must be ≥ 1");
        Self { values, period }
    }

    /// Simple Moving Average at each position (returns `None` for early positions).
    pub fn sma(&self) -> Vec<Option<f64>> {
        self.values
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i + 1 < self.period {
                    return None;
                }
                let sum: f64 = self.values[i + 1 - self.period..=i].iter().sum();
                Some(sum / self.period as f64)
            })
            .collect()
    }

    /// Rolling standard deviation (population) at each position.
    pub fn rolling_std(&self) -> Vec<Option<f64>> {
        self.values
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i + 1 < self.period {
                    return None;
                }
                let window = &self.values[i + 1 - self.period..=i];
                let mean = window.iter().sum::<f64>() / self.period as f64;
                let var =
                    window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.period as f64;
                Some(var.sqrt())
            })
            .collect()
    }

    /// Rolling maximum at each position.
    pub fn rolling_max(&self) -> Vec<Option<f64>> {
        self.values
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i + 1 < self.period {
                    return None;
                }
                let m = self.values[i + 1 - self.period..=i]
                    .iter()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max);
                Some(m)
            })
            .collect()
    }

    /// Rolling minimum at each position.
    pub fn rolling_min(&self) -> Vec<Option<f64>> {
        self.values
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i + 1 < self.period {
                    return None;
                }
                let m = self.values[i + 1 - self.period..=i]
                    .iter()
                    .cloned()
                    .fold(f64::INFINITY, f64::min);
                Some(m)
            })
            .collect()
    }

    /// Compute Bollinger Bands (SMA ± n_std × rolling_std).
    pub fn bollinger_bands(&self, n_std: f64) -> BollingerBands {
        let sma = self.sma();
        let std = self.rolling_std();
        let upper: Vec<Option<f64>> = sma
            .iter()
            .zip(std.iter())
            .map(|(s, d)| match (s, d) {
                (Some(m), Some(sd)) => Some(m + n_std * sd),
                _ => None,
            })
            .collect();
        let lower: Vec<Option<f64>> = sma
            .iter()
            .zip(std.iter())
            .map(|(s, d)| match (s, d) {
                (Some(m), Some(sd)) => Some(m - n_std * sd),
                _ => None,
            })
            .collect();
        BollingerBands {
            middle: sma,
            upper,
            lower,
            period: self.period,
            n_std,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    pub fn period(&self) -> usize {
        self.period
    }
}

/// Bollinger Bands result (upper, middle/SMA, lower bands).
#[derive(Debug, Clone)]
pub struct BollingerBands {
    pub middle: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
    pub period: usize,
    pub n_std: f64,
}

impl BollingerBands {
    /// Bandwidth = (upper - lower) / middle.  Measures relative band width.
    pub fn bandwidth(&self) -> Vec<Option<f64>> {
        (0..self.middle.len())
            .map(|i| match (self.upper[i], self.middle[i], self.lower[i]) {
                (Some(u), Some(m), Some(l)) if m.abs() > 1e-12 => Some((u - l) / m),
                _ => None,
            })
            .collect()
    }

    /// %B indicator: where price is relative to bands (0=lower, 1=upper).
    pub fn percent_b(&self, price: f64) -> Vec<Option<f64>> {
        (0..self.upper.len())
            .map(|i| match (self.upper[i], self.lower[i]) {
                (Some(u), Some(l)) if (u - l).abs() > 1e-12 => Some((price - l) / (u - l)),
                _ => None,
            })
            .collect()
    }
}

/// Exponential Moving Average (EMA) — span-based smoothing.
///
/// α = 2 / (span + 1), standard "span" EMA formula.
pub struct ExponentialMovingAverage {
    pub values: Vec<f64>,
    pub span: usize,
}

impl ExponentialMovingAverage {
    pub fn new(values: Vec<f64>, span: usize) -> Self {
        assert!(span >= 1, "span must be ≥ 1");
        Self { values, span }
    }

    /// Compute the EMA series.  First value = first observation (seeded).
    pub fn compute(&self) -> Vec<f64> {
        if self.values.is_empty() {
            return Vec::new();
        }
        let alpha = 2.0 / (self.span as f64 + 1.0);
        let mut ema = Vec::with_capacity(self.values.len());
        let mut prev = self.values[0];
        ema.push(prev);
        for &v in &self.values[1..] {
            prev = alpha * v + (1.0 - alpha) * prev;
            ema.push(prev);
        }
        ema
    }

    /// MACD = EMA(fast_span) − EMA(slow_span).
    /// Returns (macd_line, signal_line) where signal = EMA(macd, signal_span).
    pub fn macd(values: &[f64], fast: usize, slow: usize, signal: usize) -> (Vec<f64>, Vec<f64>) {
        let fast_ema = ExponentialMovingAverage::new(values.to_vec(), fast).compute();
        let slow_ema = ExponentialMovingAverage::new(values.to_vec(), slow).compute();
        let macd_line: Vec<f64> = fast_ema
            .iter()
            .zip(slow_ema.iter())
            .map(|(f, s)| f - s)
            .collect();
        let signal_line = ExponentialMovingAverage::new(macd_line.clone(), signal).compute();
        (macd_line, signal_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prices() -> Vec<f64> {
        vec![10.0, 11.0, 12.0, 11.0, 13.0, 12.0, 14.0, 13.0, 15.0, 14.0]
    }

    #[test]
    fn sma_first_valid_at_period() {
        let rw = RollingWindow::new(prices(), 3);
        let sma = rw.sma();
        assert!(sma[0].is_none());
        assert!(sma[1].is_none());
        assert!(sma[2].is_some());
        assert!((sma[2].unwrap() - 11.0).abs() < 1e-9); // (10+11+12)/3
    }

    #[test]
    fn rolling_std_nonnegative() {
        let rw = RollingWindow::new(prices(), 3);
        for v in rw.rolling_std().into_iter().flatten() {
            assert!(v >= 0.0);
        }
    }

    #[test]
    fn rolling_max_and_min_correct() {
        let rw = RollingWindow::new(prices(), 3);
        let maxes = rw.rolling_max();
        let mins = rw.rolling_min();
        // window [10,11,12]: max=12, min=10
        assert_eq!(maxes[2].unwrap(), 12.0);
        assert_eq!(mins[2].unwrap(), 10.0);
    }

    #[test]
    fn bollinger_upper_gt_lower() {
        let rw = RollingWindow::new(prices(), 3);
        let bands = rw.bollinger_bands(2.0);
        for i in 2..prices().len() {
            let u = bands.upper[i].unwrap();
            let l = bands.lower[i].unwrap();
            assert!(u >= l, "upper={u} lower={l} at i={i}");
        }
    }

    #[test]
    fn bollinger_middle_equals_sma() {
        let rw = RollingWindow::new(prices(), 3);
        let bands = rw.bollinger_bands(2.0);
        let sma = rw.sma();
        for (b, s) in bands.middle.iter().zip(sma.iter()) {
            assert_eq!(b, s);
        }
    }

    #[test]
    fn ema_same_length_as_input() {
        let ema = ExponentialMovingAverage::new(prices(), 3).compute();
        assert_eq!(ema.len(), prices().len());
    }

    #[test]
    fn ema_first_value_equals_first_price() {
        let ema = ExponentialMovingAverage::new(prices(), 3).compute();
        assert!((ema[0] - prices()[0]).abs() < 1e-9);
    }

    #[test]
    fn ema_trend_follows_direction() {
        let rising: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let ema = ExponentialMovingAverage::new(rising.clone(), 3).compute();
        // Last EMA should be greater than first
        assert!(ema.last().unwrap() > &ema[0]);
    }

    #[test]
    fn macd_same_length_as_input() {
        let p = prices();
        let (macd, signal) = ExponentialMovingAverage::macd(&p, 3, 5, 2);
        assert_eq!(macd.len(), p.len());
        assert_eq!(signal.len(), p.len());
    }

    #[test]
    fn flat_series_ema_equals_constant() {
        let flat: Vec<f64> = vec![5.0; 10];
        let ema = ExponentialMovingAverage::new(flat, 3).compute();
        for v in &ema {
            assert!((v - 5.0).abs() < 1e-9);
        }
    }
}
