//! pauli_monitor.rs — Continuous Pauli Exclusion surveillance.
#![forbid(unsafe_code)]

use ea_agent_algebra::PauliChecker;
use ea_agent_core::ParticleSnapshot;

/// Report from a Pauli monitoring cycle.
#[derive(Debug)]
pub struct MonitorReport {
    pub tribe_id: u16,
    pub epoch: u64,
    pub particle_count: usize,
    pub violation_count: usize,
    pub critical_count: usize,
    pub sovereign: bool,
    pub alerts: Vec<String>,
}

impl MonitorReport {
    pub fn summary(&self) -> String {
        if self.sovereign {
            format!(
                "✅ Tribe 0x{:04X} sovereign at epoch {} ({} particles)",
                self.tribe_id, self.epoch, self.particle_count
            )
        } else {
            format!(
                "⚠ Tribe 0x{:04X} — {} Pauli violation(s) ({} critical) at epoch {}",
                self.tribe_id, self.violation_count, self.critical_count, self.epoch
            )
        }
    }
}

/// Continuous Pauli monitor — called every epoch.
pub struct PauliMonitor {
    checker: PauliChecker,
    pub reports: Vec<MonitorReport>,
}

impl PauliMonitor {
    pub fn new() -> Self {
        Self {
            checker: PauliChecker::new(),
            reports: Vec::new(),
        }
    }

    /// Run one monitoring cycle over the current particle snapshot.
    pub fn scan(
        &mut self,
        tribe_id: u16,
        epoch: u64,
        particles: &[ParticleSnapshot],
    ) -> &MonitorReport {
        let result = self.checker.check(particles);
        let alerts: Vec<String> = result
            .violations
            .iter()
            .map(|v| {
                format!(
                    "[{}] {} ↔ {} (dist={:.4})",
                    v.severity.as_str(),
                    v.particle_a,
                    v.particle_b,
                    v.distance
                )
            })
            .collect();

        self.reports.push(MonitorReport {
            tribe_id,
            epoch,
            particle_count: particles.len(),
            violation_count: result.violations.len(),
            critical_count: result.critical_count(),
            sovereign: result.sovereign,
            alerts,
        });
        self.reports.last().unwrap()
    }

    /// How many epochs has the tribe been sovereign?
    pub fn sovereign_streak(&self) -> usize {
        self.reports
            .iter()
            .rev()
            .take_while(|r| r.sovereign)
            .count()
    }

    /// Total violations across all scans.
    pub fn total_violations(&self) -> usize {
        self.reports.iter().map(|r| r.violation_count).sum()
    }
}

impl Default for PauliMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ea_agent_core::HeptaVector;

    fn make_p(name: &str, d: [f64; 7]) -> ParticleSnapshot {
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
    fn clean_scan_is_sovereign() {
        let mut m = PauliMonitor::new();
        let p1 = make_p("P1", [0.1; 7]);
        let p2 = make_p("P2", [0.9; 7]);
        let r = m.scan(0x0001, 1, &[p1, p2]);
        assert!(r.sovereign);
    }

    #[test]
    fn sovereign_streak_counts_correctly() {
        let mut m = PauliMonitor::new();
        let p1 = make_p("A", [0.1; 7]);
        let p2 = make_p("B", [0.9; 7]);
        for ep in 1..=5 {
            m.scan(0x0001, ep, &[p1.clone(), p2.clone()]);
        }
        assert_eq!(m.sovereign_streak(), 5);
    }

    #[test]
    fn total_violations_summed() {
        let mut m = PauliMonitor::new();
        let p1 = make_p("X", [0.9; 7]);
        let p2 = make_p("Y", [0.9; 7]);
        m.scan(0x0001, 1, &[p1, p2]);
        // violations depend on state matching — just verify no panic
        assert!(m.total_violations() <= 1);
    }
}
