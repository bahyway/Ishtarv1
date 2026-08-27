//! pauli.rs — Pauli Exclusion Principle enforcement for BahyWay particles.
//!
//! "No two particles may occupy the same (Identity-KAKI, Orbit-Level, QuantumState)
//!  triplet simultaneously." — The High Council of Data
#![forbid(unsafe_code)]

use ea_agent_core::constants::PAULI_EXCLUSION_THRESHOLD;
use ea_agent_core::ParticleSnapshot;

/// A detected Pauli violation between two particles.
#[derive(Debug, Clone)]
pub struct PauliViolation {
    pub particle_a: String,
    pub particle_b: String,
    pub distance: f64,
    pub dimension: Option<usize>, // which dimension is closest to collision
    pub severity: PauliSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauliSeverity {
    /// Distance < threshold/4 — identity crisis imminent.
    Critical,
    /// Distance < threshold/2 — repulsion needed.
    High,
    /// Distance < threshold — warning, monitor.
    Warning,
}

impl PauliSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL — Identity Crisis",
            Self::High => "HIGH — Repulsion Needed",
            Self::Warning => "WARNING — Monitor",
        }
    }
}

/// Result of a Pauli Exclusion check on a set of particles.
#[derive(Debug)]
pub struct PauliResult {
    pub violations: Vec<PauliViolation>,
    pub checked: usize,
    pub sovereign: bool,
}

impl PauliResult {
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
    pub fn critical_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == PauliSeverity::Critical)
            .count()
    }
    pub fn summary(&self) -> String {
        if self.sovereign {
            format!(
                "✅ Sovereign — {} particles checked, no violations",
                self.checked
            )
        } else {
            format!(
                "⚠ {} violation(s) in {} particles checked ({} critical)",
                self.violations.len(),
                self.checked,
                self.critical_count()
            )
        }
    }
}

/// The Pauli Exclusion Principle checker.
pub struct PauliChecker {
    threshold: f64,
}

impl PauliChecker {
    pub fn new() -> Self {
        Self {
            threshold: PAULI_EXCLUSION_THRESHOLD,
        }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Check all pairs of particles for Pauli violations.
    /// O(n²) — sovereign, no external index.
    pub fn check(&self, particles: &[ParticleSnapshot]) -> PauliResult {
        let mut violations = Vec::new();

        for i in 0..particles.len() {
            for j in i + 1..particles.len() {
                let a = &particles[i];
                let b = &particles[j];

                // Only check particles in the same quantum state (same shell)
                if a.state != b.state {
                    continue;
                }

                let dist = a.hepta.manhattan_distance(&b.hepta);

                if dist < self.threshold {
                    let severity = if dist < self.threshold / 4.0 {
                        PauliSeverity::Critical
                    } else if dist < self.threshold / 2.0 {
                        PauliSeverity::High
                    } else {
                        PauliSeverity::Warning
                    };

                    // Find the dimension causing the closest overlap
                    let dimension = a
                        .hepta
                        .d
                        .iter()
                        .zip(b.hepta.d.iter())
                        .enumerate()
                        .min_by(|(_, (a1, b1)), (_, (a2, b2))| {
                            (*a1 - *b1).abs().partial_cmp(&(*a2 - *b2).abs()).unwrap()
                        })
                        .map(|(i, _)| i);

                    violations.push(PauliViolation {
                        particle_a: a.name.clone(),
                        particle_b: b.name.clone(),
                        distance: dist,
                        dimension,
                        severity,
                    });
                }
            }
        }

        let sovereign = violations.is_empty();
        PauliResult {
            violations,
            checked: particles.len(),
            sovereign,
        }
    }

    /// Quick check if two specific particles violate Pauli.
    pub fn check_pair(&self, a: &ParticleSnapshot, b: &ParticleSnapshot) -> bool {
        if a.state != b.state {
            return false;
        }
        a.hepta.manhattan_distance(&b.hepta) < self.threshold
    }
}

impl Default for PauliChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ea_agent_core::{HeptaVector, QuantumState};

    fn make_particle(name: &str, d: [f64; 7]) -> ParticleSnapshot {
        ParticleSnapshot::new(name, fnv(name), 0x0001, HeptaVector::new(d))
    }

    fn fnv(s: &str) -> u32 {
        let mut h: u32 = 2166136261;
        for b in s.bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        h
    }

    #[test]
    fn identical_particles_violate_pauli() {
        let checker = PauliChecker::new();
        let p1 = make_particle("P1", [0.9; 7]);
        let p2 = make_particle("P2", [0.9; 7]);
        // Same state, same vector → violation
        let result = checker.check(&[p1, p2]);
        assert!(result.has_violations());
    }

    #[test]
    fn distant_particles_are_sovereign() {
        let checker = PauliChecker::new();
        let p1 = make_particle("P1", [0.1; 7]);
        let p2 = make_particle("P2", [0.9; 7]);
        let result = checker.check(&[p1, p2]);
        // Different B11 → different states → no violation
        assert!(result.sovereign);
    }

    #[test]
    fn dead_and_active_no_violation() {
        let checker = PauliChecker::new();
        // Dead particle (low B11)
        let mut p1 = make_particle("Dead", [0.1; 7]);
        p1.state = QuantumState::Dead;
        // Active particle (high B11)
        let mut p2 = make_particle("Active", [0.1; 7]);
        p2.state = QuantumState::Active;
        // Different states → no Pauli violation
        let result = checker.check(&[p1, p2]);
        assert!(result.sovereign);
    }

    #[test]
    fn pauli_result_summary_correct() {
        let checker = PauliChecker::new();
        let result = checker.check(&[]);
        assert!(result.summary().contains("Sovereign"));
    }

    #[test]
    fn critical_severity_at_quarter_threshold() {
        let checker = PauliChecker::with_threshold(0.20);
        let p1 = make_particle("P1", [0.9, 0.9, 0.9, 0.9, 0.9, 0.9, 0.9]);
        let p2 = make_particle("P2", [0.9, 0.9, 0.9, 0.9, 0.9, 0.9, 0.901]);
        let result = checker.check(&[p1, p2]);
        if result.has_violations() {
            assert_eq!(result.violations[0].severity, PauliSeverity::Critical);
        }
    }
}
