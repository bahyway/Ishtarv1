//! PULUḪTU GATE — the Clearance Confidence Equation (PB-606).
//! GL-RU-001 (Puluḫtu unified risk unit) meets the Never-Averaged Theorem,
//! extended from KNOWN suspects to UNKNOWN residual risk.
//!
//! Honesty clause, stated in code: no equation certifies certainty about
//! hidden explosives. This module certifies a BOUNDED residual risk with
//! stated inputs, and refuses everything else. The refusal is the product.
//!
//! Residual risk after n independent clean sweeps (Bayes):
//!
//!   rho_n = pi0 * PROD(1 - p_i) / [ pi0 * PROD(1 - p_i) + (1 - pi0) ]
//!
//! Two-lock certification:
//!   Lock 1 — Never-Averaged (known):   S(A) = 0 suspects held
//!   Lock 2 — residual bound (unknown): max over cells rho_n(c) <= eps_theta
//!            The polygon's risk is its WORST cell, never its mean.
//!            A child does not walk on an average.
//!
//! Detection probabilities p_i must be lower confidence limits from field
//! trials, and only verified-independent modalities may enter the product
//! (Kinu witness discipline). This module enforces the arithmetic; the
//! independence verification is the court's duty, journaled, never assumed
//! silently.

/// Posterior residual risk for one cell after clean sweeps.
/// `prior` in (0,1); each sweep probability in [0,1).
/// Returns Err on invalid inputs rather than a silently wrong number.
pub fn posterior_residual(prior: f64, sweep_p: &[f64]) -> Result<f64, String> {
    if !(prior > 0.0 && prior < 1.0) {
        return Err(format!("prior {} outside (0,1) — a certainty is not a prior", prior));
    }
    let mut miss = 1.0f64;
    for (i, p) in sweep_p.iter().enumerate() {
        if !(*p >= 0.0 && *p < 1.0) {
            return Err(format!(
                "sweep {} detection probability {} outside [0,1) — \
                 p = 1 would claim a perfect sensor; none exists",
                i + 1,
                p
            ));
        }
        miss *= 1.0 - p;
    }
    Ok(prior * miss / (prior * miss + (1.0 - prior)))
}

/// Sweeps of a uniform modality needed to bring residual under eps.
/// Closed form: n >= ln[eps(1-pi0)/(pi0(1-eps))] / ln(1-p).
pub fn sweeps_needed(prior: f64, p: f64, eps: f64) -> Result<u32, String> {
    if !(prior > 0.0 && prior < 1.0) || !(eps > 0.0 && eps < 1.0) || !(p > 0.0 && p < 1.0) {
        return Err("prior, p, eps must all lie strictly in (0,1)".into());
    }
    let target = eps * (1.0 - prior) / (prior * (1.0 - eps));
    if target >= 1.0 {
        return Ok(0); // prior already below the bound
    }
    Ok((target.ln() / (1.0 - p).ln()).ceil() as u32)
}

/// The two-lock certification. `cell_residuals` are per-cell rho values;
/// the polygon is judged by its MAXIMUM — never averaged.
pub fn certify_two_locks(
    polygon_id: &str,
    known_suspects: usize,
    cell_residuals: &[f64],
    eps_theta: f64,
) -> Result<String, String> {
    if known_suspects > 0 {
        return Err(format!(
            "LOCK 1 REFUSED · {} · {} known suspect(s) still held prune-exempt · \
             Never-Averaged: almost clear is not clear",
            polygon_id, known_suspects
        ));
    }
    if cell_residuals.is_empty() {
        return Err(format!(
            "LOCK 2 REFUSED · {} · no residual assessment exists — \
             absence of analysis is not absence of mines",
            polygon_id
        ));
    }
    let worst = cell_residuals.iter().cloned().fold(0.0f64, f64::max);
    if worst > eps_theta {
        return Err(format!(
            "LOCK 2 REFUSED · {} · worst cell residual {:.2e} puluḫtu exceeds \
             ε(Θ) = {:.2e} · the polygon's risk is its worst cell, never its \
             mean — a child does not walk on an average",
            polygon_id, worst, eps_theta
        ));
    }
    Ok(format!(
        "ABSENCE CERTIFIED (BOUNDED) · {} · zero known suspects · worst-cell \
         residual {:.2e} ≤ ε(Θ) = {:.2e} puluḫtu · Kanīku chain carries the \
         full evidence trail — this is a bound, never a certainty",
        polygon_id, worst, eps_theta
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_independent_sweep_multiplies_risk_down() {
        let r0 = posterior_residual(0.2, &[]).unwrap();
        let r1 = posterior_residual(0.2, &[0.8]).unwrap();
        let r2 = posterior_residual(0.2, &[0.8, 0.7]).unwrap();
        let r3 = posterior_residual(0.2, &[0.8, 0.7, 0.6]).unwrap();
        assert!(r0 > r1 && r1 > r2 && r2 > r3, "monotone descent");
        assert!((r0 - 0.2).abs() < 1e-12, "no sweeps → prior unchanged");
    }

    #[test]
    fn a_perfect_sensor_is_refused_as_a_lie() {
        assert!(posterior_residual(0.2, &[1.0]).is_err());
        assert!(posterior_residual(0.0, &[0.5]).is_err());
        assert!(posterior_residual(1.0, &[0.5]).is_err());
    }

    #[test]
    fn worst_cell_governs_never_the_mean() {
        // Mean residual = (0.0001*9 + 0.05)/10 ≈ 0.0051 — below eps 0.01.
        // Worst cell 0.05 is above. Certification MUST refuse.
        let mut cells = vec![0.0001; 9];
        cells.push(0.05);
        let v = certify_two_locks("MF-URBAN-12", 0, &cells, 0.01);
        assert!(v.is_err());
        assert!(v.unwrap_err().contains("worst cell"));
    }

    #[test]
    fn zero_known_suspects_is_not_enough_alone() {
        // PB-604's lock passes (0 suspects) but the residual lock refuses:
        let v = certify_two_locks("MF-URBAN-12", 0, &[0.2], 0.001);
        assert!(v.is_err(), "unknown risk blocks even with zero known suspects");
        // and no assessment at all is a refusal, never a pass:
        assert!(certify_two_locks("MF-URBAN-12", 0, &[], 0.001).is_err());
    }

    #[test]
    fn both_locks_open_only_together() {
        assert!(certify_two_locks("MF-URBAN-12", 1, &[1e-6], 0.001).is_err());
        let ok = certify_two_locks("MF-URBAN-12", 0, &[1e-6, 5e-7], 0.001);
        assert!(ok.is_ok());
        assert!(ok.unwrap().contains("never a certainty"), "honesty travels in the verdict");
    }

    #[test]
    fn closed_form_sweeps_match_the_posterior() {
        let (prior, p, eps) = (0.2, 0.7, 1e-4);
        let n = sweeps_needed(prior, p, eps).unwrap();
        let sweeps = vec![p; n as usize];
        assert!(posterior_residual(prior, &sweeps).unwrap() <= eps);
        if n > 0 {
            let fewer = vec![p; (n - 1) as usize];
            assert!(posterior_residual(prior, &fewer).unwrap() > eps, "n is minimal");
        }
    }
}
