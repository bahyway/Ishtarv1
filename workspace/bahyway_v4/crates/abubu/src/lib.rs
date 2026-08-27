//! abubu — GL-ALG-003 (The Abubu Calculus): membrane rupture —
//! compliance, critical density, horizon, and the quarantine of
//! witnesses. PB-327. Pure Rust, zero dependencies.
#![forbid(unsafe_code)]

pub const EPS_Y_DEFAULT: f64 = 0.6;
pub const EPS_Q_DEFAULT: f64 = 0.05;

/// The compliance coefficient's sealed bounds: κ_min = 0 (the Rigid
/// Decree), κ_max = 1 (the Yield Normalization).
pub fn kappa_clamp(kappa: f64) -> f64 {
    kappa.clamp(0.0, 1.0)
}

/// The Critical Density Equation (§4). κ_M → 0 sends ρ* → ∞: the Rigid
/// Decree in equation form (L1).
pub fn rho_star(u_crit: f64, sigma: f64, kappa_m: f64, m_bar: f64) -> f64 {
    let kappa_m = kappa_clamp(kappa_m);
    if kappa_m == 0.0 || m_bar == 0.0 {
        return f64::INFINITY;
    }
    u_crit / (2.0 * std::f64::consts::PI * sigma * sigma * kappa_m * m_bar)
}

/// The station's breathing density at time t (§5): dρ/dt = λ_a − μρ.
pub fn rho_t(lambda_a: f64, mu: f64, rho0: f64, t: f64) -> f64 {
    if mu == 0.0 {
        return rho0 + lambda_a * t;
    }
    (lambda_a / mu) * (1.0 - (-mu * t).exp()) + rho0 * (-mu * t).exp()
}

/// The Horizon Equation (§5). `None` when the steady state never reaches
/// rho_star (safe verdict, L3) or when rho_star is infinite (Rigid
/// Decree, L1).
pub fn t_star(lambda_a: f64, mu: f64, rho0: f64, rho_star: f64) -> Option<f64> {
    if rho_star.is_infinite() || mu <= 0.0 {
        return None;
    }
    if lambda_a / mu <= rho_star {
        return None;
    }
    let numerator = lambda_a - mu * rho_star;
    let denominator = lambda_a - mu * rho0;
    if numerator <= 0.0 || denominator <= 0.0 {
        return None;
    }
    Some(-(1.0 / mu) * (numerator / denominator).ln())
}

pub fn horizon_alarm(t_star: Option<f64>, tau: f64) -> bool {
    matches!(t_star, Some(t) if t <= tau)
}

/// §3 — the three sealed strain regimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    Elastic,
    Plastic,
    Rupture,
}

fn regime_rank(r: Regime) -> u8 {
    match r {
        Regime::Elastic => 0,
        Regime::Plastic => 1,
        Regime::Rupture => 2,
    }
}

pub fn regime_ladder(s: f64, eps_y: f64) -> Regime {
    if s < eps_y {
        Regime::Elastic
    } else if s < 1.0 {
        Regime::Plastic
    } else {
        Regime::Rupture
    }
}

/// A membrane's regime never improves silently: worsening (Elastic ->
/// Plastic -> Rupture) always applies from a fresh S reading; improving
/// requires an explicit decree, because a ruptured membrane does not
/// heal itself from noise alone (§3, §6 "regime ladder never skips
/// downward without decree").
#[derive(Debug, Clone, Copy)]
pub struct RegimeTracker {
    pub current: Regime,
}

impl RegimeTracker {
    pub fn new() -> Self {
        Self { current: Regime::Elastic }
    }

    pub fn advance(&mut self, s: f64, eps_y: f64, decree_signed: bool) -> Regime {
        let candidate = regime_ladder(s, eps_y);
        if regime_rank(candidate) >= regime_rank(self.current) || decree_signed {
            self.current = candidate;
        }
        self.current
    }
}

impl Default for RegimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessState {
    Untouched,
    DemotedFuzzyQuarantined,
}

/// §6 — the Quarantine Clause.
pub fn quarantine_verdict(p_rupture: f64, eps_q: f64) -> WitnessState {
    if p_rupture > eps_q {
        WitnessState::DemotedFuzzyQuarantined
    } else {
        WitnessState::Untouched
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // L1 — Rigid Decree: kappa_M = 0 => rho* = infinity, T* = None,
    // regime = Elastic forever.
    #[test]
    fn l1_rigid_decree() {
        let rs = rho_star(10.0, 1.0, 0.0, 5.0);
        assert!(rs.is_infinite());
        assert!(t_star(2.0, 0.5, 0.0, rs).is_none());
        assert_eq!(regime_ladder(0.0, EPS_Y_DEFAULT), Regime::Elastic);
    }

    // L2 — compliance monotonicity: rho* strictly decreasing in
    // kappa_M and in m_bar.
    #[test]
    fn l2_compliance_monotonicity() {
        let rs_low_kappa = rho_star(10.0, 1.0, 0.2, 5.0);
        let rs_high_kappa = rho_star(10.0, 1.0, 0.8, 5.0);
        assert!(rs_high_kappa < rs_low_kappa, "softer walls rupture at lower crowding");

        let rs_low_load = rho_star(10.0, 1.0, 0.5, 2.0);
        let rs_high_load = rho_star(10.0, 1.0, 0.5, 8.0);
        assert!(rs_high_load < rs_low_load, "heavier traffic ruptures a membrane sooner");
    }

    // L3 — safe breath: lambda_a/mu <= rho* => no finite T*.
    #[test]
    fn l3_safe_breath_no_horizon() {
        let rs = rho_star(10.0, 1.0, 0.5, 1.0);
        let lambda_a = 0.01;
        let mu = 1.0;
        assert!(lambda_a / mu <= rs);
        assert!(t_star(lambda_a, mu, 0.0, rs).is_none());
    }

    // L4 — horizon monotonicity: T* decreasing in lambda_a; alarm iff
    // T* <= tau.
    #[test]
    fn l4_horizon_monotonicity_and_alarm() {
        let rs = 5.0;
        let mu = 0.5;
        let rho0 = 0.0;
        let t_slow = t_star(6.0, mu, rho0, rs).unwrap();
        let t_fast = t_star(20.0, mu, rho0, rs).unwrap();
        assert!(t_fast < t_slow, "a higher arrival rate must shorten the horizon");
        assert!(horizon_alarm(Some(t_fast), t_fast + 1.0));
        assert!(!horizon_alarm(Some(t_fast), t_fast - 1.0));
        assert!(!horizon_alarm(None, 100.0));
    }

    // L5 — quarantine: p_rupture > eps_Q => FUZZY + EnkiQDB route (this
    // crate names the state); p <= eps_Q => untouched.
    #[test]
    fn l5_quarantine_threshold() {
        assert_eq!(quarantine_verdict(0.10, EPS_Q_DEFAULT), WitnessState::DemotedFuzzyQuarantined);
        assert_eq!(quarantine_verdict(0.05, EPS_Q_DEFAULT), WitnessState::Untouched);
        assert_eq!(quarantine_verdict(0.01, EPS_Q_DEFAULT), WitnessState::Untouched);
    }

    // L6 — regime ladder: crossing eps_y moves Elastic->Plastic; crossing
    // 1 moves Plastic->Rupture; never skips downward without decree.
    #[test]
    fn l6_regime_ladder_and_no_silent_healing() {
        assert_eq!(regime_ladder(0.3, EPS_Y_DEFAULT), Regime::Elastic);
        assert_eq!(regime_ladder(0.7, EPS_Y_DEFAULT), Regime::Plastic);
        assert_eq!(regime_ladder(1.2, EPS_Y_DEFAULT), Regime::Rupture);

        let mut tracker = RegimeTracker::new();
        assert_eq!(tracker.advance(0.7, EPS_Y_DEFAULT, false), Regime::Plastic);
        assert_eq!(tracker.advance(1.2, EPS_Y_DEFAULT, false), Regime::Rupture);
        // A lower S reading alone must NOT silently heal the membrane:
        assert_eq!(
            tracker.advance(0.1, EPS_Y_DEFAULT, false),
            Regime::Rupture,
            "regime must not improve without a decree"
        );
        // Only a signed decree may move it down:
        assert_eq!(tracker.advance(0.1, EPS_Y_DEFAULT, true), Regime::Elastic);
    }
}
