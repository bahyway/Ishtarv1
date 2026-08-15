//! jordan.rs — Jordan Normal Form stability analysis for BahyWay tribes.
//!
//! The Jordan Block structure governs how tribes store, shard, and index particles.
//! Each Tribe is an independent Jordan Block — the spectral radius determines stability.
#![forbid(unsafe_code)]

use crate::matrix::SovereignMatrix;
use ea_agent_core::constants::{JORDAN_STABILITY_THRESHOLD, JORDAN_COLLAPSE_THRESHOLD};
use ea_agent_core::ParticleSnapshot;

/// Stability state of a tribe's Jordan Block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabilityState {
    /// Spectral radius ≥ 0.85 — tribe is sovereign and stable.
    Sovereign,
    /// Spectral radius ∈ [0.50, 0.85) — tribe needs attention.
    Marginal,
    /// Spectral radius < 0.50 — tribe collapse imminent.
    Critical,
    /// Spectral radius = 0 — tribe is void.
    Void,
}

impl StabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sovereign => "✅ SOVEREIGN — Tribe is stable",
            Self::Marginal  => "⚠ MARGINAL — Monitor orbit decay",
            Self::Critical  => "🔴 CRITICAL — Collapse imminent",
            Self::Void      => "⚫ VOID — Tribe has collapsed",
        }
    }
    pub fn from_spectral_radius(sr: f64) -> Self {
        if sr < 1e-10          { Self::Void }
        else if sr < JORDAN_COLLAPSE_THRESHOLD  { Self::Critical }
        else if sr < JORDAN_STABILITY_THRESHOLD { Self::Marginal }
        else                   { Self::Sovereign }
    }
}

/// Result of a Jordan stability analysis.
#[derive(Debug)]
pub struct JordanResult {
    pub tribe_id:        u16,
    pub particle_count:  usize,
    pub spectral_radius: f64,
    pub trace:           f64,
    pub determinant:     f64,
    pub state:           StabilityState,
    pub dominant_dim:    Option<usize>,  // dimension causing instability
    pub prediction:      String,
}

impl JordanResult {
    pub fn is_stable(&self) -> bool { self.state == StabilityState::Sovereign }
    pub fn is_critical(&self) -> bool { self.state == StabilityState::Critical }

    pub fn summary(&self) -> String {
        format!(
            "Tribe 0x{:04X} | {} particles | ρ={:.4} | Tr={:.4} | Det={:.6} | {}",
            self.tribe_id, self.particle_count,
            self.spectral_radius, self.trace, self.determinant,
            self.state.as_str()
        )
    }
}

/// The Jordan Stability Analyzer — EaAgent's oracle mode.
pub struct JordanAnalyzer;

impl JordanAnalyzer {
    pub fn new() -> Self { Self }

    /// Analyze the stability of a tribe given its particle snapshots.
    /// Builds the adjacency/similarity matrix and computes spectral radius.
    pub fn analyze(&self, tribe_id: u16, particles: &[ParticleSnapshot]) -> JordanResult {
        let n = particles.len();

        if n == 0 {
            return JordanResult {
                tribe_id, particle_count: 0,
                spectral_radius: 0.0, trace: 0.0, determinant: 0.0,
                state: StabilityState::Void,
                dominant_dim: None,
                prediction: "Empty tribe — no particles to analyze.".into(),
            };
        }

        if n == 1 {
            let b11 = particles[0].b11() as f64 / 240.0;
            return JordanResult {
                tribe_id, particle_count: 1,
                spectral_radius: b11, trace: b11, determinant: b11,
                state: StabilityState::from_spectral_radius(b11),
                dominant_dim: None,
                prediction: format!("Single particle tribe. B11={}", particles[0].b11()),
            };
        }

        // Build similarity matrix: M[i][j] = cosine_similarity(p_i, p_j)
        let mut m = SovereignMatrix::new(n, n);
        for i in 0..n {
            for j in 0..n {
                let sim = particles[i].hepta.cosine_similarity(&particles[j].hepta);
                m.set(i, j, sim);
            }
        }

        let spectral_radius = m.spectral_radius(200);
        let trace           = m.trace();
        let determinant     = if n <= 7 { m.determinant() } else { f64::NAN };
        let state           = StabilityState::from_spectral_radius(spectral_radius);

        // Find the most unstable dimension (highest variance across particles)
        let dominant_dim = if state != StabilityState::Sovereign {
            Some(self.find_unstable_dimension(particles))
        } else {
            None
        };

        let prediction = self.generate_prediction(state, spectral_radius, dominant_dim);

        JordanResult {
            tribe_id, particle_count: n,
            spectral_radius, trace, determinant,
            state, dominant_dim, prediction,
        }
    }

    /// Find which Hepta dimension has the highest variance (instability source).
    fn find_unstable_dimension(&self, particles: &[ParticleSnapshot]) -> usize {
        let n = particles.len() as f64;
        let mut max_var = 0.0f64;
        let mut max_dim = 0;

        for d in 0..7 {
            let mean: f64 = particles.iter().map(|p| p.hepta.d[d]).sum::<f64>() / n;
            let var:  f64 = particles.iter()
                .map(|p| (p.hepta.d[d] - mean).powi(2))
                .sum::<f64>() / n;
            if var > max_var { max_var = var; max_dim = d; }
        }
        max_dim
    }

    fn generate_prediction(&self, state: StabilityState, sr: f64, dim: Option<usize>) -> String {
        let dim_names = ["Identity","Belonging","Domain","Quality","Freshness","Temporal","Integrity"];
        match state {
            StabilityState::Sovereign =>
                format!("Tribe is stable. Spectral radius {sr:.4} ≥ {JORDAN_STABILITY_THRESHOLD}."),
            StabilityState::Marginal => {
                let d = dim.map(|i| dim_names[i]).unwrap_or("unknown");
                format!("Warning: Orbit decay detected in {d} dimension (ρ={sr:.4}). Monitor B11 trend.")
            }
            StabilityState::Critical => {
                let d = dim.map(|i| dim_names[i]).unwrap_or("unknown");
                format!("CRITICAL: Tribe collapse imminent. {d} dimension variance causing instability (ρ={sr:.4}). Invoke Quantum Freeze.")
            }
            StabilityState::Void =>
                "Tribe has collapsed. All particles in Dead state. Archive to EnkiDW.".into(),
        }
    }
}

impl Default for JordanAnalyzer { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use ea_agent_core::HeptaVector;

    fn make_p(name: &str, d: [f64; 7]) -> ParticleSnapshot {
        ParticleSnapshot::new(name, 0x1234, 0x0001, HeptaVector::new(d))
    }

    #[test]
    fn empty_tribe_is_void() {
        let r = JordanAnalyzer::new().analyze(0x0001, &[]);
        assert_eq!(r.state, StabilityState::Void);
    }

    #[test]
    fn gem_tribe_is_sovereign() {
        let particles: Vec<_> = (0..5)
            .map(|i| make_p(&format!("P{i}"), [0.95, 0.90, 0.85, 0.92, 0.88, 0.91, 0.93]))
            .collect();
        let r = JordanAnalyzer::new().analyze(0x0001, &particles);
        assert!(r.spectral_radius > 0.0);
        assert!(r.particle_count == 5);
    }

    #[test]
    fn single_particle_tribe() {
        let p = make_p("Solo", [1.0; 7]);
        let r = JordanAnalyzer::new().analyze(0x0001, &[p]);
        assert_eq!(r.particle_count, 1);
        assert_eq!(r.b11_from_sr(), 240);
    }

    #[test]
    fn stability_state_from_sr() {
        assert_eq!(StabilityState::from_spectral_radius(0.95), StabilityState::Sovereign);
        assert_eq!(StabilityState::from_spectral_radius(0.70), StabilityState::Marginal);
        assert_eq!(StabilityState::from_spectral_radius(0.30), StabilityState::Critical);
        assert_eq!(StabilityState::from_spectral_radius(0.0),  StabilityState::Void);
    }

    #[test]
    fn summary_contains_tribe_id() {
        let r = JordanAnalyzer::new().analyze(0xABCD, &[]);
        assert!(r.summary().contains("ABCD"));
    }
}

impl JordanResult {
    fn b11_from_sr(&self) -> u8 {
        (self.spectral_radius * 240.0).round().min(240.0) as u8
    }
}
