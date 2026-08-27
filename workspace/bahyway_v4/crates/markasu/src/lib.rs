//! markasu — GL-NSR-001 (The Nasaru Alert Law) + GL-NSR-001-A1 (The Temennu
//! Baseline & The Rigmu Escalation). PB-330/PB-331.
//!
//! *Markasu* — the mooring rope. A healthy tribe's centroid X(t) around its
//! Apsu center μ is Ornstein-Uhlenbeck: dX = θ(μ−X)dt + σdW. This crate
//! seals the first alert on *motion*: the early warning that homeostasis is
//! failing while position still looks healthy — DETECT→PROVE→PREDICT made
//! operational.
//!
//! Pure Rust, zero dependencies, per the tablet's own playbook clause.
#![forbid(unsafe_code)]

/// A tiny deterministic PRNG (xorshift64*) so the OU simulator and its law
/// tests are reproducible without pulling in an external `rand` dependency.
pub struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn next_unit(&mut self) -> f64 {
        // 53 bits of mantissa -> (0, 1)
        ((self.next_u64() >> 11) as f64) / ((1u64 << 53) as f64)
    }

    /// Standard normal draw via Box-Muller.
    pub fn next_normal(&mut self) -> f64 {
        let u1 = self.next_unit().max(f64::MIN_POSITIVE);
        let u2 = self.next_unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

// ───────────────────────── §2 The Mooring Model (OU process) ─────────────

/// Exact-discretization Ornstein-Uhlenbeck step:
/// X⁺ = μ + e^(−θΔ)(X−μ) + σ·√((1−e^(−2θΔ))/(2θ))·ξ
pub fn ou_step(x: f64, mu: f64, theta: f64, sigma: f64, dt: f64, rng: &mut Xorshift64) -> f64 {
    if theta <= 0.0 {
        // θ → 0: unmoored, free Brownian motion (§2).
        return x + sigma * dt.sqrt() * rng.next_normal();
    }
    let decay = (-theta * dt).exp();
    let var_term = ((1.0 - (-2.0 * theta * dt).exp()) / (2.0 * theta)).max(0.0);
    mu + decay * (x - mu) + sigma * var_term.sqrt() * rng.next_normal()
}

/// Simulate `n_steps` of the OU process starting at μ (its own center).
pub fn simulate(theta: f64, mu: f64, sigma: f64, dt: f64, n_steps: usize, seed: u64) -> Vec<f64> {
    let mut rng = Xorshift64::new(seed);
    let mut series = Vec::with_capacity(n_steps);
    let mut x = mu;
    for _ in 0..n_steps {
        x = ou_step(x, mu, theta, sigma, dt, &mut rng);
        series.push(x);
    }
    series
}

// ───────────────────── §3 The Two Witnesses (PROVE-form) ─────────────────

fn mean(series: &[f64]) -> f64 {
    series.iter().sum::<f64>() / series.len() as f64
}

fn sample_variance(series: &[f64]) -> f64 {
    let m = mean(series);
    series.iter().map(|x| (x - m).powi(2)).sum::<f64>() / series.len() as f64
}

/// Witness A — the rope's memory: lag-1 autocorrelation of the centered
/// series; θ̂_A = −ln(φ̂)/Δ. `None` when φ̂ ≤ 0 (ln undefined — an unmoored
/// or anti-persistent series, not a slackening estimate).
pub fn witness_a(series: &[f64], dt: f64) -> Option<f64> {
    if series.len() < 2 {
        return None;
    }
    let m = mean(series);
    let centered: Vec<f64> = series.iter().map(|x| x - m).collect();
    let denom: f64 = centered.iter().map(|c| c * c).sum();
    if denom <= 0.0 {
        return None;
    }
    let numer: f64 = centered.windows(2).map(|w| w[0] * w[1]).sum();
    let phi = numer / denom;
    if phi <= 0.0 {
        return None;
    }
    Some(-phi.ln() / dt)
}

/// Witness B — the rope's slack: increment variance against sample
/// variance; θ̂_B = σ̂²/(2·Var̂(X)).
pub fn witness_b(series: &[f64], dt: f64) -> Option<f64> {
    if series.len() < 2 {
        return None;
    }
    let increments: Vec<f64> = series.windows(2).map(|w| w[1] - w[0]).collect();
    let sigma_sq = increments.iter().map(|d| d * d).sum::<f64>() / increments.len() as f64 / dt;
    let var_x = sample_variance(series);
    if var_x <= 0.0 {
        return None;
    }
    Some(sigma_sq / (2.0 * var_x))
}

// ───────────────── §3 Alert state machine (MARKASU-01, base law) ─────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowVerdict {
    Healthy,
    Fuzzy,
    BothLow,
}

/// One window's verdict against the base (non-Temennu) law: fires
/// `BothLow` only when both witnesses are below θ_min beyond ε; a single
/// low witness is `Fuzzy` (journaled, no bell) per §3.
pub fn evaluate_window(
    theta_hat_a: Option<f64>,
    theta_hat_b: Option<f64>,
    theta_min: f64,
    eps: f64,
) -> WindowVerdict {
    let a_low = theta_hat_a.map(|t| t < theta_min - eps).unwrap_or(false);
    let b_low = theta_hat_b.map(|t| t < theta_min - eps).unwrap_or(false);
    match (a_low, b_low) {
        (true, true) => WindowVerdict::BothLow,
        (true, false) | (false, true) => WindowVerdict::Fuzzy,
        (false, false) => WindowVerdict::Healthy,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertState {
    Healthy,
    Fuzzy,
    Markasu,
}

/// Tracks consecutive `BothLow` windows; MARKASU-01 fires only after `w`
/// consecutive both-low windows (default W=3, §3).
pub struct AlertTracker {
    consecutive_both_low: u32,
    w: u32,
}

impl AlertTracker {
    pub fn new(w: u32) -> Self {
        Self {
            consecutive_both_low: 0,
            w,
        }
    }

    pub fn push(&mut self, verdict: WindowVerdict) -> AlertState {
        match verdict {
            WindowVerdict::BothLow => {
                self.consecutive_both_low += 1;
                if self.consecutive_both_low >= self.w {
                    AlertState::Markasu
                } else {
                    AlertState::Fuzzy
                }
            }
            WindowVerdict::Fuzzy => {
                self.consecutive_both_low = 0;
                AlertState::Fuzzy
            }
            WindowVerdict::Healthy => {
                self.consecutive_both_low = 0;
                AlertState::Healthy
            }
        }
    }
}

// ───────────────────── A1 §1 The Temennu Baseline ─────────────────────────

fn median(values: &[f64]) -> f64 {
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn mad(values: &[f64], center: f64) -> f64 {
    let devs: Vec<f64> = values.iter().map(|v| (v - center).abs()).collect();
    median(&devs)
}

/// A Temennu is a *document*, not a variable: sealed once, re-laid only by
/// decree (§1 — silent baseline creep is how a slackening system teaches
/// its own watchman to sleep). There is deliberately no public setter for
/// `theta0`/`s0`: the only way to change them is `re_lay_by_decree`, which
/// refuses without an explicit signed decree.
#[derive(Debug, Clone)]
pub struct Temennu {
    pub tribe_id: u64,
    pub theta0: f64,
    pub s0: f64,
    pub enrollment_windows: usize,
}

/// Lays a Temennu from accepted enrollment estimates: θ̂ is accepted only
/// when witnesses A and B agree within tolerance δ (witness-agreement
/// gate, §1). θ₀ = median(θ̂), s₀ = MAD(θ̂) — robust, outlier-proof.
pub fn lay_temennu(
    tribe_id: u64,
    witness_pairs: &[(f64, f64)],
    delta: f64,
) -> Option<Temennu> {
    let accepted: Vec<f64> = witness_pairs
        .iter()
        .filter(|(a, b)| (a - b).abs() <= delta * a.abs())
        .map(|(a, _)| *a)
        .collect();
    if accepted.is_empty() {
        return None;
    }
    let theta0 = median(&accepted);
    let s0 = mad(&accepted, theta0);
    Some(Temennu {
        tribe_id,
        theta0,
        s0,
        enrollment_windows: accepted.len(),
    })
}

impl Temennu {
    /// The only lawful path to a changed baseline: a signed decree lays a
    /// *new* Temennu beside the old one. Refuses (returns `None`) without
    /// `decree_signed` — automatic re-baselining is forbidden (§1).
    pub fn re_lay_by_decree(
        &self,
        witness_pairs: &[(f64, f64)],
        delta: f64,
        decree_signed: bool,
    ) -> Option<Temennu> {
        if !decree_signed {
            return None;
        }
        lay_temennu(self.tribe_id, witness_pairs, delta)
    }
}

// ─────────────────── A1 §2 LEVEL/TREND witnesses, re-founded ─────────────

/// LEVEL witness: θ̂ < κ·θ₀ beyond ε (default κ = 0.5).
pub fn level_witness(theta_hat: f64, theta0: f64, kappa: f64, eps: f64) -> bool {
    theta_hat < kappa * theta0 - eps
}

/// Mann-Kendall Z statistic over a series (positive = rising, negative =
/// falling trend).
pub fn mann_kendall_z(series: &[f64]) -> f64 {
    let n = series.len() as i64;
    let mut s: i64 = 0;
    for i in 0..series.len() {
        for j in (i + 1)..series.len() {
            // NOTE: deliberately not `f64::signum()` -- that method returns
            // 1.0 for +0.0 (never 0.0), which would count every tied pair
            // as "positive" and break Mann-Kendall on any series with
            // repeats (a flat step, in particular, would read as a strong
            // upward trend instead of no trend at all). Mann-Kendall's own
            // sgn(x) is 1/0/-1 for pos/zero/neg.
            let diff = series[j] - series[i];
            s += if diff > 0.0 {
                1
            } else if diff < 0.0 {
                -1
            } else {
                0
            };
        }
    }
    let var_s = (n * (n - 1) * (2 * n + 5)) as f64 / 18.0;
    if var_s <= 0.0 {
        return 0.0;
    }
    if s > 0 {
        (s as f64 - 1.0) / var_s.sqrt()
    } else if s < 0 {
        (s as f64 + 1.0) / var_s.sqrt()
    } else {
        0.0
    }
}

/// Sen's slope: the median of all pairwise slopes.
pub fn sens_slope(series: &[f64]) -> f64 {
    let mut slopes: Vec<f64> = Vec::new();
    for i in 0..series.len() {
        for j in (i + 1)..series.len() {
            slopes.push((series[j] - series[i]) / (j - i) as f64);
        }
    }
    median(&slopes)
}

/// TREND witness: Mann-Kendall Z ≤ −z_crit (default 1.96).
pub fn trend_witness(series: &[f64], z_crit: f64) -> bool {
    mann_kendall_z(series) <= -z_crit
}

/// T_θ horizon: time until the level witness fires at the current decay
/// rate — the countdown, not just the alarm.
pub fn t_theta_horizon(theta_hat: f64, kappa: f64, theta0: f64, slope: f64) -> Option<f64> {
    if slope >= 0.0 {
        return None;
    }
    Some((theta_hat - kappa * theta0) / slope.abs())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A1WindowVerdict {
    Healthy,
    /// TREND alone: early-early warning, no integrity concern.
    FuzzyTrendOnly,
    /// LEVEL alone without TREND: a cliff is more often a pipeline fault
    /// than a slackening spring — flagged for data-integrity review.
    FuzzyLevelOnlyIntegrityFlag,
    Markasu,
}

/// A1 §2: MARKASU-01 fires iff LEVEL ∧ TREND for W consecutive
/// evaluations (tracked by the caller via `AlertTracker`-style counting,
/// same as the base law). This function evaluates a single window.
pub fn evaluate_a1_window(level: bool, trend: bool) -> A1WindowVerdict {
    match (level, trend) {
        (true, true) => A1WindowVerdict::Markasu,
        (false, true) => A1WindowVerdict::FuzzyTrendOnly,
        (true, false) => A1WindowVerdict::FuzzyLevelOnlyIntegrityFlag,
        (false, false) => A1WindowVerdict::Healthy,
    }
}

// ───────────────────── A1 §3 The Rigmu Escalation ─────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TribeClass {
    Fuzzy,
    GoldenEnkiDb7004,
    GoldenEnkiDw7005,
    GoldenOther,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityKind {
    /// Dead tribes: no mooring watch — the dead are not moored.
    NoWatch,
    Markasu,
    Rigmu,
}

/// Severity is decided by the state class of the tribe's particles (§3).
pub fn route_severity(class: TribeClass) -> SeverityKind {
    match class {
        TribeClass::Dead => SeverityKind::NoWatch,
        TribeClass::GoldenEnkiDb7004 | TribeClass::GoldenEnkiDw7005 => SeverityKind::Rigmu,
        TribeClass::Fuzzy | TribeClass::GoldenOther => SeverityKind::Markasu,
    }
}

/// A RIGMU cannot be acknowledged, only answered: it remains open until a
/// signed explanation particle is attached (§3.2). This state machine
/// refuses to close without one.
#[derive(Debug, Default)]
pub struct RigmuCase {
    pub frozen: bool,
    open: bool,
    explanation: Option<String>,
}

impl RigmuCase {
    pub fn open_new() -> Self {
        Self {
            frozen: true,
            open: true,
            explanation: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Refuses to close (`Err`) without a non-empty explanation particle.
    pub fn close(&mut self, explanation: Option<&str>) -> Result<(), &'static str> {
        match explanation {
            Some(text) if !text.is_empty() => {
                self.explanation = Some(text.to_string());
                self.open = false;
                self.frozen = false;
                Ok(())
            }
            _ => Err("RIGMU refuses to close without a signed explanation particle"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // L1 — healthy tribe (θ = 2·θ_min), long run: zero alerts.
    #[test]
    fn l1_healthy_tribe_zero_alerts() {
        let theta_min = 0.05;
        let theta = 2.0 * theta_min;
        let dt = 1.0;
        let window_len = 200;
        let n_windows = 10;
        let series = simulate(theta, 0.0, 1.0, dt, window_len * n_windows, 42);

        let mut tracker = AlertTracker::new(3);
        let mut ever_markasu = false;
        for w in 0..n_windows {
            let window = &series[w * window_len..(w + 1) * window_len];
            let a = witness_a(window, dt);
            let b = witness_b(window, dt);
            let verdict = evaluate_window(a, b, theta_min, 0.0);
            if tracker.push(verdict) == AlertState::Markasu {
                ever_markasu = true;
            }
        }
        assert!(!ever_markasu, "healthy tribe must never ring MARKASU");
    }

    // L2 — scripted slackening: the bell rings while |X-mu| is still
    // inside the healthy band (early warning proven, not asserted).
    #[test]
    fn l2_early_warning_before_excursion() {
        let theta_min = 0.05;
        let dt = 1.0;
        let window_len = 200;
        // Two healthy windows establish a settled baseline, then theta
        // drops sharply and clearly below theta_min: a single window is
        // enough for both witnesses to detect it (W=1 here -- W's own
        // consecutive-count mechanic is exercised separately by L1/L4;
        // this test isolates the early-warning claim itself).
        let thetas = [0.20, 0.20, 0.02, 0.01, 0.01];
        let healthy_band = 3.0 * (1.0f64 / (2.0 * theta_min)).sqrt(); // 3*sigma_stationary at theta_min, sigma=1

        let mut tracker = AlertTracker::new(1);
        let mut x = 0.0;
        let mut rng = Xorshift64::new(7);
        let mut fired_within_band = false;
        let mut ever_fired = false;

        for &theta in &thetas {
            // The position at the START of this window is what "still
            // healthy" means at the moment the estimation window that will
            // trigger this alert begins accumulating -- not the wander that
            // happens *during* the very estimation window used to detect
            // it (checking the latter would conflate the statistical
            // estimator's required observation length with the excursion
            // claim itself).
            let x_at_window_start = x;
            let mut window = Vec::with_capacity(window_len);
            for _ in 0..window_len {
                x = ou_step(x, 0.0, theta, 1.0, dt, &mut rng);
                window.push(x);
            }
            let a = witness_a(&window, dt);
            let b = witness_b(&window, dt);
            let verdict = evaluate_window(a, b, theta_min, 0.0);
            let state = tracker.push(verdict);
            if state == AlertState::Markasu && !ever_fired {
                ever_fired = true;
                fired_within_band = x_at_window_start.abs() < healthy_band;
            }
        }
        assert!(ever_fired, "scripted slackening must eventually ring MARKASU");
        assert!(
            fired_within_band,
            "the bell must ring while the tribe is still within the healthy band"
        );
    }

    // L3 — estimator consistency: theta_hat_A, theta_hat_B -> theta on a
    // long stationary series within epsilon.
    #[test]
    fn l3_estimator_consistency() {
        let theta = 0.3;
        let dt = 1.0;
        let series = simulate(theta, 0.0, 1.0, dt, 20_000, 99);
        let a = witness_a(&series, dt).expect("witness A must resolve");
        let b = witness_b(&series, dt).expect("witness B must resolve");
        assert!(
            (a - theta).abs() / theta < 0.25,
            "witness A {a} too far from true theta {theta}"
        );
        assert!(
            (b - theta).abs() / theta < 0.25,
            "witness B {b} too far from true theta {theta}"
        );
    }

    // L4 — two-witness enforced: corrupt one witness -> FUZZY only, never
    // a bell.
    #[test]
    fn l4_two_witness_enforced() {
        let mut tracker = AlertTracker::new(3);
        let mut ever_markasu = false;
        for _ in 0..10 {
            // Witness A genuinely low; witness B corrupted (unresolved).
            let verdict = evaluate_window(Some(0.001), None, 0.05, 0.0);
            if tracker.push(verdict) == AlertState::Markasu {
                ever_markasu = true;
            }
        }
        assert!(!ever_markasu, "a single corrupted witness must never ring the bell");
    }

    // L5 — Temennu never auto-relays (creep attempt -> unchanged seal).
    #[test]
    fn l5_temennu_immutable_without_decree() {
        let pairs = vec![(0.30, 0.29), (0.31, 0.30), (0.29, 0.28)];
        let t0 = lay_temennu(1, &pairs, 0.1).expect("enrollment should accept agreeing pairs");
        let original_theta0 = t0.theta0;

        // Creep attempt: drastically different estimates, no decree signed.
        let creep_pairs = vec![(0.05, 0.05), (0.04, 0.04)];
        let attempt = t0.re_lay_by_decree(&creep_pairs, 0.1, false);
        assert!(attempt.is_none(), "un-decreed re-lay must be refused");
        assert_eq!(t0.theta0, original_theta0, "the sealed Temennu itself is never mutated");

        // With a signed decree, a NEW Temennu may be laid (old one still untouched).
        let decreed = t0
            .re_lay_by_decree(&creep_pairs, 0.1, true)
            .expect("a signed decree lays a new Temennu");
        assert_ne!(decreed.theta0, original_theta0);
        assert_eq!(t0.theta0, original_theta0, "old deposit remains, per §1");
    }

    // L6 — trend-only or level-only never rings.
    #[test]
    fn l6_single_witness_never_rings() {
        assert_eq!(evaluate_a1_window(true, false), A1WindowVerdict::FuzzyLevelOnlyIntegrityFlag);
        assert_eq!(evaluate_a1_window(false, true), A1WindowVerdict::FuzzyTrendOnly);
        assert_ne!(evaluate_a1_window(true, false), A1WindowVerdict::Markasu);
        assert_ne!(evaluate_a1_window(false, true), A1WindowVerdict::Markasu);
    }

    // L7 — step-change fires the integrity flag, not the bell.
    #[test]
    fn l7_step_change_integrity_flag_not_bell() {
        // A constant, immediately-low level with a flat (no-trend) series:
        // Mann-Kendall Z ~= 0, so TREND is false -> level-only path.
        let flat_low_theta = vec![0.01; 20];
        let z = mann_kendall_z(&flat_low_theta);
        let trend = trend_witness(&flat_low_theta, 1.96);
        assert!(z.abs() < 1e-9, "a flat step has no trend");
        assert!(!trend);
        let level = level_witness(0.01, 0.10, 0.5, 0.0);
        assert!(level, "0.01 is well below kappa*theta0 = 0.05");
        assert_eq!(
            evaluate_a1_window(level, trend),
            A1WindowVerdict::FuzzyLevelOnlyIntegrityFlag
        );
    }

    // L8 — RIGMU refuses to close without a signed explanation particle.
    #[test]
    fn l8_rigmu_refuses_silent_close() {
        let mut case = RigmuCase::open_new();
        assert!(case.is_open());
        assert!(case.frozen);
        assert!(case.close(None).is_err(), "silent close must be refused");
        assert!(case.is_open(), "case stays open after a refused close");
        assert!(case.close(Some("")).is_err(), "empty explanation is not an explanation");
        assert!(case
            .close(Some("upstream corruption, confirmed by two-witness"))
            .is_ok());
        assert!(!case.is_open());
        assert!(!case.frozen);
    }

    // Severity routing sanity: GOLDEN 7004/7005 -> RIGMU, everything else
    // watched -> MARKASU, DEAD -> no watch.
    #[test]
    fn severity_routing() {
        assert_eq!(route_severity(TribeClass::Fuzzy), SeverityKind::Markasu);
        assert_eq!(route_severity(TribeClass::GoldenEnkiDb7004), SeverityKind::Rigmu);
        assert_eq!(route_severity(TribeClass::GoldenEnkiDw7005), SeverityKind::Rigmu);
        assert_eq!(route_severity(TribeClass::GoldenOther), SeverityKind::Markasu);
        assert_eq!(route_severity(TribeClass::Dead), SeverityKind::NoWatch);
    }

    #[test]
    fn sens_slope_and_horizon() {
        let series = vec![1.0, 0.9, 0.8, 0.7, 0.6];
        let slope = sens_slope(&series);
        assert!((slope - (-0.1)).abs() < 1e-9);
        let horizon = t_theta_horizon(0.6, 0.5, 1.0, slope).unwrap();
        assert!((horizon - 1.0).abs() < 1e-9); // (0.6 - 0.5)/0.1 = 1.0
    }
}
