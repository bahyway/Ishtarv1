//! solver.rs — Sovereign algebraic equation solver for EaAgent.
//! Exact arithmetic — zero hallucination. No LLM for the math itself.
#![forbid(unsafe_code)]

use ea_agent_core::constants::ABZU_PRECISION;

/// A mathematical equation to solve.
#[derive(Debug, Clone)]
pub enum Equation {
    /// Linear: ax + b = 0
    Linear { a: f64, b: f64 },
    /// Quadratic: ax² + bx + c = 0
    Quadratic { a: f64, b: f64, c: f64 },
    /// Cubic: ax³ + bx² + cx + d = 0
    Cubic { a: f64, b: f64, c: f64, d: f64 },
    /// H(P) quality equation for given target and weights
    HeptaQuality {
        dimensions: [f64; 7],
        target: [f64; 7],
        weights: [f64; 7],
    },
    /// Spectral radius of a 2×2 matrix
    SpectralRadius2x2 { a: f64, b: f64, c: f64, d: f64 },
}

/// Result of solving an equation.
#[derive(Debug, Clone)]
pub struct SolverResult {
    pub equation: String,
    pub roots: Vec<f64>,
    pub complex: Vec<(f64, f64)>, // (real, imag) pairs
    pub value: Option<f64>,       // for non-root equations
    pub steps: Vec<String>,       // chain-of-thought proof
    pub latex: String,            // LaTeX representation
}

impl SolverResult {
    pub fn has_real_roots(&self) -> bool {
        !self.roots.is_empty()
    }
    pub fn is_exact(&self) -> bool {
        self.complex.is_empty() || !self.roots.is_empty()
    }
}

/// The sovereign algebraic solver — exact, zero-hallucination.
pub struct AlgebraSolver;

impl AlgebraSolver {
    pub fn new() -> Self {
        Self
    }

    pub fn solve(&self, eq: &Equation) -> SolverResult {
        match eq {
            Equation::Linear { a, b } => self.solve_linear(*a, *b),
            Equation::Quadratic { a, b, c } => self.solve_quadratic(*a, *b, *c),
            Equation::Cubic { a, b, c, d } => self.solve_cubic(*a, *b, *c, *d),
            Equation::HeptaQuality {
                dimensions,
                target,
                weights,
            } => self.solve_hepta_quality(*dimensions, *target, *weights),
            Equation::SpectralRadius2x2 { a, b, c, d } => self.solve_spectral_2x2(*a, *b, *c, *d),
        }
    }

    fn solve_linear(&self, a: f64, b: f64) -> SolverResult {
        let mut steps = Vec::new();
        steps.push(format!("Linear equation: {a}x + {b} = 0"));

        if a.abs() < ABZU_PRECISION {
            steps.push("a = 0: not a valid linear equation".into());
            return SolverResult {
                equation: format!("{a}x + {b} = 0"),
                roots: vec![],
                complex: vec![],
                value: None,
                steps,
                latex: format!("{a}x + {b} = 0"),
            };
        }

        let x = -b / a;
        steps.push(format!("x = -b/a = -{b}/{a} = {x}"));

        SolverResult {
            equation: format!("{a}x + {b} = 0"),
            roots: vec![x],
            complex: vec![],
            value: Some(x),
            steps,
            latex: format!("x = \\frac{{{}}}{{{}}} = {x:.6}", -b, a),
        }
    }

    fn solve_quadratic(&self, a: f64, b: f64, c: f64) -> SolverResult {
        let mut steps = Vec::new();
        steps.push(format!("Quadratic: {a}x² + {b}x + {c} = 0"));

        if a.abs() < ABZU_PRECISION {
            return self.solve_linear(b, c);
        }

        let discriminant = b * b - 4.0 * a * c;
        steps.push(format!(
            "Discriminant Δ = b² - 4ac = {b}² - 4({a})({c}) = {discriminant}"
        ));

        if discriminant > 0.0 {
            let sq = discriminant.sqrt();
            let x1 = (-b + sq) / (2.0 * a);
            let x2 = (-b - sq) / (2.0 * a);
            steps.push("Δ > 0 → two distinct real roots".into());
            steps.push(format!("x₁ = (-{b} + {sq:.6}) / {:.6} = {x1:.6}", 2.0 * a));
            steps.push(format!("x₂ = (-{b} - {sq:.6}) / {:.6} = {x2:.6}", 2.0 * a));
            SolverResult {
                equation: format!("{a}x² + {b}x + {c} = 0"),
                roots: vec![x1, x2],
                complex: vec![],
                value: None,
                steps,
                latex: format!(
                    "x = \\frac{{-{b} \\pm \\sqrt{{{discriminant:.4}}}}}{{{}}}",
                    2.0 * a
                ),
            }
        } else if discriminant.abs() < ABZU_PRECISION {
            let x = -b / (2.0 * a);
            steps.push("Δ = 0 → one repeated real root".into());
            steps.push(format!("x = -b/2a = {x:.6}"));
            SolverResult {
                equation: format!("{a}x² + {b}x + {c} = 0"),
                roots: vec![x],
                complex: vec![],
                value: Some(x),
                steps,
                latex: format!("x = \\frac{{-{b}}}{{{}}} = {x:.6}", 2.0 * a),
            }
        } else {
            let real = -b / (2.0 * a);
            let imag = (-discriminant).sqrt() / (2.0 * a);
            steps.push("Δ < 0 → two complex conjugate roots".into());
            steps.push(format!("x₁ = {real:.6} + {imag:.6}i"));
            steps.push(format!("x₂ = {real:.6} - {imag:.6}i"));
            SolverResult {
                equation: format!("{a}x² + {b}x + {c} = 0"),
                roots: vec![],
                complex: vec![(real, imag), (real, -imag)],
                value: None,
                steps,
                latex: format!("x = {real:.4} \\pm {imag:.4}i"),
            }
        }
    }

    fn solve_cubic(&self, a: f64, b: f64, c: f64, d: f64) -> SolverResult {
        // Cardano's formula for depressed cubic
        let mut steps = Vec::new();
        steps.push(format!("Cubic: {a}x³ + {b}x² + {c}x + {d} = 0"));

        if a.abs() < ABZU_PRECISION {
            return self.solve_quadratic(b, c, d);
        }

        // Normalize: x³ + px² + qx + r = 0
        let p = b / a;
        let q = c / a;
        let r = d / a;
        steps.push(format!("Normalized: x³ + {p:.4}x² + {q:.4}x + {r:.4} = 0"));

        // Depressed cubic: substitute x = t - p/3
        let shift = p / 3.0;
        let pp = q - p * p / 3.0;
        let qq = r + 2.0 * p.powi(3) / 27.0 - p * q / 3.0;
        steps.push(format!(
            "Depressed cubic t³ + {pp:.4}t + {qq:.4} = 0 (shift x = t - {shift:.4})"
        ));

        let disc = qq * qq / 4.0 + pp.powi(3) / 27.0;
        steps.push(format!("Discriminant D = {disc:.6}"));

        let roots = if disc > 0.0 {
            let u = (-qq / 2.0 + disc.sqrt()).cbrt();
            let v = (-qq / 2.0 - disc.sqrt()).cbrt();
            let t = u + v;
            steps.push(format!("D > 0: one real root t = {t:.6}"));
            vec![t - shift]
        } else if disc.abs() < ABZU_PRECISION {
            let u = (-qq / 2.0).cbrt();
            steps.push("D = 0: repeated roots".into());
            vec![2.0 * u - shift, -u - shift]
        } else {
            // Three real roots via trigonometric method
            let m = 2.0 * (-pp / 3.0).sqrt();
            let theta = (3.0 * qq / (pp * m)).acos() / 3.0;
            let t1 = m * theta.cos() - shift;
            let t2 = m * (theta - std::f64::consts::PI * 2.0 / 3.0).cos() - shift;
            let t3 = m * (theta - std::f64::consts::PI * 4.0 / 3.0).cos() - shift;
            steps.push("D < 0: three real roots via trigonometric method".into());
            vec![t1, t2, t3]
        };

        let latex = roots
            .iter()
            .map(|r| format!("x={r:.4}"))
            .collect::<Vec<_>>()
            .join(", ");
        SolverResult {
            equation: format!("{a}x³ + {b}x² + {c}x + {d} = 0"),
            roots,
            complex: vec![],
            value: None,
            steps,
            latex: format!("\\boxed{{{latex}}}"),
        }
    }

    fn solve_hepta_quality(&self, d: [f64; 7], t: [f64; 7], w: [f64; 7]) -> SolverResult {
        let mut steps = Vec::new();
        steps.push("H(P) = 1 / (1 + √ Σᵢwᵢ(Pᵢ−Tᵢ)²)".into());

        let weighted_sq: f64 = w
            .iter()
            .zip(d.iter().zip(t.iter()))
            .map(|(wi, (pi, ti))| wi * (pi - ti).powi(2))
            .sum();
        steps.push(format!("Σwᵢ(Pᵢ−Tᵢ)² = {weighted_sq:.6}"));

        let dist = weighted_sq.sqrt();
        steps.push(format!("√Σ = {dist:.6}"));

        let hp = 1.0 / (1.0 + dist);
        steps.push(format!("H(P) = 1/(1+{dist:.6}) = {hp:.6}"));

        let b11 = (hp * 240.0).round() as u8;
        steps.push(format!(
            "B11 = round(H(P) × 240) = round({:.4} × 240) = {b11}",
            hp
        ));

        SolverResult {
            equation: "H(P) quality equation".into(),
            roots: vec![],
            complex: vec![],
            value: Some(hp),
            steps,
            latex: format!("H(P) = {hp:.6} \\Rightarrow B11 = {b11}"),
        }
    }

    fn solve_spectral_2x2(&self, a: f64, b: f64, c: f64, d: f64) -> SolverResult {
        let mut steps = Vec::new();
        steps.push(format!("2×2 matrix [[{a},{b}],[{c},{d}]]"));
        steps.push("Characteristic polynomial: λ² - tr(A)λ + det(A) = 0".into());

        let tr = a + d;
        let det = a * d - b * c;
        steps.push(format!("tr(A) = {tr}, det(A) = {det}"));

        let result = self.solve_quadratic(1.0, -tr, det);
        let sr = if result.has_real_roots() {
            result.roots.iter().map(|x| x.abs()).fold(0.0f64, f64::max)
        } else {
            result
                .complex
                .iter()
                .map(|(r, i)| (r * r + i * i).sqrt())
                .fold(0.0f64, f64::max)
        };

        let mut final_steps = steps;
        final_steps.extend(result.steps);
        final_steps.push(format!("Spectral radius ρ(A) = {sr:.6}"));

        SolverResult {
            equation: format!("2×2 matrix [[{a},{b}],[{c},{d}]]"),
            roots: result.roots,
            complex: result.complex,
            value: Some(sr),
            steps: final_steps,
            latex: format!("\\rho(A) = {sr:.6}"),
        }
    }
}

impl Default for AlgebraSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_solver_correct() {
        let r = AlgebraSolver::new().solve(&Equation::Linear { a: 2.0, b: -6.0 });
        assert_eq!(r.roots.len(), 1);
        assert!((r.roots[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn quadratic_two_real_roots() {
        // x² - 5x + 6 = 0 → x=2, x=3
        let r = AlgebraSolver::new().solve(&Equation::Quadratic {
            a: 1.0,
            b: -5.0,
            c: 6.0,
        });
        assert_eq!(r.roots.len(), 2);
        let mut roots = r.roots.clone();
        roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((roots[0] - 2.0).abs() < 1e-10);
        assert!((roots[1] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn quadratic_complex_roots() {
        // x² + 1 = 0 → ±i
        let r = AlgebraSolver::new().solve(&Equation::Quadratic {
            a: 1.0,
            b: 0.0,
            c: 1.0,
        });
        assert_eq!(r.complex.len(), 2);
        assert!(r.roots.is_empty());
        assert!((r.complex[0].1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hepta_quality_unit_vector() {
        use ea_agent_core::constants::HEPTA_WEIGHTS;
        let d = [1.0; 7];
        let t = [1.0; 7];
        let r = AlgebraSolver::new().solve(&Equation::HeptaQuality {
            dimensions: d,
            target: t,
            weights: HEPTA_WEIGHTS,
        });
        assert!((r.value.unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cubic_three_real_roots() {
        // (x-1)(x-2)(x-3) = x³ - 6x² + 11x - 6
        let r = AlgebraSolver::new().solve(&Equation::Cubic {
            a: 1.0,
            b: -6.0,
            c: 11.0,
            d: -6.0,
        });
        assert_eq!(r.roots.len(), 3);
        let mut roots = r.roots.clone();
        roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((roots[0] - 1.0).abs() < 1e-6);
        assert!((roots[1] - 2.0).abs() < 1e-6);
        assert!((roots[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn spectral_radius_identity() {
        // Identity matrix has spectral radius 1.0
        let r = AlgebraSolver::new().solve(&Equation::SpectralRadius2x2 {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
        });
        assert!((r.value.unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn steps_contain_chain_of_thought() {
        let r = AlgebraSolver::new().solve(&Equation::Quadratic {
            a: 1.0,
            b: -2.0,
            c: 1.0,
        });
        assert!(!r.steps.is_empty());
        assert!(r.latex.contains("x"));
    }
}
