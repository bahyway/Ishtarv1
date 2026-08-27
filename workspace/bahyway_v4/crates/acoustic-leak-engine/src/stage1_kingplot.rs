//! Stage 1 -- hydraulic King-plot layer.
//!
//! Deming regression (errors-in-both-variables, the ODR special case for a
//! straight line) replaces the OLS shortcut. Residual dispersion uses n-2
//! degrees of freedom (slope + intercept consumed) -- the n-divisor bias is
//! outlawed here.

#[derive(Debug, Clone, Copy)]
pub struct DemingFit {
    pub slope: f64,
    pub intercept: f64,
    /// Unbiased residual standard deviation (n-2 dof).
    pub resid_sd: f64,
    pub n: usize,
}

/// `lambda` = ratio of Y-error variance to X-error variance (1.0 when both
/// sensor pairs share the same noise class -- the honest default).
pub fn deming_fit(x: &[f64], y: &[f64], lambda: f64) -> Option<DemingFit> {
    let n = x.len();
    if n < 3 || y.len() != n || lambda <= 0.0 {
        return None;
    }
    let nf = n as f64;
    let mx = x.iter().sum::<f64>() / nf;
    let my = y.iter().sum::<f64>() / nf;
    let (mut sxx, mut syy, mut sxy) = (0.0_f64, 0.0_f64, 0.0_f64);
    for i in 0..n {
        let dx = x[i] - mx;
        let dy = y[i] - my;
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    if sxy.abs() < f64::EPSILON {
        return None;
    }
    // Deming closed form.
    let a = syy - lambda * sxx;
    let slope = (a + (a * a + 4.0 * lambda * sxy * sxy).sqrt()) / (2.0 * sxy);
    let intercept = my - slope * mx;

    // Orthogonal-weighted residuals, n-2 dof.
    let denom = (n - 2) as f64;
    let norm = 1.0 + slope * slope / lambda;
    let ss: f64 = (0..n)
        .map(|i| {
            let r = y[i] - (slope * x[i] + intercept);
            r * r / norm
        })
        .sum();
    Some(DemingFit {
        slope,
        intercept,
        resid_sd: (ss / denom).sqrt(),
        n,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage1Judgement {
    GridIntegrityHolds,
    /// Residual exceeded k*sd -- segment flagged for Stage 2.
    SegmentFlagged,
}

/// Steady-state gating is the caller's duty: never feed transient windows.
pub fn judge_point(fit: &DemingFit, x: f64, y: f64, k_sigma: f64) -> Stage1Judgement {
    let r = y - (fit.slope * x + fit.intercept);
    if r.abs() > k_sigma * fit.resid_sd {
        Stage1Judgement::SegmentFlagged
    } else {
        Stage1Judgement::GridIntegrityHolds
    }
}
