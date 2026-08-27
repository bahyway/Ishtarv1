//! orbit-spectral-engine — nearest-neighbor spacing statistics for orbit
//! return-time sequences. Rhythm, not shape: this crate complements
//! `lamassu-engine` (persistent homology — GOLDEN/FUZZY/DEAD shape
//! signatures) rather than duplicating it. See `docs/01_mathematics/
//! GL-MRD-003-orbit-spectral-diagnostics.md` for the sealed concept this
//! implements and its place in GL-MRD-002's Analysis-to-Solution Law
//! (this crate is the DETECT-phase signal).
//!
//! Input is a sequence of orbit crossing/return times through a Poincaré
//! section (the Nēberu Slicer's own output, once GL-MRD-002 is built —
//! not yet, so this crate's tests use clearly-labeled synthetic fixtures,
//! never fabricated live data). What ships is cheap and closed-form: a
//! mean-normalized spacing histogram compared, via Kolmogorov-Smirnov
//! distance, against two reference distributions —
//!   - Poisson (uncorrelated returns): CDF(s) = 1 - exp(-s)
//!   - GUE Wigner surmise (level-repulsion, β=2): a real closed-form CDF
//!     derived from the standard PDF p(s) = (32/π²) s² exp(-4s²/π) — see
//!     `cdf_gue`'s own doc comment for the derivation.
//!
//! The heavy analytic machinery this is a cheap proxy for (dynamical
//! zeta functions, Selberg/Gutzwiller trace formulas) stays design-time
//! math, per GL-MRD-003 — it is never computed at runtime here.
#![forbid(unsafe_code)]

/// Consecutive differences of a sorted return-time sequence, normalized
/// so the mean spacing is 1.0 (the standard convention for comparing
/// against universal spacing distributions, which are themselves defined
/// on mean-spacing-1 data). Returns an empty vec if fewer than 2 inputs.
///
/// Sorts its own copy of the input — callers do not need to pre-sort.
pub fn spacings(returns: &[f64]) -> Vec<f64> {
    if returns.len() < 2 {
        return Vec::new();
    }
    let mut sorted: Vec<f64> = returns.to_vec();
    sorted.sort_by(|a, b| {
        a.partial_cmp(b)
            .expect("orbit return times must not be NaN")
    });
    let raw: Vec<f64> = sorted.windows(2).map(|w| w[1] - w[0]).collect();
    let mean: f64 = raw.iter().sum::<f64>() / raw.len() as f64;
    if mean <= 0.0 {
        return raw; // degenerate (duplicate/non-increasing times) — caller's problem, not ours to hide
    }
    raw.iter().map(|s| s / mean).collect()
}

/// Abramowitz & Stegun 7.1.26 rational approximation to the error
/// function. Max absolute error ~1.5e-7 — more than sufficient for a
/// runtime diagnostic (this is not the design-time analytic derivation,
/// which per GL-MRD-003 stays in the tablet-house). No external crate
/// needed for this; kept local so the crate has zero dependencies.
pub fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    const P: f64 = 0.3275911;
    const A1: f64 = 0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 = 1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 = 1.061405429;
    let t = 1.0 / (1.0 + P * x);
    let poly = ((((A5 * t + A4) * t + A3) * t + A2) * t + A1) * t;
    sign * (1.0 - poly * (-x * x).exp())
}

/// CDF of the exponential(1) distribution — the nearest-neighbor spacing
/// law for an uncorrelated (Poissonian) point process with mean spacing
/// normalized to 1. Standard, no derivation needed.
pub fn cdf_poisson(s: f64) -> f64 {
    if s <= 0.0 {
        0.0
    } else {
        1.0 - (-s).exp()
    }
}

/// CDF of the GUE (β=2) Wigner surmise, derived here from its standard
/// PDF p(s) = (32/π²) s² exp(-4s²/π) (mean spacing normalized to 1).
///
/// Derivation (verified by differentiating back to the PDF — see the
/// crate's own test `cdf_gue_derivative_matches_pdf`): with c = 4/π,
/// ∫₀ˢ (32/π²) t² e^{-c t²} dt = erf(2s/√π) − (4s/π) e^{-4s²/π}.
/// This is closed-form (no numerical integration at runtime) and uses
/// only the local `erf` approximation above.
pub fn cdf_gue(s: f64) -> f64 {
    if s <= 0.0 {
        return 0.0;
    }
    const FOUR_OVER_PI: f64 = 1.273_239_544_735_162_7; // 4/pi
    erf(std::f64::consts::FRAC_2_SQRT_PI * s) - FOUR_OVER_PI * s * (-FOUR_OVER_PI * s * s).exp()
}

/// PDF of the GUE Wigner surmise — exposed for the derivative-matches
/// test and for callers that want the density directly, not just the CDF.
pub fn pdf_gue(s: f64) -> f64 {
    if s <= 0.0 {
        return 0.0;
    }
    const THIRTY_TWO_OVER_PI_SQ: f64 = 3.242_757_380_608_74; // 32/pi^2
    const FOUR_OVER_PI: f64 = 1.273_239_544_735_162_7;
    THIRTY_TWO_OVER_PI_SQ * s * s * (-FOUR_OVER_PI * s * s).exp()
}

/// Two-sided Kolmogorov-Smirnov distance between an empirical sample and
/// a reference CDF: max over the sorted sample of
/// max(|F_ref(xᵢ) − (i-1)/n|, |F_ref(xᵢ) − i/n|).
/// Standard one-sample KS statistic — sorts its own copy of `sample`.
pub fn ks_distance(sample: &[f64], reference_cdf: impl Fn(f64) -> f64) -> f64 {
    if sample.is_empty() {
        return f64::NAN;
    }
    let mut sorted: Vec<f64> = sample.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("KS sample must not be NaN"));
    let n = sorted.len() as f64;
    let mut max_d = 0.0_f64;
    for (i, &x) in sorted.iter().enumerate() {
        let f_ref = reference_cdf(x);
        let below = (i as f64) / n; // empirical CDF just below x (i points strictly before, since i is 0-indexed count of points before this one)
        let at = ((i + 1) as f64) / n; // empirical CDF at/after x
        max_d = max_d.max((f_ref - below).abs()).max((f_ref - at).abs());
    }
    max_d
}

/// Minimum spacing count below which a classification is not meaningful
/// (too few crossings to distinguish level-repulsion from chance).
pub const MIN_SPACINGS_FOR_CLASSIFICATION: usize = 5;

/// The diagnostic verdict: which universal spacing law the data's rhythm
/// resembles more closely, or a refusal to guess.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpectralSignature {
    /// Level repulsion — closer to GUE than Poisson. In GL-MRD-003's
    /// framing: healthy chaotic orbit dynamics.
    GueLike,
    /// No repulsion — closer to Poisson than GUE. Clumping/rigidity
    /// signal: possible decoupling or pathological locking.
    PoissonLike,
    /// Not enough spacings to distinguish the two laws honestly.
    Ambiguous,
}

impl SpectralSignature {
    pub fn label(&self) -> &'static str {
        match self {
            SpectralSignature::GueLike => "GUE_LIKE",
            SpectralSignature::PoissonLike => "POISSON_LIKE",
            SpectralSignature::Ambiguous => "AMBIGUOUS",
        }
    }
}

/// Full result of classifying one orbit return-time sequence: the
/// verdict plus both KS distances, so a caller (or a WITNESS event) can
/// show its work rather than trust a bare label.
#[derive(Debug, Clone, Copy)]
pub struct SpectralClassification {
    pub signature: SpectralSignature,
    pub ks_gue: f64,
    pub ks_poisson: f64,
    pub spacing_count: usize,
}

/// Classify a raw orbit return-time sequence (need not be pre-sorted).
/// Computes mean-normalized spacings, then the KS distance to each
/// reference law; whichever is smaller wins, unless there are fewer than
/// `MIN_SPACINGS_FOR_CLASSIFICATION` spacings, in which case this refuses
/// to guess (`Ambiguous`) rather than report a number built on noise.
pub fn classify(returns: &[f64]) -> SpectralClassification {
    let gaps = spacings(returns);
    if gaps.len() < MIN_SPACINGS_FOR_CLASSIFICATION {
        return SpectralClassification {
            signature: SpectralSignature::Ambiguous,
            ks_gue: f64::NAN,
            ks_poisson: f64::NAN,
            spacing_count: gaps.len(),
        };
    }
    let ks_gue = ks_distance(&gaps, cdf_gue);
    let ks_poisson = ks_distance(&gaps, cdf_poisson);
    let signature = if ks_gue < ks_poisson {
        SpectralSignature::GueLike
    } else {
        SpectralSignature::PoissonLike
    };
    SpectralClassification {
        signature,
        ks_gue,
        ks_poisson,
        spacing_count: gaps.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erf_matches_known_values() {
        // Standard reference values (Abramowitz & Stegun tables).
        assert!((erf(0.0) - 0.0).abs() < 1e-6);
        assert!((erf(0.5) - 0.520_499_9).abs() < 1e-5);
        assert!((erf(1.0) - 0.842_700_8).abs() < 1e-5);
        assert!((erf(2.0) - 0.995_322_3).abs() < 1e-5);
        assert!((erf(-1.0) + 0.842_700_8).abs() < 1e-5); // odd function
    }

    #[test]
    fn cdf_boundary_conditions() {
        assert_eq!(cdf_poisson(0.0), 0.0);
        assert_eq!(cdf_gue(0.0), 0.0);
        assert!(cdf_poisson(10.0) > 0.9999);
        assert!(cdf_gue(10.0) > 0.9999);
        // Both must be monotonically non-decreasing.
        let xs = [0.0, 0.1, 0.5, 1.0, 1.5, 2.0, 3.0, 5.0];
        for w in xs.windows(2) {
            assert!(cdf_poisson(w[1]) >= cdf_poisson(w[0]));
            assert!(cdf_gue(w[1]) >= cdf_gue(w[0]));
        }
    }

    #[test]
    fn cdf_gue_derivative_matches_pdf() {
        // Numerical derivative of cdf_gue should track pdf_gue closely —
        // this is the actual proof the closed-form CDF derivation (in
        // cdf_gue's own doc comment) is correct, not just plausible.
        // Tolerance is looser than erf()'s own stated ~1.5e-7 error because
        // finite-difference division by 2h amplifies that approximation
        // error -- this test is checking the CDF derivation is correct,
        // not re-deriving erf()'s own precision bound.
        let h = 1e-5;
        for &s in &[0.2, 0.5, 0.8, 1.0, 1.5, 2.0] {
            let numeric_deriv = (cdf_gue(s + h) - cdf_gue(s - h)) / (2.0 * h);
            let analytic = pdf_gue(s);
            assert!(
                (numeric_deriv - analytic).abs() < 5e-4,
                "at s={s}: numeric={numeric_deriv}, analytic={analytic}"
            );
        }
    }

    #[test]
    fn spacings_normalizes_mean_to_one() {
        let returns = vec![0.0, 1.0, 3.0, 4.0, 8.0];
        let gaps = spacings(&returns);
        assert_eq!(gaps.len(), 4);
        let mean: f64 = gaps.iter().sum::<f64>() / gaps.len() as f64;
        assert!((mean - 1.0).abs() < 1e-9);
    }

    #[test]
    fn spacings_handles_unsorted_input() {
        let returns = vec![4.0, 0.0, 8.0, 1.0, 3.0];
        let gaps = spacings(&returns);
        assert_eq!(gaps.len(), 4);
        assert!(gaps.iter().all(|g| *g >= 0.0));
    }

    #[test]
    fn too_few_spacings_is_ambiguous_not_a_guess() {
        let result = classify(&[0.0, 1.0, 2.0]); // only 2 spacings
        assert_eq!(result.signature, SpectralSignature::Ambiguous);
        assert!(result.ks_gue.is_nan());
    }

    /// Deterministic xorshift32 PRNG — used only to build reproducible
    /// synthetic test fixtures below. Not exported; this crate ships no
    /// randomness in its real API.
    fn xorshift32(state: &mut u32) -> f64 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *state = x;
        (x as f64) / (u32::MAX as f64)
    }

    #[test]
    fn classify_recognizes_synthetic_poisson_process() {
        // Real inverse-transform sampling of Exponential(1): spacing_i =
        // -ln(u_i). This IS the Poisson nearest-neighbor law exactly, so
        // classify() should confidently call it PoissonLike.
        let mut state = 0xC0FFEE_u32;
        let mut t = 0.0_f64;
        let mut returns = vec![0.0];
        for _ in 0..200 {
            let u = xorshift32(&mut state).max(1e-9); // avoid ln(0)
            t += -u.ln();
            returns.push(t);
        }
        let result = classify(&returns);
        assert_eq!(result.signature, SpectralSignature::PoissonLike);
        assert!(result.ks_poisson < result.ks_gue);
    }

    #[test]
    fn classify_recognizes_synthetic_repulsion_sequence() {
        // A jittered picket fence: evenly-spaced base positions plus
        // small bounded jitter. This has none of Poisson's characteristic
        // clustering (near-zero gaps) or long exponential tail — the
        // qualitative signature classify() must tell apart from Poisson.
        // Not literally sampled from cdf_gue (that would make the test
        // circular), just a genuinely different, non-Poissonian rhythm.
        let mut state = 0xDEADBEEF_u32;
        let mut returns = Vec::new();
        for i in 0..200 {
            let jitter = (xorshift32(&mut state) - 0.5) * 0.3; // bounded, no long tail
            returns.push(i as f64 + jitter);
        }
        let result = classify(&returns);
        assert_eq!(result.signature, SpectralSignature::GueLike);
        assert!(result.ks_gue < result.ks_poisson);
    }
}
