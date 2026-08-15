//! Sovereign fuzzy membership functions and linguistic variables.
//!
//! Implements: Triangular, Trapezoidal, Gaussian, Sigmoid, SigmoidRev, Constant.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

// ── MembershipFn ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum MembershipFn {
    /// Triangular(a, b, c): 0 at a and c, 1 at b
    Triangular { a: f32, b: f32, c: f32 },
    /// Trapezoidal(a, b, c, d): rises a→b, flat b→c, falls c→d
    Trapezoidal { a: f32, b: f32, c: f32, d: f32 },
    /// Gaussian(mean, sigma)
    Gaussian { mean: f32, sigma: f32 },
    /// Sigmoid(steepness, center) — smooth 0→1 transition
    Sigmoid { steepness: f32, center: f32 },
    /// Reverse sigmoid — smooth 1→0 transition
    SigmoidRev { steepness: f32, center: f32 },
    /// Constant membership value (for always-true antecedents)
    Constant(f32),
}

impl MembershipFn {
    /// Evaluate μ(x) → [0.0, 1.0]
    pub fn eval(&self, x: f32) -> f32 {
        match self {
            Self::Triangular { a, b, c } => {
                if x <= *a || x >= *c { 0.0 }
                else if x < *b        { (x - a) / (b - a) }
                else if (x - b).abs() < 1e-8 { 1.0 }
                else                  { (c - x) / (c - b) }
            }
            Self::Trapezoidal { a, b, c, d } => {
                if x <= *a || x >= *d          { 0.0 }
                else if x >= *b && x <= *c     { 1.0 }
                else if x < *b                 { (x - a) / (b - a) }
                else                           { (d - x) / (d - c) }
            }
            Self::Gaussian { mean, sigma } => {
                let exp = -((x - mean).powi(2)) / (2.0 * sigma.powi(2));
                exp.exp().clamp(0.0, 1.0)
            }
            Self::Sigmoid { steepness, center } => {
                1.0 / (1.0 + (-steepness * (x - center)).exp())
            }
            Self::SigmoidRev { steepness, center } => {
                1.0 / (1.0 + (steepness * (x - center)).exp())
            }
            Self::Constant(v) => v.clamp(0.0, 1.0),
        }
    }
}

// ── LinguisticVar ─────────────────────────────────────────────────────────────

/// Named fuzzy variable with labeled membership functions per term.
#[derive(Debug, Clone)]
pub struct LinguisticVar {
    pub name:  String,
    pub terms: Vec<(String, MembershipFn)>,
    pub range: (f32, f32),
}

impl LinguisticVar {
    pub fn eval_term(&self, term: &str, x: f32) -> Option<f32> {
        self.terms.iter()
            .find(|(t, _)| t == term)
            .map(|(_, f)| f.eval(x))
    }

    pub fn eval_all(&self, x: f32) -> Vec<(String, f32)> {
        self.terms.iter().map(|(t, f)| (t.clone(), f.eval(x))).collect()
    }
}

// ── Pre-built Nusku Fuzzy Variables ──────────────────────────────────────────

/// Torso temperature deviation variable [-4, +4 °C]
pub fn var_torso_dev() -> LinguisticVar {
    LinguisticVar {
        name: "torso_deviation".into(),
        range: (-4.0, 4.0),
        terms: vec![
            ("cold_shadow".into(),   MembershipFn::Trapezoidal { a: -4.0, b: -3.0, c: -1.5, d: -0.8 }),
            ("slightly_cool".into(), MembershipFn::Triangular  { a: -1.5, b: -0.5, c:  0.2 }),
            ("normal".into(),        MembershipFn::Trapezoidal { a: -0.5, b: -0.2, c:  0.2, d:  0.6 }),
            ("warm".into(),          MembershipFn::Triangular  { a:  0.4, b:  1.0, c:  1.8 }),
            ("hot_fever".into(),     MembershipFn::Trapezoidal { a:  1.2, b:  1.8, c:  3.5, d:  4.0 }),
        ],
    }
}

/// Head temperature deviation variable [-3, +4 °C]
pub fn var_head_dev() -> LinguisticVar {
    LinguisticVar {
        name: "head_deviation".into(),
        range: (-3.0, 4.0),
        terms: vec![
            ("cold".into(),           MembershipFn::Sigmoid    { steepness: -3.0, center: -1.0 }),
            ("normal".into(),         MembershipFn::Triangular { a: -0.3, b:  0.0, c:  0.5 }),
            ("stress_elevated".into(),MembershipFn::Triangular { a:  0.4, b:  0.9, c:  1.6 }),
            ("fever_elevated".into(), MembershipFn::Trapezoidal{ a:  1.2, b:  1.8, c:  3.5, d: 4.0 }),
        ],
    }
}

/// Arm temperature deviation variable (detonator heat indicator) [-3, +4 °C]
pub fn var_arm_dev() -> LinguisticVar {
    LinguisticVar {
        name: "arm_deviation".into(),
        range: (-3.0, 4.0),
        terms: vec![
            ("cold_hands".into(), MembershipFn::Trapezoidal { a: -3.0, b: -2.0, c: -0.8, d: -0.3 }),
            ("normal".into(),     MembershipFn::Triangular  { a: -0.4, b:  0.0, c:  0.4 }),
            ("warm".into(),       MembershipFn::Triangular  { a:  0.3, b:  0.8, c:  1.5 }),
            ("hot_wiring".into(), MembershipFn::Trapezoidal { a:  1.0, b:  1.5, c:  3.5, d: 4.0 }),
        ],
    }
}

/// Body symmetry variable [0, 1]
pub fn var_symmetry() -> LinguisticVar {
    LinguisticVar {
        name: "symmetry".into(),
        range: (0.0, 1.0),
        terms: vec![
            ("asymmetric".into(), MembershipFn::Trapezoidal { a: 0.0,  b: 0.0,  c: 0.35, d: 0.55 }),
            ("moderate".into(),   MembershipFn::Triangular  { a: 0.4,  b: 0.65, c: 0.82 }),
            ("symmetric".into(),  MembershipFn::Trapezoidal { a: 0.72, b: 0.85, c: 1.0,  d: 1.0  }),
        ],
    }
}

/// Torso-arm inversion variable [0, 1] — security threat indicator
pub fn var_torso_arm_inversion() -> LinguisticVar {
    LinguisticVar {
        name: "torso_arm_inversion".into(),
        range: (0.0, 1.0),
        terms: vec![
            ("none".into(),    MembershipFn::Trapezoidal { a: 0.0, b: 0.0,  c: 0.15, d: 0.30 }),
            ("partial".into(), MembershipFn::Triangular  { a: 0.2, b: 0.40, c: 0.60 }),
            ("strong".into(),  MembershipFn::Trapezoidal { a: 0.5, b: 0.65, c: 1.0,  d: 1.0  }),
        ],
    }
}

/// Output variable: sovereign state classification [0, 1]
pub fn var_state_output() -> LinguisticVar {
    LinguisticVar {
        name: "state".into(),
        range: (0.0, 1.0),
        terms: vec![
            ("clear".into(),           MembershipFn::Trapezoidal { a: 0.0,  b: 0.0,  c: 0.20, d: 0.35 }),
            ("fever".into(),           MembershipFn::Triangular  { a: 0.25, b: 0.40, c: 0.55 }),
            ("stress".into(),          MembershipFn::Triangular  { a: 0.30, b: 0.45, c: 0.60 }),
            ("medical_other".into(),   MembershipFn::Triangular  { a: 0.35, b: 0.50, c: 0.65 }),
            ("ambiguous".into(),       MembershipFn::Triangular  { a: 0.50, b: 0.65, c: 0.78 }),
            ("security_threat".into(), MembershipFn::Trapezoidal { a: 0.70, b: 0.82, c: 1.0,  d: 1.0  }),
        ],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangular_peak_returns_one() {
        let f = MembershipFn::Triangular { a: 0.0, b: 1.0, c: 2.0 };
        assert!((f.eval(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn triangular_outside_range_returns_zero() {
        let f = MembershipFn::Triangular { a: 0.0, b: 1.0, c: 2.0 };
        assert_eq!(f.eval(-0.1), 0.0);
        assert_eq!(f.eval(2.0),  0.0);
        assert_eq!(f.eval(3.0),  0.0);
    }

    #[test]
    fn trapezoidal_flat_region_returns_one() {
        let f = MembershipFn::Trapezoidal { a: 0.0, b: 1.0, c: 3.0, d: 4.0 };
        assert_eq!(f.eval(2.0), 1.0);
    }

    #[test]
    fn trapezoidal_outside_returns_zero() {
        let f = MembershipFn::Trapezoidal { a: 0.0, b: 1.0, c: 3.0, d: 4.0 };
        assert_eq!(f.eval(-1.0), 0.0);
        assert_eq!(f.eval(5.0),  0.0);
    }

    #[test]
    fn gaussian_at_mean_returns_one() {
        let f = MembershipFn::Gaussian { mean: 0.5, sigma: 0.2 };
        assert!((f.eval(0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sigmoid_at_center_returns_half() {
        let f = MembershipFn::Sigmoid { steepness: 5.0, center: 0.5 };
        assert!((f.eval(0.5) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn sigmoid_rev_at_center_returns_half() {
        let f = MembershipFn::SigmoidRev { steepness: 5.0, center: 0.5 };
        assert!((f.eval(0.5) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn constant_returns_its_value() {
        assert!((MembershipFn::Constant(0.7).eval(999.0) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn all_membership_values_bounded() {
        let fns = [
            MembershipFn::Triangular { a: 0.0, b: 0.5, c: 1.0 },
            MembershipFn::Trapezoidal { a: 0.0, b: 0.3, c: 0.7, d: 1.0 },
            MembershipFn::Gaussian { mean: 0.5, sigma: 0.15 },
            MembershipFn::Sigmoid { steepness: 10.0, center: 0.5 },
            MembershipFn::SigmoidRev { steepness: 10.0, center: 0.5 },
        ];
        for x_i in 0..=20 {
            let x = x_i as f32 * 0.1 - 0.5;
            for f in &fns {
                let v = f.eval(x);
                assert!(v >= 0.0 && v <= 1.0, "{f:?} eval({x})={v}");
            }
        }
    }

    #[test]
    fn var_torso_dev_has_five_terms() {
        assert_eq!(var_torso_dev().terms.len(), 5);
    }

    #[test]
    fn var_state_output_has_six_terms() {
        assert_eq!(var_state_output().terms.len(), 6);
    }

    #[test]
    fn linguistic_var_eval_all_covers_all_terms() {
        let v = var_torso_dev();
        let results = v.eval_all(0.0);
        assert_eq!(results.len(), v.terms.len());
    }

    #[test]
    fn cold_shadow_term_fires_on_cold_torso() {
        let v = var_torso_dev();
        let mu = v.eval_term("cold_shadow", -2.0).unwrap();
        assert!(mu > 0.9, "cold_shadow μ(-2.0)={mu}");
    }

    #[test]
    fn hot_wiring_term_fires_on_hot_arms() {
        let v = var_arm_dev();
        let mu = v.eval_term("hot_wiring", 2.0).unwrap();
        assert!(mu > 0.9, "hot_wiring μ(2.0)={mu}");
    }
}
