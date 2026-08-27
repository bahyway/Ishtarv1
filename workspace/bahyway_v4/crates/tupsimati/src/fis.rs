//! Authoring model for the real FuzzyEngine + ScoreEngine (Tertu diagnosis
//! layer). This module authors their inputs; it does not reimplement
//! inference.

#[derive(Clone, Debug)]
pub enum Membership {
    Triangular { a: f64, b: f64, c: f64 },
    Trapezoidal { a: f64, b: f64, c: f64, d: f64 },
}

impl Membership {
    pub fn degree(&self, x: f64) -> f64 {
        match *self {
            Membership::Triangular { a, b, c } => {
                if x <= a || x >= c {
                    0.0
                } else if x <= b {
                    (x - a) / (b - a)
                } else {
                    (c - x) / (c - b)
                }
            }
            Membership::Trapezoidal { a, b, c, d } => {
                if x <= a || x >= d {
                    0.0
                } else if x < b {
                    (x - a) / (b - a)
                } else if x <= c {
                    1.0
                } else {
                    (d - x) / (d - c)
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct FuzzySet {
    pub metric: String, // e.g. cqrs_lag_ms
    pub label: String,  // e.g. high
    pub shape: Membership,
}

/// IF metric IS label (AND ...) THEN health IS label, weight w
#[derive(Clone, Debug)]
pub struct MamdaniRule {
    pub antecedents: Vec<(String, String)>, // (metric, label)
    pub consequent: String,                 // health label
    pub weight: f64,
}

#[derive(Default, Clone, Debug)]
pub struct FisConfig {
    pub sets: Vec<FuzzySet>,
    pub rules: Vec<MamdaniRule>,
    pub score_weights: Vec<(String, f64)>, // ScoreEngine aggregation
}

impl FisConfig {
    pub fn validate(&self) -> Result<(), String> {
        for r in &self.rules {
            for (m, l) in &r.antecedents {
                if !self.sets.iter().any(|s| &s.metric == m && &s.label == l) {
                    return Err(format!("rule references undefined set {m}/{l}"));
                }
            }
            if r.weight < 0.0 || r.weight > 1.0 {
                return Err("rule weight outside [0,1]".into());
            }
        }
        Ok(())
    }

    pub fn to_lines(&self) -> String {
        let mut out = String::new();
        for s in &self.sets {
            let shape = match s.shape {
                Membership::Triangular { a, b, c } => format!("TRI {a} {b} {c}"),
                Membership::Trapezoidal { a, b, c, d } => format!("TRA {a} {b} {c} {d}"),
            };
            out += &format!("SET {} {} {}\n", s.metric, s.label, shape);
        }
        for r in &self.rules {
            let ants: Vec<String> = r
                .antecedents
                .iter()
                .map(|(m, l)| format!("{m}:{l}"))
                .collect();
            out += &format!(
                "RULE {} THEN {} W {}\n",
                ants.join(" AND "),
                r.consequent,
                r.weight
            );
        }
        for (m, w) in &self.score_weights {
            out += &format!("WEIGHT {m} {w}\n");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_rule_referencing_undefined_set() {
        let fis = FisConfig {
            sets: vec![],
            rules: vec![MamdaniRule {
                antecedents: vec![("cqrs_lag_ms".into(), "high".into())],
                consequent: "degraded".into(),
                weight: 0.5,
            }],
            score_weights: vec![],
        };
        assert!(fis.validate().is_err());
    }

    #[test]
    fn validate_accepts_a_grounded_rule() {
        let fis = FisConfig {
            sets: vec![FuzzySet {
                metric: "cqrs_lag_ms".into(),
                label: "high".into(),
                shape: Membership::Trapezoidal {
                    a: 100.0,
                    b: 500.0,
                    c: 1000.0,
                    d: 2000.0,
                },
            }],
            rules: vec![MamdaniRule {
                antecedents: vec![("cqrs_lag_ms".into(), "high".into())],
                consequent: "degraded".into(),
                weight: 0.8,
            }],
            score_weights: vec![("cqrs_lag_ms".into(), 0.5)],
        };
        assert!(fis.validate().is_ok());
    }

    #[test]
    fn validate_rejects_out_of_range_weight() {
        let fis = FisConfig {
            sets: vec![FuzzySet {
                metric: "m".into(),
                label: "l".into(),
                shape: Membership::Triangular {
                    a: 0.0,
                    b: 1.0,
                    c: 2.0,
                },
            }],
            rules: vec![MamdaniRule {
                antecedents: vec![("m".into(), "l".into())],
                consequent: "c".into(),
                weight: 1.5,
            }],
            score_weights: vec![],
        };
        assert!(fis.validate().is_err());
    }

    #[test]
    fn triangular_membership_peaks_at_b() {
        let m = Membership::Triangular {
            a: 0.0,
            b: 1.0,
            c: 2.0,
        };
        assert_eq!(m.degree(1.0), 1.0);
        assert_eq!(m.degree(0.0), 0.0);
        assert_eq!(m.degree(2.0), 0.0);
        assert!((m.degree(0.5) - 0.5).abs() < 1e-9);
    }
}
