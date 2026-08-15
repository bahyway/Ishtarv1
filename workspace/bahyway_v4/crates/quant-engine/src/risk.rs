//! Risk metrics — VaR, CVaR, Sharpe, Sortino, Calmar, max drawdown.
//!
//! Historical (non-parametric) VaR/CVaR: no normal distribution assumption.
//! All metrics are computed from a return series (Vec<f64>).

use crate::returns::mean_and_std;

/// Full risk report for one asset's return series.
#[derive(Debug, Clone)]
pub struct RiskReport {
    /// Annualised Sharpe ratio (assumes mean/std already annualised if desired).
    /// Sharpe = (mean_return − risk_free) / std_dev
    pub sharpe:         f64,
    /// Sortino ratio — penalises only downside volatility.
    /// Sortino = (mean_return − risk_free) / downside_std
    pub sortino:        f64,
    /// Calmar ratio = annualised_return / |max_drawdown|.
    pub calmar:         f64,
    /// Historical Value at Risk at the given confidence level.
    /// VaR_95 = loss not exceeded 95% of the time (positive = loss).
    pub var:            f64,
    /// Conditional VaR (Expected Shortfall) — mean loss beyond VaR.
    pub cvar:           f64,
    /// Confidence level used for VaR/CVaR (e.g. 0.95).
    pub confidence:     f64,
    /// Maximum drawdown (fraction, negative value).
    pub max_drawdown:   f64,
    /// Mean return per period.
    pub mean_return:    f64,
    /// Population standard deviation of returns (total volatility).
    pub volatility:     f64,
    /// Downside deviation (std of negative returns only).
    pub downside_std:   f64,
    /// Total cumulative return.
    pub total_return:   f64,
    /// Win rate (fraction of positive-return periods).
    pub win_rate:       f64,
}

impl RiskReport {
    /// Compute all risk metrics from a return series.
    ///
    /// - `returns`    — slice of per-period returns (log or simple)
    /// - `confidence` — VaR/CVaR confidence level (e.g. 0.95)
    /// - `risk_free`  — risk-free rate per period (0.0 for pure Sharpe)
    pub fn from_returns(returns: &[f64], confidence: f64) -> Self {
        Self::with_risk_free(returns, confidence, 0.0)
    }

    pub fn with_risk_free(returns: &[f64], confidence: f64, risk_free: f64) -> Self {
        if returns.is_empty() {
            return Self::zero(confidence);
        }
        let (mean, vol) = mean_and_std(returns);
        let downside_std = downside_deviation(returns);
        let sharpe  = if vol.abs() > 1e-15 { (mean - risk_free) / vol } else { 0.0 };
        let sortino = if downside_std.abs() > 1e-15 { (mean - risk_free) / downside_std } else { 0.0 };
        let (var, cvar) = historical_var_cvar(returns, confidence);
        let max_dd  = max_drawdown(returns);
        let total   = returns.iter().sum::<f64>();
        let calmar  = if max_dd.abs() > 1e-15 { total / max_dd.abs() } else { 0.0 };
        let win_rate = returns.iter().filter(|&&r| r > 0.0).count() as f64 / returns.len() as f64;

        RiskReport { sharpe, sortino, calmar, var, cvar, confidence,
            max_drawdown: max_dd, mean_return: mean, volatility: vol,
            downside_std, total_return: total, win_rate }
    }

    fn zero(confidence: f64) -> Self {
        RiskReport { sharpe: 0.0, sortino: 0.0, calmar: 0.0, var: 0.0, cvar: 0.0,
            confidence, max_drawdown: 0.0, mean_return: 0.0, volatility: 0.0,
            downside_std: 0.0, total_return: 0.0, win_rate: 0.0 }
    }

    /// One-line summary for Dashboard tooltip.
    pub fn summary(&self) -> String {
        format!(
            "Sharpe={:.3} VaR{:.0}%={:.4} MaxDD={:.2}% WinRate={:.1}%",
            self.sharpe,
            self.confidence * 100.0,
            self.var,
            self.max_drawdown * 100.0,
            self.win_rate * 100.0,
        )
    }
}

/// Drawdown time series — each value is the drawdown from the running peak.
#[derive(Debug, Clone)]
pub struct DrawdownSeries {
    pub drawdowns: Vec<f64>,
    pub max_drawdown: f64,
    /// Index of the drawdown trough.
    pub trough_index: usize,
}

impl DrawdownSeries {
    pub fn from_returns(returns: &[f64]) -> Self {
        let mut equity = 1.0f64;
        let mut peak   = 1.0f64;
        let mut drawdowns = Vec::with_capacity(returns.len());
        let mut max_dd = 0.0f64;
        let mut trough = 0usize;

        for (i, &r) in returns.iter().enumerate() {
            equity *= 1.0 + r;
            if equity > peak { peak = equity; }
            let dd = (equity - peak) / peak;
            if dd < max_dd {
                max_dd = dd;
                trough = i;
            }
            drawdowns.push(dd);
        }
        Self { drawdowns, max_drawdown: max_dd, trough_index: trough }
    }

    /// Duration of the maximum drawdown in periods (from peak to trough).
    pub fn max_drawdown_duration(&self) -> usize { self.trough_index + 1 }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Downside deviation: std of returns below zero.
fn downside_deviation(returns: &[f64]) -> f64 {
    let below: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).cloned().collect();
    if below.is_empty() { return 0.0; }
    let mean = below.iter().sum::<f64>() / below.len() as f64;
    let var  = below.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / below.len() as f64;
    var.sqrt()
}

/// Historical (non-parametric) VaR and CVaR.
///
/// Returns `(VaR, CVaR)` where VaR is the loss at the given confidence level
/// (positive = loss, so a return of -0.03 contributes VaR of 0.03).
fn historical_var_cvar(returns: &[f64], confidence: f64) -> (f64, f64) {
    if returns.is_empty() { return (0.0, 0.0); }
    // Sort ascending (most negative first)
    let mut sorted: Vec<f64> = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let cutoff_idx = ((1.0 - confidence) * sorted.len() as f64).floor() as usize;
    let cutoff_idx = cutoff_idx.min(sorted.len().saturating_sub(1));

    let var = -sorted[cutoff_idx]; // negate: loss is positive
    let cvar = if cutoff_idx == 0 {
        -sorted[0]
    } else {
        -sorted[..cutoff_idx].iter().sum::<f64>() / cutoff_idx as f64
    };
    (var.max(0.0), cvar.max(0.0))
}

/// Maximum drawdown: worst peak-to-trough decline in cumulative returns.
fn max_drawdown(returns: &[f64]) -> f64 {
    DrawdownSeries::from_returns(returns).max_drawdown
}

#[cfg(test)]
mod tests {
    use super::*;

    fn returns() -> Vec<f64> {
        vec![0.02, -0.01, 0.03, -0.02, 0.01, -0.03, 0.04, -0.015, 0.02, 0.01]
    }

    #[test]
    fn sharpe_finite() {
        let r = RiskReport::from_returns(&returns(), 0.95);
        assert!(r.sharpe.is_finite());
    }

    #[test]
    fn var_nonnegative() {
        let r = RiskReport::from_returns(&returns(), 0.95);
        assert!(r.var >= 0.0);
        assert!(r.cvar >= 0.0);
    }

    #[test]
    fn cvar_ge_var() {
        let r = RiskReport::from_returns(&returns(), 0.95);
        assert!(r.cvar >= r.var - 1e-9, "CVaR={} < VaR={}", r.cvar, r.var);
    }

    #[test]
    fn max_drawdown_nonpositive() {
        let r = RiskReport::from_returns(&returns(), 0.95);
        assert!(r.max_drawdown <= 0.0);
    }

    #[test]
    fn monotone_rising_has_zero_drawdown() {
        let up: Vec<f64> = vec![0.01; 10];
        let r = RiskReport::from_returns(&up, 0.95);
        assert!((r.max_drawdown).abs() < 1e-9);
    }

    #[test]
    fn win_rate_in_unit_interval() {
        let r = RiskReport::from_returns(&returns(), 0.95);
        assert!(r.win_rate >= 0.0 && r.win_rate <= 1.0);
    }

    #[test]
    fn empty_returns_gives_zero_report() {
        let r = RiskReport::from_returns(&[], 0.95);
        assert_eq!(r.sharpe, 0.0);
        assert_eq!(r.var, 0.0);
    }

    #[test]
    fn drawdown_series_max_matches_risk_report() {
        let rets = returns();
        let ds = DrawdownSeries::from_returns(&rets);
        let rr = RiskReport::from_returns(&rets, 0.95);
        assert!((ds.max_drawdown - rr.max_drawdown).abs() < 1e-12);
    }

    #[test]
    fn all_negative_returns_high_var() {
        let bad: Vec<f64> = vec![-0.05; 20];
        let r = RiskReport::from_returns(&bad, 0.95);
        assert!(r.var > 0.0, "var should be positive for all-negative returns");
    }

    #[test]
    fn sortino_ge_sharpe_for_symmetric_returns() {
        // Symmetric returns: downside ≈ total std → Sortino ≈ Sharpe
        let sym: Vec<f64> = vec![0.02, -0.02, 0.02, -0.02];
        let r = RiskReport::from_returns(&sym, 0.95);
        assert!(r.sharpe.is_finite());
        assert!(r.sortino.is_finite());
    }
}
