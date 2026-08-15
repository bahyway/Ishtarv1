//! Sovereign Factor Model — maps VGCA 7D FSV + price/volume metrics to
//! 5 classic quantitative factors.
//!
//! The insight: an asset's Feature Score Vector IS its factor loading vector.
//! D1–D7 of FSV map naturally to the 5 Fama-French + momentum factors:
//!
//! | VGCA Dimension | Sovereign Factor | Classic Factor |
//! |----------------|-----------------|----------------|
//! | D7 shannon_entropy_norm | MOMENTUM   | Price momentum (trend) |
//! | D1 char_count_norm      | SIZE       | Asset size / volume profile |
//! | D2 digit_density        | VALUE      | Valuation density (numeric precision) |
//! | D4 latin_density        | QUALITY    | Data quality geometry |
//! | D6 word_count_norm      | LOW_VOL    | Volatility / frequency proxy |
//!
//! Additional factor from price analytics:
//! | Source | REVERSAL | Short-term mean reversion from rolling std |

use crate::returns::ReturnSeries;
use crate::rolling::RollingWindow;

/// The 5 sovereign quantitative factors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantFactor {
    /// D7 entropy → momentum (high entropy = high price information = trending).
    Momentum,
    /// D1 char_count_norm → size (larger data profile ≈ larger asset).
    Size,
    /// D2 digit_density → value (numeric data precision ≈ valuation clarity).
    Value,
    /// D4 latin_density → quality (data cleanliness = earnings quality).
    Quality,
    /// D6 word_count_norm + volatility → low-volatility factor.
    LowVolatility,
}

impl QuantFactor {
    pub fn label(self) -> &'static str {
        match self {
            Self::Momentum     => "MOMENTUM",
            Self::Size         => "SIZE",
            Self::Value        => "VALUE",
            Self::Quality      => "QUALITY",
            Self::LowVolatility => "LOW_VOL",
        }
    }

    /// Index into the FSV array (D1=0, D2=1, ..., D7=6).
    pub fn fsv_index(self) -> usize {
        match self {
            Self::Momentum      => 6, // D7 shannon_entropy_norm
            Self::Size          => 0, // D1 char_count_norm
            Self::Value         => 1, // D2 digit_density
            Self::Quality       => 3, // D4 latin_density
            Self::LowVolatility => 5, // D6 word_count_norm
        }
    }
}

/// Factor scores for one asset at one point in time.
#[derive(Debug, Clone)]
pub struct FactorScores {
    pub asset_hash:   u32,
    pub epoch:        u32,
    /// Raw factor scores in [0, 1] mapped from FSV dimensions.
    pub scores:       [f64; 5],
    /// Z-scored factor exposures (cross-sectional, set externally).
    pub z_scores:     [f64; 5],
}

impl FactorScores {
    /// Build from a VGCA FSV array (7 dimensions).
    pub fn from_fsv(asset_hash: u32, epoch: u32, fsv: &[f64; 7]) -> Self {
        let scores = [
            fsv[QuantFactor::Momentum.fsv_index()],
            fsv[QuantFactor::Size.fsv_index()],
            fsv[QuantFactor::Value.fsv_index()],
            fsv[QuantFactor::Quality.fsv_index()],
            fsv[QuantFactor::LowVolatility.fsv_index()],
        ];
        Self { asset_hash, epoch, scores, z_scores: [0.0; 5] }
    }

    /// Score for a specific factor.
    pub fn score(&self, factor: QuantFactor) -> f64 {
        self.scores[factor as usize]
    }

    /// Composite quality score (equal-weighted mean of all 5 factors).
    pub fn composite(&self) -> f64 {
        self.scores.iter().sum::<f64>() / 5.0
    }

    /// B11 quality score for this asset based on composite factor.
    /// B11 = round(composite × 240) — ADR-001.
    pub fn b11(&self) -> u8 {
        (self.composite() * 240.0).round() as u8
    }
}

/// The factor model — builds factor scores for a universe of assets.
pub struct FactorModel {
    pub universe: Vec<FactorScores>,
}

impl FactorModel {
    pub fn new() -> Self { FactorModel { universe: Vec::new() } }

    /// Add factor scores for one asset.
    pub fn add(&mut self, scores: FactorScores) {
        self.universe.push(scores);
    }

    /// Cross-sectional z-score of factor `f` across all assets.
    /// Updates `z_scores[f]` for every asset in the universe.
    pub fn compute_cross_sectional_z(&mut self, factor: QuantFactor) {
        let idx = factor as usize;
        let vals: Vec<f64> = self.universe.iter().map(|s| s.scores[idx]).collect();
        let (mean, std) = mean_and_std_local(&vals);
        for s in &mut self.universe {
            s.z_scores[idx] = if std > 1e-12 { (s.scores[idx] - mean) / std } else { 0.0 };
        }
    }

    /// Run cross-sectional z-scoring for all 5 factors.
    pub fn compute_all_z_scores(&mut self) {
        for factor in [QuantFactor::Momentum, QuantFactor::Size,
                       QuantFactor::Value, QuantFactor::Quality,
                       QuantFactor::LowVolatility] {
            self.compute_cross_sectional_z(factor);
        }
    }

    /// Rank assets by composite score (highest first).
    pub fn rank_by_composite(&self) -> Vec<&FactorScores> {
        let mut ranked: Vec<&FactorScores> = self.universe.iter().collect();
        ranked.sort_by(|a, b| b.composite().partial_cmp(&a.composite()).unwrap());
        ranked
    }

    /// Assets in the GEM lane (B11 ≥ 200).
    pub fn gem_assets(&self) -> Vec<&FactorScores> {
        self.universe.iter().filter(|s| s.b11() >= 200).collect()
    }

    /// GEM rate across the universe (ADR-004 target: 35.4%).
    pub fn gem_rate(&self) -> f64 {
        if self.universe.is_empty() { return 0.0; }
        let gem = self.universe.iter().filter(|s| s.b11() >= 200).count();
        gem as f64 / self.universe.len() as f64
    }

    pub fn asset_count(&self) -> usize { self.universe.len() }
}

impl Default for FactorModel {
    fn default() -> Self { Self::new() }
}

/// Build factor scores from a return series using price-derived momentum.
///
/// Uses rolling Z-score of returns to generate the momentum factor score,
/// mapping it to a [0,1] range (0.5 = neutral, >0.5 = positive momentum).
pub fn momentum_from_returns(returns: &ReturnSeries, window: usize) -> Vec<f64> {
    if returns.is_empty() { return Vec::new(); }
    let rw = RollingWindow::new(returns.log_returns().to_vec(), window);
    let sma = rw.sma();
    sma.iter().map(|v| match v {
        None    => 0.5,
        Some(m) => (0.5 + m * 10.0).clamp(0.0, 1.0),
    }).collect()
}

fn mean_and_std_local(values: &[f64]) -> (f64, f64) {
    let n = values.len();
    if n == 0 { return (0.0, 0.0); }
    let mean = values.iter().sum::<f64>() / n as f64;
    let var  = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    (mean, var.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fsv_perfect() -> [f64; 7] { [1.0; 7] }
    fn fsv_zero()    -> [f64; 7] { [0.0; 7] }
    fn fsv_mixed()   -> [f64; 7] { [0.8, 0.6, 0.7, 0.9, 0.5, 0.4, 0.7] }

    #[test]
    fn perfect_fsv_b11_is_240() {
        let s = FactorScores::from_fsv(0x01, 1, &fsv_perfect());
        assert_eq!(s.b11(), 240);
    }

    #[test]
    fn zero_fsv_b11_is_zero() {
        let s = FactorScores::from_fsv(0x01, 1, &fsv_zero());
        assert_eq!(s.b11(), 0);
    }

    #[test]
    fn composite_in_unit_interval() {
        let s = FactorScores::from_fsv(0x01, 1, &fsv_mixed());
        assert!(s.composite() >= 0.0 && s.composite() <= 1.0);
    }

    #[test]
    fn factor_labels_non_empty() {
        for f in [QuantFactor::Momentum, QuantFactor::Size, QuantFactor::Value,
                  QuantFactor::Quality, QuantFactor::LowVolatility] {
            assert!(!f.label().is_empty());
        }
    }

    #[test]
    fn factor_fsv_indices_in_range() {
        for f in [QuantFactor::Momentum, QuantFactor::Size, QuantFactor::Value,
                  QuantFactor::Quality, QuantFactor::LowVolatility] {
            assert!(f.fsv_index() < 7, "{} index={}", f.label(), f.fsv_index());
        }
    }

    #[test]
    fn gem_rate_all_perfect() {
        let mut model = FactorModel::new();
        for i in 0..10u32 {
            model.add(FactorScores::from_fsv(i, 1, &fsv_perfect()));
        }
        assert!((model.gem_rate() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn gem_rate_none_perfect() {
        let mut model = FactorModel::new();
        for i in 0..10u32 {
            model.add(FactorScores::from_fsv(i, 1, &fsv_zero()));
        }
        assert!((model.gem_rate()).abs() < 1e-9);
    }

    #[test]
    fn rank_by_composite_descending() {
        let mut model = FactorModel::new();
        model.add(FactorScores::from_fsv(0x01, 1, &fsv_mixed()));
        model.add(FactorScores::from_fsv(0x02, 1, &fsv_perfect()));
        model.add(FactorScores::from_fsv(0x03, 1, &fsv_zero()));
        let ranked = model.rank_by_composite();
        assert_eq!(ranked[0].asset_hash, 0x02); // perfect → highest
        assert_eq!(ranked[2].asset_hash, 0x03); // zero → lowest
    }

    #[test]
    fn cross_sectional_z_scores_computed() {
        let mut model = FactorModel::new();
        for i in 0..5u32 {
            let mut fsv = [0.5f64; 7];
            fsv[0] = i as f64 * 0.2; // vary D1 only
            model.add(FactorScores::from_fsv(i, 1, &fsv));
        }
        model.compute_cross_sectional_z(QuantFactor::Size);
        let z_scores: Vec<f64> = model.universe.iter().map(|s| s.z_scores[QuantFactor::Size as usize]).collect();
        let z_mean: f64 = z_scores.iter().sum::<f64>() / z_scores.len() as f64;
        assert!(z_mean.abs() < 1e-9, "z-score mean should be ~0, got {z_mean}");
    }

    #[test]
    fn momentum_from_returns_clamps_to_unit_interval() {
        use crate::returns::ReturnSeries;
        let prices: Vec<i64> = (1..=20).map(|i| i * 100).collect();
        let rs = ReturnSeries::from_prices(&prices, 1);
        let mom = momentum_from_returns(&rs, 3);
        for &v in &mom { assert!(v >= 0.0 && v <= 1.0, "v={v}"); }
    }
}
